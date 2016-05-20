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

use handler::*;
use http1::*;
use test::*;
use test::http1::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        $handler.set_transfer_encoding(TransferEncoding::Chunked);

        setup(&mut $parser, &mut $handler, b"GET / HTTP/1.1\r\n\r\nF;extension1=value1\r\n",
              State::ChunkData);
    });
}

#[test]
fn byte_check() {
    for byte in 0..255 {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eof(&mut p, &mut h, &[byte], State::ChunkData, 1);
    }
}

#[test]
fn multiple() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"abcdefg", State::ChunkData, 7);
    assert_eq!(h.chunk_data, b"abcdefg");
    assert_eof(&mut p, &mut h, b"hijklmno", State::ChunkDataNewline1, 8);
    assert_eq!(h.chunk_data, b"abcdefghijklmno");
}

#[test]
fn multiple_chunks() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"abcdefghijklmno\r\n", State::ChunkSize, 17);
    assert_eq!(h.chunk_data, b"abcdefghijklmno");
    assert_eof(&mut p, &mut h, b"5\r\n", State::ChunkData, 3);
    assert_eof(&mut p, &mut h, b"pqrst", State::ChunkDataNewline1, 5);
    assert_eq!(h.chunk_data, b"abcdefghijklmnopqrst");
}

#[test]
fn single() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"abcdefghijklmno", State::ChunkDataNewline1, 15);
    assert_eq!(h.chunk_data, b"abcdefghijklmno");
}
