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

use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $parser.init_chunked();

        assert_eos!($parser, $handler,
                    b"F;",
                    ParserState::StripChunkExtensionName);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b"\r\t=; ", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_error_byte!(p, h,
                           &[b'a', byte],
                           ParserError::ChunkExtensionName,
                           byte);
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new();

        setup!(p, h);

        assert_eos!(p, h,
                    &[byte],
                    ParserState::LowerChunkExtensionName);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_chunk_extension_name(&mut self, _name: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new();

    setup!(p, h);

    // because chunk extension name is processed by 2 states, the callback exit will first
    // happen on the first byte
    assert_callback!(p, h,
                     b"ChunkExtension=",
                     ParserState::LowerChunkExtensionName,
                     1);
}

#[test]
fn normalize() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"CHANGE----LOWER",
                ParserState::LowerChunkExtensionName);

    assert_eq!(h.chunk_extension_name, b"change----lower");
}

#[test]
fn no_value() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"valid-extension;",
                ParserState::StripChunkExtensionName);
    assert_eq!(h.chunk_extension_name, b"valid-extension");
    assert_eq!(h.chunk_extension_value, b"");
}

#[test]
fn valid() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    setup!(p, h);

    assert_eos!(p, h,
                b"valid-extension=",
                ParserState::StripChunkExtensionValue);
    assert_eq!(h.chunk_extension_name, b"valid-extension");
}
