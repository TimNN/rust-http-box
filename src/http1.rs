// +-----------------------------------------------------------------------------------------------+
// | Copyright 2016 Sean Kerr                                                                      |
// |                                                                                               |
// | Licensed under the Apache License, Version 2.0 (the "License");                               |
// | you may not use this file except in compliance with the License.                              |
// | You may obtain a copy of the License at                                                       |
// |                                                                                               |
// |  http://www.apache.org/licenses/LICENSE-2.0                                                   |
// |                                                                                               |
// | Unless required by applicable law or agreed to in writing, software                           |
// | distributed under the License is distributed on an "AS IS" BASIS,                             |
// | WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.                      |
// | See the License for the specific language governing permissions and                           |
// | limitations under the License.                                                                |
// +-----------------------------------------------------------------------------------------------+
// | Author: Sean Kerr <sean@code-box.org>                                                         |
// +-----------------------------------------------------------------------------------------------+

//! Zero-copy streaming HTTP parser.

#![allow(dead_code)]

use byte::hex_to_byte;
use byte::is_token;

use std::{ fmt,
           str };

// State flag mask.
const FLAG_MASK: u64 = 0x7F;

// State flag shift.
const FLAG_SHIFT: u8 = 1;

// Lower 8 bits mask.
const LOWER8_MASK: u64 = 0xFF;

// Lower 8 bits shift.
const LOWER8_SHIFT: u8 = 8;

// Lower 16 bits mask.
const LOWER16_MASK: u64 = 0xFFFF;

// Lower 16 bits shift.
const LOWER16_SHIFT: u8 = 8;

// Mid 8 bits mask.
const MID8_MASK: u64 = 0xFF;

// Mid 8 bits shift.
const MID8_SHIFT: u8 = 16;

// Upper 40 bits mask.
const UPPER40_MASK: u64 = 0xFFFFFFFFFF;

// Upper 40 bits shift.
const UPPER40_SHIFT: u8 = 24;

// Flags used to track state details.
bitflags! {
    flags Flag: u64 {
        // Parsing chunked transfer encoding.
        const F_CHUNKED = 1 << 0,

        // Parsing data that needs to check against content length.
        const F_CONTENT_LENGTH = 1 << 1,

        // Parsing multipart data.
        const F_MULTIPART = 1 << 2
    }
}

// -------------------------------------------------------------------------------------------------
// BIT DATA MACROS
// -------------------------------------------------------------------------------------------------

// Retrieve the lower 8 bits.
macro_rules! get_lower8 {
    ($parser:expr) => ({
        (($parser.bit_data >> LOWER8_SHIFT) & LOWER8_MASK) as u8
    });
}

// Retrieve the lower 16 bits.
macro_rules! get_lower16 {
    ($parser:expr) => ({
        (($parser.bit_data >> LOWER16_SHIFT) & LOWER16_MASK) as u16
    });
}

// Retrieve the mid 8 bits.
macro_rules! get_mid8 {
    ($parser:expr) => ({
        (($parser.bit_data >> MID8_SHIFT) & MID8_MASK) as u8
    });
}

// Retrieve the upper 40 bits.
macro_rules! get_upper40 {
    ($parser:expr) => ({
        ($parser.bit_data >> UPPER40_SHIFT) & UPPER40_MASK
    });
}

// Indicates that a state flag is set.
macro_rules! has_flag {
    ($parser:expr, $flag:expr) => ({
        (($parser.bit_data >> FLAG_SHIFT) & FLAG_MASK) & $flag.bits == $flag.bits
    });
}

// Set a state flag.
macro_rules! set_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data |= ($flag.bits & FLAG_MASK) << FLAG_SHIFT;
    });
}

// Set the lower 8 bits.
macro_rules! set_lower8 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(LOWER8_MASK << LOWER8_SHIFT);
        $parser.bit_data |= bits << LOWER8_SHIFT;
    });
}

// Set the mid 8 bits.
macro_rules! set_mid8 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(MID8_MASK << MID8_SHIFT);
        $parser.bit_data |= bits << MID8_SHIFT;
    });
}

// Set the lower 16 bits.
macro_rules! set_lower16 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(LOWER16_MASK << LOWER16_SHIFT);
        $parser.bit_data |= bits << LOWER16_SHIFT;
    });
}

// Set the upper 40 bits.
macro_rules! set_upper40 {
    ($parser:expr, $bits:expr) => ({
        let bits = $bits as u64;

        $parser.bit_data &= !(UPPER40_MASK << UPPER40_SHIFT);
        $parser.bit_data |= bits << UPPER40_SHIFT;
    });
}

// Unset a state flag.
macro_rules! unset_flag {
    ($parser:expr, $flag:expr) => ({
        $parser.bit_data &= !(($flag.bits & FLAG_MASK) << FLAG_SHIFT);
    });
}

// -------------------------------------------------------------------------------------------------
// STREAM MACROS
// -------------------------------------------------------------------------------------------------

// Execute a callback and if it returns true, execute a block, otherwise exit with callback status.
macro_rules! callback {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $block:block) => ({
        if $context.handler.$function($data) {
            $block
        } else {
            exit_callback!($parser, $context);
        }
    });

    ($parser:expr, $context:expr, $function:ident, $block:block) => ({
        let slice = collected_bytes!($context);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                $block
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            $block
        }
    });
}

// Execute a callback ignoring the last marked byte, and if it returns true, transition to the next
// state, otherwise exit with callback status.
macro_rules! callback_ignore_transition {
    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        let slice = &$context.stream[$context.mark_index..$context.stream_index - 1];

        set_state!($parser, $state, $state_function);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                transition!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            transition!($parser, $context);
        }
    });
}

// Execute a callback ignoring the last marked byte, and if it returns true, transition to the next
// state, otherwise exit with callback status.
macro_rules! callback_ignore_transition_fast {
    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        let slice = &$context.stream[$context.mark_index..$context.stream_index - 1];

        set_state!($parser, $state, $state_function);

        if slice.len() > 0 {
            if $context.handler.$function(slice) {
                transition_fast!($parser, $context);
            } else {
                exit_callback!($parser, $context);
            }
        } else {
            transition_fast!($parser, $context);
        }
    });
}

// Execute a callback and if it returns true, transition to the next state using transition!(),
// otherwise exit with callback status.
//
// This macro exists to enforce the design decision that after each callback, state must either
// change, or the parser exits with callback status.
macro_rules! callback_transition {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $state:expr,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, $data, {
            transition!($parser, $context);
        });
    });

    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, {
            transition!($parser, $context);
        });
    });
}

// Execute a callback and if it returns true, transition to the next state using transition_fast!(),
// otherwise exit with callback status.
//
// This macro exists to enforce the design decision that after each callback, state must either
// change, or the parser exits with callback status.
macro_rules! callback_transition_fast {
    ($parser:expr, $context:expr, $function:ident, $data:expr, $state:expr,
     $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, $data, {
            transition_fast!($parser, $context);
        });
    });

    ($parser:expr, $context:expr, $function:ident, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);
        callback!($parser, $context, $function, {
            transition_fast!($parser, $context);
        });
    });
}

// Collect base macro.
macro_rules! collect {
    ($parser:expr, $context:expr, $function:ident, $block:block) => ({
        loop {
            if is_eof!($context) {
                callback!($parser, $context, $function, {
                    exit_eof!($parser, $context);
                });
            }

            if $block {
                break;
            }
        }
    });
}

// Collect remaining bytes until content length is zero.
//
// Use the upper 40 bits as the content length.
macro_rules! collect_content_length {
    ($parser:expr, $context:expr) => ({
        exit_if_eof!($parser, $context);

        if has_bytes!($context, get_upper40!($parser) as usize) {
            $context.stream_index += get_upper40!($parser) as usize;

            set_upper40!($parser, 0);

            true
        } else {
            $context.stream_index += $context.stream.len();

            set_upper40!($parser, get_upper40!($parser) as usize - $context.stream.len());

            false
        }
    });
}

// Collect digits as a single numerical value.
macro_rules! collect_digits {
    ($parser:expr, $context:expr, $digit:expr, $max:expr, $byte_error:expr, $eof_block:block) => ({
        loop {
            if is_eof!($context) {
                $eof_block
            }

            next!($context);

            if is_digit!($context.byte) {
                $digit *= 10;
                $digit += ($context.byte - b'0') as u64;

                if $digit > $max {
                    exit_error!($parser, $context, $byte_error($context.byte));
                }
            } else {
                break;
            }
        }
    });
}

// Collect all bytes that are allowed within a quoted value.
macro_rules! collect_quoted {
    ($parser:expr, $context:expr, $function:ident, $stop1:expr, $stop2:expr, $byte_error:expr) => ({
        collect!($parser, $context, $function, {
            next!($context);

            if $stop1 == $context.byte || $stop2 == $context.byte {
                true
            } else if is_non_visible!($context.byte) && $context.byte != b' ' {
                exit_error!($parser, $context, $byte_error($context.byte));
            } else {
                false
            }
        });
    });
}

// Collect all bytes that are considered token.
macro_rules! collect_tokens {
    ($parser:expr, $context:expr, $function:ident, $stop1:expr, $stop2:expr, $stop3:expr,
     $byte_error:expr) => ({
        collect!($parser, $context, $function, {
            next!($context);

            if $stop1 == $context.byte
            || $stop2 == $context.byte
            || $stop3 == $context.byte {
                true
            } else if is_token($context.byte) {
                false
            } else {
                exit_error!($parser, $context, $byte_error($context.byte));
            }
        });
    });

    ($parser:expr, $context:expr, $function:ident, $stop:expr, $byte_error:expr) => ({
        collect!($parser, $context, $function, {
            next!($context);

            if $stop == $context.byte {
                true
            } else if is_token($context.byte) {
                false
            } else {
                exit_error!($parser, $context, $byte_error($context.byte));
            }
        });
    });
}

// Collect all visible 7-bit bytes, which is any non-control byte with the exception of space.
macro_rules! collect_visible {
    ($parser:expr, $context:expr, $function:ident, $stop1:expr, $stop2:expr, $stop3:expr,
     $stop4:expr, $stop5:expr, $byte_error:expr) => ({
        collect!($parser, $context, $function, {
            next!($context);

            if $stop1 == $context.byte
            || $stop2 == $context.byte
            || $stop3 == $context.byte
            || $stop4 == $context.byte
            || $stop5 == $context.byte {
                true
            } else if is_non_visible!($context.byte) {
                exit_error!($parser, $context, $byte_error($context.byte));
            } else {
                false
            }
        });
    });

    ($parser:expr, $context:expr, $function:ident, $stop1:expr, $stop2:expr, $byte_error:expr) => ({
        collect!($parser, $context, $function, {
            next!($context);

            if $stop1 == $context.byte || $stop2 == $context.byte {
                true
            } else if is_non_visible!($context.byte) {
                exit_error!($parser, $context, $byte_error($context.byte));
            } else {
                false
            }
        });
    });

    ($parser:expr, $context:expr, $function:ident, $stop:expr, $byte_error:expr) => ({
        collect!($parser, $context, $function, {
            next!($context);

            if $stop == $context.byte {
                true
            } else if is_non_visible!($context.byte) {
                exit_error!($parser, $context, $byte_error($context.byte));
            } else {
                false
            }
        });
    });

    ($parser:expr, $context:expr, $function:ident, $byte_error:expr) => ({
        collect!($parser, $context, $function, {
            next!($context);

            if is_non_visible!($context.byte) {
                exit_error!($parser, $context, $byte_error($context.byte));
            } else {
                false
            }
        });
    });
}

// Retrieve slice of collected bytes.
macro_rules! collected_bytes {
    ($context:expr) => (
        &$context.stream[$context.mark_index..$context.stream_index];
    );
}

// Consume all linear white space until a non-linear white space byte is found.
macro_rules! consume_linear_space {
    ($parser:expr, $context:expr) => ({
        loop {
            if is_eof!($context) {
                exit_eof!($parser, $context);
            }

            next!($context);

            if $context.byte == b' ' || $context.byte == b'\t' {
                continue;
            } else {
                break;
            }
        }
    });
}

// Exit parser function with a callback status.
macro_rules! exit_callback {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;

        return Ok(ParserValue::Exit(Success::Callback($context.stream_index)));
    });
}

// Exit parser function with an EOF status.
macro_rules! exit_eof {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;

        return Ok(ParserValue::Exit(Success::Eof($context.stream_index)));
    });
}

// Exit parser function with an error.
macro_rules! exit_error {
    ($parser:expr, $context:expr, $error:expr) => ({
        $parser.byte_count += $context.stream_index;
        $parser.state       = State::Dead;

        return Err($error);
    });
}

// Exit parser with finished status.
macro_rules! exit_finished {
    ($parser:expr, $context:expr) => ({
        $parser.byte_count += $context.stream_index;
        $parser.state       = State::Finished;

        return Ok(ParserValue::Exit(Success::Finished($context.stream_index)));
    });
}

// Exit parser function with an EOF status if the stream is EOF, otherwise do nothing.
macro_rules! exit_if_eof {
    ($parser:expr, $context:expr) => ({
        if is_eof!($context) {
            exit_eof!($parser, $context);
        }
    });
}

// Indicates that a specified amount of bytes are available.
macro_rules! has_bytes {
    ($context:expr, $length:expr) => (
        $context.stream_index + $length <= $context.stream.len()
    );
}

// Indicates that we're at the end of the stream.
macro_rules! is_eof {
    ($context:expr) => (
        $context.stream_index == $context.stream.len()
    );
}

// Jump a specified amount of bytes.
macro_rules! jump_bytes {
    ($context:expr, $length:expr) => ({
        $context.stream_index += $length;
    });
}

// Advance the stream one byte.
macro_rules! next {
    ($context:expr) => ({
        $context.stream_index += 1;
        $context.byte          = $context.stream[$context.stream_index - 1];
    });
}

// Peek at a slice of available bytes.
macro_rules! peek_bytes {
    ($context:expr, $length:expr) => (
        &$context.stream[$context.stream_index..$context.stream_index + $length]
    );
}

// Replay the most recent byte by rewinding the stream index 1 byte.
macro_rules! replay {
    ($context:expr) => ({
        $context.stream_index -= 1;
    });
}

// Set state and state function.
macro_rules! set_state {
    ($parser:expr, $state:expr, $state_function:ident) => ({
        $parser.state          = $state;
        $parser.state_function = Parser::$state_function;
    });
}

// Transition to a new state by returning to the beginning of the state loop and then processing
// the next state.
macro_rules! transition {
    ($parser:expr, $context:expr, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        $context.mark_index = $context.stream_index;

        return Ok(ParserValue::Continue);
    });

    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return Ok(ParserValue::Continue);
    });
}

// Transition to a new state by directly calling the next state function.
macro_rules! transition_fast {
    ($parser:expr, $context:expr, $state:expr, $state_function:ident) => ({
        set_state!($parser, $state, $state_function);

        $context.mark_index = $context.stream_index;

        return ($parser.state_function)($parser, $context);
    });

    ($parser:expr, $context:expr) => ({
        $context.mark_index = $context.stream_index;

        return ($parser.state_function)($parser, $context);
    });
}

// -------------------------------------------------------------------------------------------------

/// Connection.
#[derive(Clone,Copy,PartialEq)]
#[repr(u8)]
pub enum Connection {
    /// No connection.
    None,

    /// Close connection.
    Close,

    /// Keep connection alive.
    KeepAlive,

    /// Upgrade connection.
    Upgrade
}

impl fmt::Debug for Connection {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Connection::None => {
                write!(formatter, "Connection::None")
            },
            Connection::Close => {
                write!(formatter, "Connection::Close")
            },
            Connection::KeepAlive => {
                write!(formatter, "Connection::KeepAlive")
            },
            Connection::Upgrade => {
                write!(formatter, "Connection::Upgrade")
            }
        }
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Connection::None => {
                write!(formatter, "None")
            },
            Connection::Close => {
                write!(formatter, "Close")
            },
            Connection::KeepAlive => {
                write!(formatter, "KeepAlive")
            },
            Connection::Upgrade => {
                write!(formatter, "Upgrade")
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Content length.
#[derive(Clone,Copy,PartialEq)]
pub enum ContentLength {
    /// No content length.
    None,

    /// Specified content length.
    Specified(u64)
}

impl fmt::Debug for ContentLength {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentLength::None => {
                write!(formatter, "ContentLength::None")
            },
            ContentLength::Specified(x) => {
                write!(formatter, "ContentLength::Specified({})", x)
            }
        }
    }
}

impl fmt::Display for ContentLength {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentLength::None => {
                write!(formatter, "None")
            },
            ContentLength::Specified(x) => {
                write!(formatter, "{}", x)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Content type.
#[derive(Clone,PartialEq)]
pub enum ContentType {
    /// No content type.
    None,

    /// Multipart content type.
    Multipart(Vec<u8>),

    /// URL encoded content type.
    UrlEncoded,

    /// Other content type.
    Other(Vec<u8>),
}

impl fmt::Debug for ContentType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentType::None => {
                write!(formatter, "None")
            },
            ContentType::Multipart(ref x) => {
                write!(formatter, "ContentType::Multipart({})",
                       str::from_utf8((*x).as_slice()).unwrap())
            },
            ContentType::UrlEncoded => {
                write!(formatter, "ContentType::UrlEncoded")
            },
            ContentType::Other(ref x) => {
                write!(formatter, "ContentType::Other({})",
                       str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ContentType::None => {
                write!(formatter, "None")
            },
            ContentType::Multipart(ref x) => {
                write!(formatter, "{}", str::from_utf8((*x).as_slice()).unwrap())
            },
            ContentType::UrlEncoded => {
                write!(formatter, "UrlEncoded")
            },
            ContentType::Other(ref x) => {
                write!(formatter, "{}", str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser error messages.
#[derive(Clone,Copy,PartialEq)]
pub enum ParserError {
    /// Invalid chunk extension name.
    ChunkExtensionName(u8),

    /// Invalid chunk extension value.
    ChunkExtensionValue(u8),

    /// Invalid chunk size.
    ChunkSize(u8),

    /// Invalid CRLF sequence.
    CrlfSequence(u8),

    /// Parsing has failed, but `Parser::parse()` is executed again.
    Dead,

    /// Invalid header field.
    HeaderField(u8),

    /// Invalid header value.
    HeaderValue(u8),

    /// Maximum chunk extension length has been met.
    MaxChunkExtensionLength,

    /// Maximum content length has been met.
    MaxContentLength,

    /// Maximum multipart boundary length has been met.
    MaxMultipartBoundaryLength,

    /// Missing an expected Content-Length header.
    MissingContentLength,

    /// Invalid request method.
    Method(u8),

    /// Invalid multipart boundary.
    MultipartBoundary(u8),

    /// Invalid status.
    Status(u8),

    /// Invalid status code.
    StatusCode(u8),

    /// Invalid URL character.
    Url(u8),

    /// Invalid URL encoded field.
    UrlEncodedField(u8),

    /// Invalid URL encoded value.
    UrlEncodedValue(u8),

    /// Invalid HTTP version.
    Version(u8),
}

impl fmt::Display for ParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParserError::ChunkExtensionName(ref byte) => {
                write!(formatter, "Invalid chunk extension name at byte {}", byte)
            },
            ParserError::ChunkExtensionValue(ref byte) => {
                write!(formatter, "Invalid chunk extension value at byte {}", byte)
            },
            ParserError::ChunkSize(ref byte) => {
                write!(formatter, "Invalid chunk size at byte {}", byte)
            },
            ParserError::CrlfSequence(ref byte) => {
                write!(formatter, "Invalid CRLF sequence at byte {}", byte)
            },
            ParserError::Dead => {
                write!(formatter, "Parser is dead")
            },
            ParserError::HeaderField(ref byte) => {
                write!(formatter, "Invalid header field at byte {}", byte)
            },
            ParserError::HeaderValue(ref byte) => {
                write!(formatter, "Invalid header value at byte {}", byte)
            },
            ParserError::MaxChunkExtensionLength => {
                write!(formatter, "Maximum chunk extension length exceeded")
            },
            ParserError::MaxContentLength => {
                write!(formatter, "Maximum content length exceeded")
            },
            ParserError::MaxMultipartBoundaryLength => {
                write!(formatter, "Maximum multipart boundary size exceeded")
            },
            ParserError::MissingContentLength => {
                write!(formatter, "Missing content length")
            },
            ParserError::Method(ref byte) => {
                write!(formatter, "Invalid method at byte {}", byte)
            },
            ParserError::MultipartBoundary(ref byte) => {
                write!(formatter, "Invalid multipart boundary at byte {}", byte)
            },
            ParserError::Status(ref byte) => {
                write!(formatter, "Invalid status at byte {}", byte)
            },
            ParserError::StatusCode(ref byte) => {
                write!(formatter, "Invalid status code at byte {}", byte)
            },
            ParserError::Url(ref byte) => {
                write!(formatter, "Invalid URL at byte {}", byte)
            },
            ParserError::UrlEncodedField(ref byte) => {
                write!(formatter, "Invalid URL encoded field at byte {}", byte)
            },
            ParserError::UrlEncodedValue(ref byte) => {
                write!(formatter, "Invalid URL encoded value at byte {}", byte)
            },
            ParserError::Version(ref byte) => {
                write!(formatter, "Invalid HTTP version at byte {}", byte)
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser return values.
pub enum ParserValue {
    /// Continue the parser loop.
    Continue,

    /// Exit the parser loop.
    Exit(Success)
}

/// Parser states.
///
/// These states are in the order that they are processed.
#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(u8)]
pub enum State {
    /// An error was returned from a call to `Parser::parse()`.
    Dead = 1,

    // ---------------------------------------------------------------------------------------------
    // REQUEST
    // ---------------------------------------------------------------------------------------------

    /// Stripping linear white space before method.
    StripRequestMethod,

    /// Parsing request method.
    RequestMethod,

    /// Stripping linear white space before URL.
    StripRequestUrl,

    /// Parsing URL.
    RequestUrl,

    /// Stripping linear white space before request HTTP version.
    StripRequestHttp,

    /// Parsing request HTTP version byte 1.
    RequestHttp1,

    /// Parsing request HTTP version byte 2.
    RequestHttp2,

    /// Parsing request HTTP version byte 3.
    RequestHttp3,

    /// Parsing request HTTP version byte 4.
    RequestHttp4,

    /// Parsing request HTTP version byte 5.
    RequestHttp5,

    /// Parsing request HTTP major version.
    RequestVersionMajor,

    /// Parsing request HTTP minor version.
    RequestVersionMinor,

    // ---------------------------------------------------------------------------------------------
    // RESPONSE
    // ---------------------------------------------------------------------------------------------

    /// Stripping linear white space before response HTTP version.
    StripResponseHttp,

    /// Parsing response HTTP version byte 1.
    ResponseHttp1,

    /// Parsing response HTTP version byte 2.
    ResponseHttp2,

    /// Parsing response HTTP version byte 3.
    ResponseHttp3,

    /// Parsing response HTTP version byte 4.
    ResponseHttp4,

    /// Parsing response HTTP version byte 5.
    ResponseHttp5,

    /// Parsing response HTTP major version.
    ResponseVersionMajor,

    /// Parsing response HTTP minor version.
    ResponseVersionMinor,

    /// Stripping linear white space before response status code.
    StripResponseStatusCode,

    /// Parsing response status code.
    ResponseStatusCode,

    /// Stripping linear white space before response status.
    StripResponseStatus,

    /// Parsing response status.
    ResponseStatus,

    // ---------------------------------------------------------------------------------------------
    // HEADERS
    // ---------------------------------------------------------------------------------------------

    // pre-header states:
    //   These only exist purely to avoid the situation where a client can send an initial
    //   request/response line then CRLF[SPACE], and the parser would have assumed the next
    //   piece of content is the second line of a multiline header value.
    //
    //   In addition to this, multiline header value support has been deprecated, but we'll keep
    //   support for now: https://tools.ietf.org/html/rfc7230#section-3.2.4

    /// Parsing pre-header line feed.
    PreHeaders1,

    /// Parsing pre-header potential carriage return.
    PreHeaders2,

    /// Stripping linear white space before header field.
    StripHeaderField,

    /// Parsing first byte of header field.
    FirstHeaderField,

    /// Parsing header field.
    HeaderField,

    /// Stripping linear white space before header value.
    StripHeaderValue,

    /// Parsing header value.
    HeaderValue,

    /// Parsing quoted header value.
    HeaderQuotedValue,

    /// Parsing escaped header value.
    HeaderEscapedValue,

    /// Parsing first carriage return after header value.
    Newline1,

    /// Parsing first line feed after header value.
    Newline2,

    /// Parsing second carriage return after header value.
    Newline3,

    /// Parsing second line feed after header value.
    Newline4,

    // ---------------------------------------------------------------------------------------------
    // BODY
    // ---------------------------------------------------------------------------------------------

    /// Parsing body.
    Body,

    /// Unparsable content.
    Content,

    /// Parsing chunk size.
    ChunkSize,

    /// Parsing chunk extension name.
    ChunkExtensionName,

    /// Parsing chunk extension value.
    ChunkExtensionValue,

    /// Parsing quoted chunk extension value.
    ChunkExtensionQuotedValue,

    /// Parsing escaped chunk extension value.
    ChunkExtensionEscapedValue,

    /// Parsing potential semi-colon or carriage return after chunk extension quoted value.
    ChunkExtensionSemiColon,

    /// Parsing line feed after chunk size.
    ChunkSizeNewline,

    /// Parsing chunk data.
    ChunkData,

    /// Parsing carriage return after chunk data.
    ChunkDataNewline1,

    /// Parsing line feed after chunk data.
    ChunkDataNewline2,

    /// Parsing first hyphen before and after multipart boundary.
    MultipartHyphen1,

    /// Parsing second hyphen before and after multipart boundary.
    MultipartHyphen2,

    /// Parsing multipart boundary.
    MultipartBoundary,

    /// Parsing carriage return after multipart boundary.
    MultipartNewline1,

    /// Parsing line feed after multipart boundary.
    MultipartNewline2,

    /// Parsing multipart data.
    MultipartData,

    /// Parsing URL encoded field.
    UrlEncodedField,

    /// Parsing URL encoded field ampersand.
    UrlEncodedFieldAmpersand,

    /// Parsing URL encoded field hex sequence.
    UrlEncodedFieldHex,

    /// Parsing URL encoded field plus sign.
    UrlEncodedFieldPlus,

    /// Parsing URL encoded value.
    UrlEncodedValue,

    /// Parsing URL encoded value hex sequence.
    UrlEncodedValueHex,

    /// Parsing URL encoded value plus sign.
    UrlEncodedValuePlus,

    // ---------------------------------------------------------------------------------------------
    // FINISHED
    // ---------------------------------------------------------------------------------------------

    /// Parsing carriage return at end of message.
    FinishedNewline1,

    /// Parsing line feed at end of message.
    FinishedNewline2,

    /// Parsing has finished successfully.
    Finished
}

// -------------------------------------------------------------------------------------------------

/// Transfer encoding.
#[derive(Clone,PartialEq)]
pub enum TransferEncoding {
    /// No transfer encoding.
    None,

    /// Chunked transfer encoding.
    Chunked,

    /// Compress transfer encoding.
    Compress,

    /// Deflate transfer encoding.
    Deflate,

    /// Gzip transfer encoding.
    Gzip,

    /// Other transfer encoding.
    Other(Vec<u8>)
}

impl fmt::Debug for TransferEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransferEncoding::None => {
                write!(formatter, "TransferEncoding::None")
            },
            TransferEncoding::Chunked => {
                write!(formatter, "TransferEncoding::Chunked")
            },
            TransferEncoding::Compress => {
                write!(formatter, "TransferEncoding::Compress")
            },
            TransferEncoding::Deflate => {
                write!(formatter, "TransferEncoding::Deflate")
            },
            TransferEncoding::Gzip => {
                write!(formatter, "TransferEncoding::Gzip")
            },
            TransferEncoding::Other(ref x) => {
                write!(formatter, "TransferEncoding::Other({})",
                       str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
}

impl fmt::Display for TransferEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransferEncoding::None => {
                write!(formatter, "None")
            },
            TransferEncoding::Chunked => {
                write!(formatter, "Chunked")
            },
            TransferEncoding::Compress => {
                write!(formatter, "Compress")
            },
            TransferEncoding::Deflate => {
                write!(formatter, "Deflate")
            },
            TransferEncoding::Gzip => {
                write!(formatter, "Gzip")
            },
            TransferEncoding::Other(ref x) => {
                write!(formatter, "{}", str::from_utf8((*x).as_slice()).unwrap())
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser state function type.
pub type StateFunction<T> = fn(&mut Parser<T>, &mut ParserContext<T>)
    -> Result<ParserValue, ParserError>;

/// Type that handles HTTP parser events.
#[allow(unused_variables)]
pub trait HttpHandler {
    /// Retrieve the most recent Connection header.
    fn get_connection(&mut self) -> Connection {
        Connection::None
    }

    /// Retrieve the most recent Content-Length header.
    fn get_content_length(&mut self) -> ContentLength {
        ContentLength::None
    }

    /// Retrieve the most recent Content-Type header.
    fn get_content_type(&mut self) -> ContentType {
        ContentType::None
    }

    /// Retrieve the most recent Transfer-Encoding header.
    fn get_transfer_encoding(&mut self) -> TransferEncoding {
        TransferEncoding::None
    }

    /// Callback that is executed when raw body data has been received.
    ///
    /// This may be executed multiple times in order to supply the entire body.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_body(&mut self, body: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk of data has been parsed.
    ///
    /// This may be executed multiple times in order to supply the entire chunk.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension name has been located.
    ///
    /// This may be executed multiple times in order to supply the entire chunk extension name.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk extension value has been located.
    ///
    /// This may be executed multiple times in order to supply the entire chunk extension value.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a chunk size has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_chunk_size(&mut self, size: u64) -> bool {
        true
    }

    /// Callback that is executed when a header field has been located.
    ///
    /// This may be executed multiple times in order to supply the entire header field.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_header_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a header value has been located.
    ///
    /// This may be executed multiple times in order to supply the entire header value.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_header_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when header parsing has completed successfully.
    fn on_headers_finished(&mut self) -> bool {
        true
    }

    /// Callback that is executed when a request method has been located.
    ///
    /// This may be executed multiple times in order to supply the entire method.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_method(&mut self, method: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when multipart data has been located.
    ///
    /// This may be executed multiple times in order to supply the entire piece of data.
    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status has been located.
    ///
    /// This may be executed multiple times in order to supply the entire status.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_status(&mut self, status: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a response status code has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_status_code(&mut self, code: u16) -> bool {
        true
    }

    /// Callback that is executed when a request URL/path has been located.
    ///
    /// This may be executed multiple times in order to supply the entire URL/path.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url(&mut self, url: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded field or query string field has been located.
    ///
    /// This may be executed multiple times in order to supply the entire field.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_encoded_field(&mut self, field: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when a URL encoded value or query string value has been located.
    ///
    /// This may be executed multiple times in order to supply the entire value.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL fragment has completed.
    ///
    /// This may be executed multiple times in order to supply the entire fragment.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_fragment(&mut self, fragment: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL host has completed.
    ///
    /// This may be executed multiple times in order to supply the entire fragment.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_host(&mut self, host: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL path has completed.
    ///
    /// This may be executed multiple times in order to supply the entire path.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_path(&mut self, path: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL port has completed.
    ///
    /// This may be executed multiple times in order to supply the entire port.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_port(&mut self, port: u16) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL query string has completed.
    ///
    /// This may be executed multiple times in order to supply the entire query string.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_query_string(&mut self, query_string: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when parsing a URL scheme has completed.
    ///
    /// This may be executed multiple times in order to supply the entire scheme.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_url_scheme(&mut self, scheme: &[u8]) -> bool {
        true
    }

    /// Callback that is executed when the HTTP major version has been located.
    ///
    /// Returns `true` when parsing should continue. Otherwise `false`.
    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        true
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser context data.
pub struct ParserContext<'a, T: HttpHandler + 'a> {
    // Current byte.
    byte: u8,

    // Callback handler.
    handler: &'a mut T,

    // Callback mark index.
    mark_index: usize,

    // Stream data.
    stream: &'a [u8],

    // Stream index.
    stream_index: usize
}

impl<'a, T: HttpHandler + 'a> ParserContext<'a, T> {
    /// Create a new `ParserContext`.
    pub fn new(handler: &'a mut T, stream: &'a [u8]) -> ParserContext<'a, T> {
        ParserContext{ byte:         0,
                       handler:      handler,
                       mark_index:   0,
                       stream:       stream,
                       stream_index: 0 }
    }
}

// -------------------------------------------------------------------------------------------------

/// Parser data.
pub struct Parser<T: HttpHandler> {
    // Bit data that stores parser bit details.
    //
    // Bits 1-8: State flags that are checked when states have a dual purpose, such as when header
    //           parsing states also parse chunk encoding trailers.
    // Macros:   has_flag!(), set_flag!(), unset_flag!()
    //
    // Bits 5-64: Used to store various numbers depending on state. Content length, chunk size,
    //            HTTP major/minor versions are all stored in here. Depending on macro used, more
    //            bits are accessible.
    // Macros:    get_lower8!(), set_lower8!()   -- lower 8 bits
    //            get_mid8!(), set_mid8!()       -- mid 8 bits
    //            get_lower16!(), set_lower16!() -- lower 16 bits
    //                                              (when not using the lower8/mid8 macros)
    //            get_upper40!(), set_upper40!() -- upper 40 bits
    bit_data: u64,

    // Total byte count processed for headers, and body.
    // Once the headers are finished processing, this is reset to 0 to track the body length.
    byte_count: usize,

    // Content type.
    content_type: ContentType,

    // Current state.
    state: State,

    // Current state function.
    state_function: StateFunction<T>
}

// Chunk size macro.
macro_rules! chunk_size {
    ($parser:expr, $context:expr) => ({
        exit_if_eof!($parser, $context);
        next!($context);

        match hex_to_byte(&[$context.byte]) {
            Some(byte) => {
                if get_lower8!($parser) == 10 {
                    // beyond max size
                    exit_error!($parser, $context, ParserError::ChunkSize($context.byte));
                }

                set_upper40!($parser, get_upper40!($parser) << 4);
                set_upper40!($parser, get_upper40!($parser) + byte as u64);
                set_lower8!($parser, get_lower8!($parser) + 1);

                transition_fast!($parser, $context, State::ChunkSize, chunk_size);
            },
            None => {
                if get_lower8!($parser) == 0 {
                    // no size supplied
                    exit_error!($parser, $context, ParserError::ChunkSize($context.byte));
                }

                if get_upper40!($parser) == 0 {
                    callback_transition_fast!($parser, $context,
                                              on_chunk_size, get_upper40!($parser),
                                              State::Newline2, newline2);
                } else if $context.byte == b'\r' {
                    callback_transition_fast!($parser, $context,
                                              on_chunk_size, get_upper40!($parser),
                                              State::ChunkSizeNewline, chunk_size_newline);
                } else if $context.byte == b';' {
                    set_lower16!($parser, 1);

                    callback_transition_fast!($parser, $context,
                                              on_chunk_size, get_upper40!($parser),
                                              State::ChunkExtensionName, chunk_extension_name);
                } else {
                    exit_error!($parser, $context, ParserError::ChunkSize($context.byte));
                }
            }
        }
    });
}

impl<T: HttpHandler> Parser<T> {
    /// Create a new `Parser`.
    fn new(state: State, state_function: StateFunction<T>) -> Parser<T> {
        Parser{ bit_data:       if state == State::StripRequestMethod {
                                    1
                                } else {
                                    0
                                },
                byte_count:     0,
                content_type:   ContentType::None,
                state:          state,
                state_function: state_function }
    }

    /// Create a new `Parser` for request parsing.
    pub fn new_request() -> Parser<T> {
        Parser::new(State::StripRequestMethod, Parser::strip_request_method)
    }

    /// Create a new `Parser` for response parsing.
    pub fn new_response() -> Parser<T> {
        Parser::new(State::StripResponseHttp, Parser::strip_response_http)
    }

    /// Retrieve the processed byte count since the start of the message.
    pub fn get_byte_count(&self) -> usize {
        self.byte_count
    }

    /// Retrieve the state.
    pub fn get_state(&self) -> State {
        self.state
    }

    /// Retrieve the state function.
    pub fn get_state_function(&self) -> StateFunction<T> {
        self.state_function
    }

    /// Parse HTTP data.
    ///
    /// If `Success` is returned, you may resuming parsing data with an additional call to
    /// `Parser::parse()`. If `ParserError` is returned, parsing cannot be resumed without a call
    /// to `Parser::reset()`.
    #[inline]
    pub fn parse(&mut self, handler: &mut T, stream: &[u8]) -> Result<Success, ParserError> {
        let mut context = ParserContext::new(handler, stream);

        loop {
            match (self.state_function)(self, &mut context) {
                Ok(ParserValue::Continue) => {
                },
                Ok(ParserValue::Exit(success)) => {
                    return Ok(success);
                },
                Err(error) => {
                    return Err(error);
                }
            }
        }
    }

    /// Reset the `Parser` back to its original state.
    pub fn reset(&mut self) {
        self.byte_count   = 0;
        self.content_type = ContentType::None;

        if self.bit_data & 1 == 1 {
            self.bit_data       = 1;
            self.state          = State::StripRequestMethod;
            self.state_function = Parser::strip_request_method;
        } else {
            self.bit_data       = 0;
            self.state          = State::StripResponseHttp;
            self.state_function = Parser::strip_response_http;
        }
    }

    // ---------------------------------------------------------------------------------------------
    // HEADER STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn pre_headers1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::PreHeaders2, pre_headers2);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn pre_headers2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline4, newline4);
        } else {
            replay!(context);

            transition_fast!(self, context, State::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn strip_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        replay!(context);

        transition_fast!(self, context, State::FirstHeaderField, first_header_field);
    }

    #[inline]
    #[cfg_attr(test, allow(cyclomatic_complexity))]
    fn first_header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! field {
            ($header:expr, $length:expr) => ({
                jump_bytes!(context, $length);

                callback_transition_fast!(self, context,
                                          on_header_field, $header,
                                          State::StripHeaderValue, strip_header_value);
            });
        }

        if has_bytes!(context, 26) {
            // have enough bytes to compare common header fields immediately, without collecting
            // individual tokens
            if context.byte == b'C' {
                if b"Connection:" == peek_bytes!(context, 11) {
                    field!(b"Connection", 11);
                } else if b"Content-Type:" == peek_bytes!(context, 13) {
                    field!(b"Content-Type", 13);
                } else if b"Content-Length:" == peek_bytes!(context, 15) {
                    field!(b"Content-Length", 15);
                } else if b"Cookie:" == peek_bytes!(context, 7) {
                    field!(b"Cookie", 7);
                } else if b"Cache-Control:" == peek_bytes!(context, 14) {
                    field!(b"Cache-Control", 14);
                } else if b"Content-Security-Policy:" == peek_bytes!(context, 24) {
                    field!(b"Content-Security-Policy", 24);
                }
            } else if context.byte == b'A' {
                if b"Accept:" == peek_bytes!(context, 7) {
                    field!(b"Accept", 7);
                } else if b"Accept-Charset:" == peek_bytes!(context, 15) {
                    field!(b"Accept-Charset", 15);
                } else if b"Accept-Encoding:" == peek_bytes!(context, 16) {
                    field!(b"Accept-Encoding", 16);
                } else if b"Accept-Language:" == peek_bytes!(context, 16) {
                    field!(b"Accept-Language", 16);
                } else if b"Authorization:" == peek_bytes!(context, 14) {
                    field!(b"Authorization", 14);
                }
            } else if context.byte == b'L' {
                if b"Location:" == peek_bytes!(context, 9) {
                    field!(b"Location", 9);
                } else if b"Last-Modified:" == peek_bytes!(context, 14) {
                    field!(b"Last-Modified", 14);
                }
            } else if context.byte == b'P' && b"Pragma:" == peek_bytes!(context, 7) {
                field!(b"Pragma", 7);
            } else if context.byte == b'S' && b"Set-Cookie:" == peek_bytes!(context, 11) {
                field!(b"Set-Cookie", 11);
            } else if context.byte == b'T' && b"Transfer-Encoding:" == peek_bytes!(context, 18) {
                field!(b"Transfer-Encoding", 18);
            } else if context.byte == b'U' {
                if b"User-Agent:" == peek_bytes!(context, 11) {
                    field!(b"User-Agent", 11);
                } else if b"Upgrade:" == peek_bytes!(context, 8) {
                    field!(b"Upgrade", 8);
                }
            } else if context.byte == b'X' {
                if b"X-Powered-By:" == peek_bytes!(context, 13) {
                    field!(b"X-Powered-By", 13);
                } else if b"X-Forwarded-For:" == peek_bytes!(context, 16) {
                    field!(b"X-Forwarded-For", 16);
                } else if b"X-Forwarded-Host:" == peek_bytes!(context, 17) {
                    field!(b"X-Forwarded-Host", 17);
                } else if b"X-XSS-Protection:" == peek_bytes!(context, 17) {
                    field!(b"X-XSS-Protection", 17);
                } else if b"X-WebKit-CSP:" == peek_bytes!(context, 13) {
                    field!(b"X-WebKit-CSP", 13);
                } else if b"X-Content-Security-Policy:" == peek_bytes!(context, 26) {
                    field!(b"X-Content-Security-Policy", 26);
                }
            } else if context.byte == b'W' && b"WWW-Authenticate:" == peek_bytes!(context, 17) {
                field!(b"WWW-Authenticate", 17);
            }
        }

        transition_fast!(self, context, State::HeaderField, header_field);
    }

    #[inline]
    fn header_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(self, context, on_header_field,
                        b':',
                        ParserError::HeaderField);

        callback_ignore_transition_fast!(self, context,
                                         on_header_field,
                                         State::StripHeaderValue, strip_header_value);
    }

    #[inline]
    fn strip_header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        if context.byte == b'"' {
            transition_fast!(self, context, State::HeaderQuotedValue, header_quoted_value);
        }

        replay!(context);

        transition_fast!(self, context, State::HeaderValue, header_value);
    }

    #[inline]
    fn header_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(self, context, on_header_value,
                         b'\r',
                         ParserError::HeaderValue);

        callback_ignore_transition_fast!(self, context,
                                         on_header_value,
                                         State::Newline2, newline2);
    }

    #[inline]
    fn header_quoted_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_quoted!(self, context, on_header_value,
                        b'"', b'\\',
                        ParserError::HeaderValue);

        if context.byte == b'"' {
            callback_ignore_transition_fast!(self, context,
                                             on_header_value,
                                             State::Newline1, newline1);
        } else {
            callback_ignore_transition_fast!(self, context,
                                             on_header_value,
                                             State::HeaderEscapedValue, header_escaped_value);
        }
    }

    #[inline]
    fn header_escaped_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        callback_transition!(self, context,
                             on_header_value, &[context.byte],
                             State::HeaderQuotedValue, header_quoted_value);
    }

    #[inline]
    fn newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if has_bytes!(context, 2) && b"\r\n" == peek_bytes!(context, 2) {
            transition_fast!(self, context, State::Newline3, newline3);
        }

        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline2, newline2);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::Newline3, newline3);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn newline3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::Newline4, newline4);
        } else if context.byte == b' ' || context.byte == b'\t' {
            // multiline header value
            callback_transition!(self, context,
                                 on_header_value, b" ",
                                 State::StripHeaderValue, strip_header_value);
        } else {
            replay!(context);
            transition!(self, context, State::StripHeaderField, strip_header_field);
        }
    }

    #[inline]
    fn newline4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            if has_flag!(self, F_CHUNKED) {
                context.handler.on_headers_finished();

                transition_fast!(self, context, State::Finished, finished);
            }

            set_state!(self, State::Body, body);

            if context.handler.on_headers_finished() {
                transition_fast!(self, context);
            } else {
                exit_callback!(self, context);
            }
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    // ---------------------------------------------------------------------------------------------
    // REQUEST STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn strip_request_method(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        replay!(context);

        transition_fast!(self, context, State::RequestMethod, request_method);
    }

    #[inline]
    fn request_method(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! method {
            ($method:expr, $length:expr) => (
                jump_bytes!(context, $length);

                callback_transition_fast!(self, context,
                                          on_method, $method,
                                          State::StripRequestUrl, strip_request_url);
            );
        }

        if has_bytes!(context, 8) {
            // have enough bytes to compare all known methods immediately, without collecting
            // individual tokens

            // get the first byte, then replay it (for use with peek_bytes!())
            next!(context);
            replay!(context);

            if context.byte == b'G' && b"GET " == peek_bytes!(context, 4) {
                method!(b"GET", 4);
            } else if context.byte == b'P' {
                if b"POST " == peek_bytes!(context, 5) {
                    method!(b"POST", 5);
                } else if b"PUT " == peek_bytes!(context, 4) {
                    method!(b"PUT", 4);
                }
            } else if context.byte == b'D' && b"DELETE " == peek_bytes!(context, 7) {
                method!(b"DELETE", 7);
            } else if context.byte == b'C' && b"CONNECT " == peek_bytes!(context, 8) {
                method!(b"CONNECT", 8);
            } else if context.byte == b'O' && b"OPTIONS " == peek_bytes!(context, 8) {
                method!(b"OPTIONS", 8);
            } else if context.byte == b'H' && b"HEAD " == peek_bytes!(context, 5) {
                method!(b"HEAD", 5);
            } else if context.byte == b'T' && b"TRACE " == peek_bytes!(context, 6) {
                method!(b"TRACE", 6);
            }
        }

        collect_tokens!(self, context, on_method,
                        b' ',
                        ParserError::Method);

        replay!(context);

        callback_transition_fast!(self, context,
                                  on_method,
                                  State::StripRequestUrl, strip_request_url);
    }

    #[inline]
    fn strip_request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        replay!(context);

        transition_fast!(self, context, State::RequestUrl, request_url);
    }

    #[inline]
    fn request_url(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(self, context, on_url,
                         b' ',
                         ParserError::Url);

        replay!(context);

        callback_transition_fast!(self, context,
                                  on_url,
                                  State::StripRequestHttp, strip_request_http);
    }

    #[inline]
    fn strip_request_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        replay!(context);

        transition_fast!(self, context, State::RequestHttp1, request_http1);
    }

    #[inline]
    fn request_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                jump_bytes!(context, $length);
                set_state!(self, State::PreHeaders1, pre_headers1);

                if context.handler.on_version($major, $minor) {
                    transition_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            );
        }

        if has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            if b"HTTP/1.1\r" == peek_bytes!(context, 9) {
                version!(1, 1, 9);
            } else if b"HTTP/2.0\r" == peek_bytes!(context, 9) {
                version!(2, 0, 9);
            } else if b"HTTP/1.0\r" == peek_bytes!(context, 9) {
                version!(1, 0, 9);
            }
        }

        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'H' || context.byte == b'h' {
            transition_fast!(self, context, State::RequestHttp2, request_http2);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn request_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::RequestHttp3, request_http3);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn request_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::RequestHttp4, request_http4);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn request_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, State::RequestHttp5, request_http5);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn request_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'/' {
            set_upper40!(self, 0);

            transition_fast!(self, context, State::RequestVersionMajor, request_version_major);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn request_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower16!(self) as u64;

        collect_digits!(self, context, digit, 999, ParserError::Version, {
            set_lower16!(self, digit as u16);

            exit_eof!(self, context);
        });

        set_lower16!(self, digit as u16);

        if context.byte == b'.' {
            transition_fast!(self, context, State::RequestVersionMinor, request_version_minor);
        }

        exit_error!(self, context, ParserError::Version(context.byte));
    }

    #[inline]
    fn request_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper40!(self);

        collect_digits!(self, context, digit, 999, ParserError::Version, {
            set_upper40!(self, digit);

            exit_eof!(self, context);
        });

        set_state!(self, State::PreHeaders1, pre_headers1);

        if context.handler.on_version(get_lower16!(self), digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // RESPONSE STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn strip_response_http(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        replay!(context);

        transition_fast!(self, context, State::ResponseHttp1, response_http1);
    }

    #[inline]
    fn response_http1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        macro_rules! version {
            ($major:expr, $minor:expr, $length:expr) => (
                jump_bytes!(context, $length);
                set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

                if context.handler.on_version($major, $minor) {
                    transition_fast!(self, context);
                } else {
                    exit_callback!(self, context);
                }
            );
        }

        if has_bytes!(context, 9) {
            // have enough bytes to compare all known versions immediately, without collecting
            // individual tokens
            if b"HTTP/1.1 " == peek_bytes!(context, 9) {
                version!(1, 1, 9);
            } else if b"HTTP/2.0 " == peek_bytes!(context, 9) {
                version!(2, 0, 9);
            } else if b"HTTP/1.0 " == peek_bytes!(context, 9) {
                version!(1, 0, 9);
            }
        }

        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'H' || context.byte == b'h' {
            transition_fast!(self, context, State::ResponseHttp2, response_http2);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn response_http2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::ResponseHttp3, response_http3);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn response_http3(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'T' || context.byte == b't' {
            transition_fast!(self, context, State::ResponseHttp4, response_http4);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn response_http4(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'P' || context.byte == b'p' {
            transition_fast!(self, context, State::ResponseHttp5, response_http5);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn response_http5(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'/' {
            set_upper40!(self, 0);

            transition_fast!(self, context, State::ResponseVersionMajor, response_version_major);
        } else {
            exit_error!(self, context, ParserError::Version(context.byte));
        }
    }

    #[inline]
    fn response_version_major(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_lower16!(self) as u64;

        collect_digits!(self, context, digit, 999, ParserError::Version, {
            set_lower16!(self, digit as u16);

            exit_eof!(self, context);
        });

        set_lower16!(self, digit as u16);

        if context.byte == b'.' {
            transition_fast!(self, context, State::ResponseVersionMinor, response_version_minor);
        }

        exit_error!(self, context, ParserError::Version(context.byte));
    }

    #[inline]
    fn response_version_minor(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper40!(self);

        collect_digits!(self, context, digit, 999, ParserError::Version, {
            set_upper40!(self, digit);

            exit_eof!(self, context);
        });

        set_state!(self, State::StripResponseStatusCode, strip_response_status_code);

        if context.handler.on_version(get_lower16!(self), digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);

        if !is_digit!(context.byte) {
            exit_error!(self, context, ParserError::StatusCode(context.byte));
        }

        replay!(context);

        set_upper40!(self, 0);

        transition_fast!(self, context, State::ResponseStatusCode, response_status_code);
    }

    #[inline]
    fn response_status_code(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        let mut digit = get_upper40!(self);

        collect_digits!(self, context, digit, 999, ParserError::StatusCode, {
            set_upper40!(self, digit);
            exit_eof!(self, context);
        });

        replay!(context);
        set_state!(self, State::StripResponseStatus, strip_response_status);

        if context.handler.on_status_code(digit as u16) {
            transition_fast!(self, context);
        } else {
            exit_callback!(self, context);
        }
    }

    #[inline]
    fn strip_response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        consume_linear_space!(self, context);
        replay!(context);

        transition_fast!(self, context, State::ResponseStatus, response_status);
    }

    #[inline]
    fn response_status(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect!(self, context, on_status, {
            next!(context);

            if context.byte == b'\r' {
                true
            } else if is_token(context.byte) || context.byte == b' ' || context.byte == b'\t' {
                false
            } else {
                exit_error!(self, context, ParserError::Status(context.byte));
            }
        });

        callback_ignore_transition_fast!(self, context,
                                         on_status,
                                         State::PreHeaders1, pre_headers1);
    }

    // ---------------------------------------------------------------------------------------------
    // BODY STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn body(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if context.handler.get_transfer_encoding() == TransferEncoding::Chunked {
            set_upper40!(self, 0);
            set_lower8!(self, 0);
            set_flag!(self, F_CHUNKED);

            transition!(self, context, State::ChunkSize, chunk_size);
        } else {
            self.content_type = context.handler.get_content_type();

            match self.content_type {
                ContentType::UrlEncoded => {
                    transition_fast!(self, context, State::UrlEncodedField, url_encoded_field);
                },
                _ => {
                    println!("This content type is not handled yet");
                }
            }
        }

        exit_eof!(self, context);
    }

    #[inline]
    fn content(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    fn chunk_size(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        chunk_size!(self, context);
    }

    #[inline]
    fn chunk_extension_name(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(self, context, on_chunk_extension_name,
                        b'=',
                        ParserError::ChunkExtensionName);

        callback_ignore_transition_fast!(self, context,
                                         on_chunk_extension_name,
                                         State::ChunkExtensionValue, chunk_extension_value);
    }

    #[inline]
    fn chunk_extension_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_tokens!(self, context, on_chunk_extension_value,
                        b'\r', b';', b'"',
                        ParserError::ChunkExtensionValue);

        match context.byte {
            b'\r' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_chunk_extension_value,
                                                 State::ChunkSizeNewline, chunk_size_newline);
            },
            b';' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_chunk_extension_value,
                                                 State::ChunkExtensionName, chunk_extension_name);
            },
            _ => {
                transition_fast!(self, context, State::ChunkExtensionQuotedValue,
                                 chunk_extension_quoted_value);
            }
        }
    }

    #[inline]
    fn chunk_extension_quoted_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_quoted!(self, context, on_chunk_extension_value,
                        b'"', b'\\',
                        ParserError::ChunkExtensionValue);

        if context.byte == b'"' {
            callback_ignore_transition_fast!(self, context,
                                             on_chunk_extension_value,
                                             State::ChunkExtensionSemiColon,
                                             chunk_extension_semi_colon);
        } else {
            callback_ignore_transition_fast!(self, context,
                                             on_chunk_extension_value,
                                             State::ChunkExtensionEscapedValue,
                                             chunk_extension_escaped_value);
        }
    }

    #[inline]
    fn chunk_extension_escaped_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if is_visible!(context.byte) || context.byte == b' ' {
            callback_transition_fast!(self, context,
                                      on_chunk_extension_value, &[context.byte],
                                      State::ChunkExtensionQuotedValue,
                                      chunk_extension_quoted_value);
        }

        exit_error!(self, context, ParserError::ChunkExtensionValue(context.byte));
    }

    #[inline]
    fn chunk_extension_semi_colon(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b';' {
            transition!(self, context, State::ChunkExtensionName, chunk_extension_name);
        } else if context.byte == b'\r' {
            transition!(self, context, State::ChunkSizeNewline, chunk_size_newline);
        }

        exit_error!(self, context, ParserError::ChunkExtensionName(context.byte));
    }

    #[inline]
    fn chunk_size_newline(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            transition!(self, context, State::ChunkData, chunk_data);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn chunk_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if collect_content_length!(self, context) {
            callback_transition!(self, context,
                                 on_chunk_data,
                                 State::ChunkDataNewline1, chunk_data_newline1);
        }

        callback_transition!(self, context,
                             on_chunk_data,
                             State::ChunkData, chunk_data);
    }

    #[inline]
    fn chunk_data_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::ChunkDataNewline2, chunk_data_newline2);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn chunk_data_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::ChunkSize, chunk_size);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn multipart_hyphen1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    fn multipart_hyphen2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    fn multipart_boundary(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    fn multipart_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    fn multipart_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    fn multipart_data(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_eof!(self, context);
    }

    #[inline]
    fn url_encoded_field(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(self, context, on_url_encoded_field,
                         b'=', b'%', b'&', b'+', b'\r',
                         ParserError::UrlEncodedField);

        match context.byte {
            b'=' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedValue, url_encoded_value);
            },
            b'%' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedFieldHex, url_encoded_field_hex);
            },
            b'&' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedFieldAmpersand,
                                                 url_encoded_field_ampersand);
            },
            b'+' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::UrlEncodedFieldPlus,
                                                 url_encoded_field_plus);
            },
            _ => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_field,
                                                 State::FinishedNewline2, finished_newline2);
            }
        }
    }

    #[inline]
    fn url_encoded_field_ampersand(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        // param field without a value, so send an empty array
        callback_transition!(self, context,
                             on_url_encoded_value, b"",
                             State::UrlEncodedField, url_encoded_field);
    }

    #[inline]
    fn url_encoded_field_hex(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if has_bytes!(context, 2) {
            jump_bytes!(context, 2);

            match hex_to_byte(collected_bytes!(context)) {
                Some(byte) => {
                    callback_transition!(self, context,
                                         on_url_encoded_field, &[byte],
                                         State::UrlEncodedField, url_encoded_field);
                },
                _ => {
                    exit_error!(self, context, ParserError::UrlEncodedField(context.byte));
                }
            }
        }

        exit_eof!(self, context);
    }

    #[inline]
    fn url_encoded_field_plus(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        callback_transition!(self, context,
                             on_url_encoded_field, b" ",
                             State::UrlEncodedField, url_encoded_field);
    }

    #[inline]
    fn url_encoded_value(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        collect_visible!(self, context, on_url_encoded_value,
                         b'%', b'&', b'+', b'\r', b'=',
                         ParserError::UrlEncodedValue);

        match context.byte {
            b'%' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 State::UrlEncodedValueHex, url_encoded_value_hex);
            },
            b'&' => {
                callback_ignore_transition!(self, context,
                                            on_url_encoded_value,
                                            State::UrlEncodedField,
                                            url_encoded_field);
            },
            b'+' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 State::UrlEncodedValuePlus,
                                                 url_encoded_value_plus);
            },
            b'\r' => {
                callback_ignore_transition_fast!(self, context,
                                                 on_url_encoded_value,
                                                 State::FinishedNewline2, finished_newline2);
            },
            _ => {
                exit_error!(self, context, ParserError::UrlEncodedValue(context.byte));
            }
        }
    }

    #[inline]
    fn url_encoded_value_hex(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        if has_bytes!(context, 2) {
            jump_bytes!(context, 2);

            match hex_to_byte(collected_bytes!(context)) {
                Some(byte) => {
                    callback_transition!(self, context,
                                         on_url_encoded_value, &[byte],
                                         State::UrlEncodedValue, url_encoded_value);
                },
                _ => {
                    exit_error!(self, context, ParserError::UrlEncodedValue(context.byte));
                }
            }
        }

        exit_eof!(self, context);
    }

    #[inline]
    fn url_encoded_value_plus(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        callback_transition!(self, context,
                             on_url_encoded_value, b" ",
                             State::UrlEncodedValue, url_encoded_value);
    }

    // ---------------------------------------------------------------------------------------------
    // FINISHED STATES
    // ---------------------------------------------------------------------------------------------

    #[inline]
    fn finished_newline1(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\r' {
            transition_fast!(self, context, State::FinishedNewline2, finished_newline2);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn finished_newline2(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_if_eof!(self, context);
        next!(context);

        if context.byte == b'\n' {
            transition_fast!(self, context, State::Finished, finished);
        }

        exit_error!(self, context, ParserError::CrlfSequence(context.byte));
    }

    #[inline]
    fn finished(&mut self, context: &mut ParserContext<T>)
    -> Result<ParserValue, ParserError> {
        exit_finished!(self, context);
    }
}

// -------------------------------------------------------------------------------------------------

/// Success response types.
#[derive(Clone,Copy,PartialEq)]
pub enum Success {
    /// Callback returned false.
    Callback(usize),

    /// Additional data expected.
    Eof(usize),

    /// Finished successfully.
    Finished(usize)
}

impl fmt::Debug for Success {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Success::Callback(length) => {
                write!(formatter, "Success::Callback({})", length)
            },
            Success::Eof(length) => {
                write!(formatter, "Success::Eof({})", length)
            },
            Success::Finished(length) => {
                write!(formatter, "Success::Finished({})", length)
            }
        }
    }
}

impl fmt::Display for Success {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Success::Callback(length) => {
                write!(formatter, "{}", length)
            },
            Success::Eof(length) => {
                write!(formatter, "{}", length)
            },
            Success::Finished(length) => {
                write!(formatter, "{}", length)
            }
        }
    }
}
