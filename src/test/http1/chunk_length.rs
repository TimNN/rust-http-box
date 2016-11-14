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
    () => ({
        let mut parser = Parser::new_chunked(DebugHandler::new());

        parser
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_hex(b"", |byte| {
        let mut p = setup!();

        assert_error_byte!(p,
                           &[byte],
                           ChunkLength,
                           byte);
    });

    // valid bytes
    loop_hex(b"0", |byte| {
        let mut p = setup!();

        assert_eos!(p,
                    &[byte],
                    ChunkLength2);
    });

    // starting 0 (end chunk)
    let mut p = setup!();

    assert_eos!(p,
                b"0",
                ChunkLengthCr);
}

#[test]
fn callback_exit() {
    struct CallbackHandler;

    impl HttpHandler for CallbackHandler {
        fn on_chunk_length(&mut self, _length: usize) -> bool {
            false
        }
    }

    let mut p = Parser::new_chunked(CallbackHandler);

    assert_callback!(p,
                     b"F\r",
                     ChunkLengthLf);
}

#[test]
fn missing_length() {
    let mut p = setup!();

    assert_error_byte!(p,
                       b"\r",
                       ChunkLength,
                       b'\r');
}

#[test]
fn length1() {
    let mut p = setup!();

    assert_eos!(p,
                b"F\r",
                ChunkLengthLf);

    assert_eq!(p.handler().chunk_length,
               15);
}

#[test]
fn length2() {
    let mut p = setup!();

    assert_eos!(p,
                b"FF\r",
                ChunkLengthLf);

    assert_eq!(p.handler().chunk_length,
               255);
}

#[test]
fn length3() {
    let mut p = setup!();

    assert_eos!(p,
                b"FFF\r",
                ChunkLengthLf);

    assert_eq!(p.handler().chunk_length,
               4095);
}

#[test]
fn too_long() {
    let mut p = setup!();

    assert_error!(p,
                  b"FFFFFFFFFFFFFFFF0",
                  MaxChunkLength);
}
