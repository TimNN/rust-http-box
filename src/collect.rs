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

//! Stream collection macros.

/// Collect and convert all digit bytes into a u16 variable.
///
/// Exit the collection loop upon finding a non-digit byte. Return `$error` if `$digit` exceeds
/// `$max`, or if an overflow would occur.
macro_rules! collect_digits16 {
    ($context:expr, $error:expr, $digit:expr, $max:expr, $on_eos:expr) => ({
        bs_collect_digits32!($context, $digit,
            if $digit > $max {
                return Err($error($context.byte));
            },
            return Err($error($context.byte)),
            $on_eos
        );
    });
}

/// Collect an unquoted field value.
macro_rules! collect_field {
    ($context:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context, {
                if $context.byte > 0x1F && $context.byte < 0x7F {
                    // space + visible
                    if $stop {
                        break;
                    }
                } else {
                    break;
                }
            },
            $on_eos
        );
    });

    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context, {
                if $context.byte > 0x1F && $context.byte < 0x7F {
                    // space + visible
                    continue;
                }

                break;
            },
            $on_eos
        );
    });
}

/// Collect and convert 2 hex bytes into a u8 variable.
///
/// This macro assumes that 2 bytes are available for reading. Return `$error` upon locating a
/// non-hex byte.
macro_rules! collect_hex8 {
    ($context:expr, $error:expr) => ({
        bs_next!($context);

        (
            if is_digit!($context.byte) {
                ($context.byte - b'0') << 4
            } else if b'@' < $context.byte && $context.byte < b'G' {
                ($context.byte - 0x37) << 4
            } else if b'`' < $context.byte && $context.byte < b'g' {
                ($context.byte - 0x57) << 4
            } else {
                return Err($error($context.byte));
            } as u8
        )
        +
        {
            bs_next!($context);

            (
                if is_digit!($context.byte) {
                    $context.byte - b'0'
                } else if b'@' < $context.byte && $context.byte < b'G' {
                    $context.byte - 0x37
                } else if b'`' < $context.byte && $context.byte < b'g' {
                    $context.byte - 0x57
                } else {
                    return Err($error($context.byte));
                } as u8
            )
        }
    });
}

/// Collect and convert 2 hex bytes into a u8 variable.
///
/// This macro is compatible with custom iterators.
///
/// This macro assumes that 2 bytes are available for reading. Return `$error` upon locating a
/// non-hex byte.
macro_rules! collect_hex8_iter {
    ($iter:expr, $context:expr, $error:expr) => ({
        bs_next!($context);

        (
            if is_digit!($context.byte) {
                ($context.byte - b'0') << 4
            } else if b'@' < $context.byte && $context.byte < b'G' {
                ($context.byte - 0x37) << 4
            } else if b'`' < $context.byte && $context.byte < b'g' {
                ($context.byte - 0x57) << 4
            } else {
                (*$iter.on_error)($error($context.byte));

                return None;
            } as u8
        )
        +
        {
            bs_next!($context);

            (
                if is_digit!($context.byte) {
                    $context.byte - b'0'
                } else if b'@' < $context.byte && $context.byte < b'G' {
                    $context.byte - 0x37
                } else if b'`' < $context.byte && $context.byte < b'g' {
                    $context.byte - 0x57
                } else {
                    (*$iter.on_error)($error($context.byte));

                    return None;
                } as u8
            )
        }
    });
}

/// Collect and convert all hex bytes into a u64 variable.
///
/// Exit the collection loop upon finding a non-hex byte. Return `$error` if an overflow would
/// occur.
macro_rules! collect_hex64 {
    ($context:expr, $error:expr, $digit:expr, $ty:ty, $on_eos:expr) => ({
        bs_collect_hex64!($context, $digit, {}, return Err($error), $on_eos, $ty);
    });
}

/// Collect a quoted value.
///
/// Exit the collection loop upon finding an unescaped double quote. Return `$error` upon finding a
/// non-visible 7-bit byte that also isn't a space, or when `$byte_error` is `true`.
macro_rules! collect_quoted {
    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_visible_7bit!($context.byte) || $context.byte == b' ' {
                if $context.byte == b'"' || $context.byte == b'\\' {
                    break;
                }
            } else {
                break;
            },
            $on_eos
        );
    });
}

/// Collect all token bytes.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_tokens {
    ($context:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_token($context.byte) {
                if $stop {
                    break;
                }
            } else {
                break;
            },
            $on_eos
        );
    });

    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_token($context.byte) {
                continue;
            } else {
                break;
            },
            $on_eos
        );
    });
}

/// Collect all visible 7-bit bytes. Visible bytes are 0x21 thru 0x7E.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_visible {
    ($context:expr, $stop:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_visible_7bit!($context.byte) {
                if $stop {
                    break;
                }
            } else {
                break;
            },
            $on_eos
        );
    });

    ($context:expr, $on_eos:expr) => ({
        bs_collect!($context,
            if is_visible_7bit!($context.byte) {
                continue;
            } else {
                break;
            },
            $on_eos
        );
    });
}

/// Consume all empty space.
///
/// Exit the collection loop when a non-space byte is found.
macro_rules! consume_empty_space {
    ($context:expr, $on_eos:expr) => ({
        if bs_is_eos!($context) {
            $on_eos
        }

        if bs_starts_with1!($context, b"\r") || bs_starts_with1!($context, b"\n")
        || bs_starts_with1!($context, b" ") || bs_starts_with1!($context, b"\t") {
            loop {
                if bs_is_eos!($context) {
                    $on_eos
                }

                bs_next!($context);

                if $context.byte == b'\r' || $context.byte == b'\n'
                || $context.byte == b' ' || $context.byte == b'\t' {
                    continue;
                } else {
                    bs_replay!($context);

                    break;
                }
            }
        }
    });
}

/// Consume all linear white space bytes.
///
/// Exit the collection loop when a non-linear white space byte is found.
macro_rules! consume_linear_space {
    ($context:expr, $on_eos:expr) => ({
        if bs_is_eos!($context) {
            $on_eos
        }

        if bs_starts_with1!($context, b" ") || bs_starts_with1!($context, b"\t") {
            loop {
                if bs_is_eos!($context) {
                    $on_eos
                }

                bs_next!($context);

                if $context.byte == b' ' || $context.byte == b'\t' {
                    continue;
                } else {
                    break;
                }
            }
        } else {
            bs_next!($context);
        }
    });
}

/// Consume all space bytes.
///
/// Exit the collection loop when a non-space byte is found.
macro_rules! consume_spaces {
    ($context:expr, $on_eos:expr) => ({
        if bs_is_eos!($context) {
            $on_eos
        }

        if bs_starts_with1!($context, b" ") {
            loop {
                if bs_is_eos!($context) {
                    $on_eos
                }

                bs_next!($context);

                if $context.byte == b' ' {
                    continue;
                } else {
                    break;
                }
            }
        } else {
            bs_next!($context);
        }
    });
}
