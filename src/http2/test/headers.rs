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

use http2::{ Flags,
             FrameType,
             Parser,
             ParserState };

use http2::test::*;

#[test]
fn with_padding_without_priority() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (24 << 8) | 0x1
    );

    // frame frame flags
    pack_u8!(v, 0x8);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // pad length
    pack_u8!(
        v,
        10
    );

    // fragment
    pack_bytes!(
        v,
        b"Hello, world!"
    );

    // padding
    pack_bytes!(
        v,
        b"XXXXXXXXXX"
    );

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert!(Flags::from_u8(h.frame_flags).is_padded());

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::Headers
    );

    assert!(!h.headers_exclusive);

    assert_eq!(
        h.headers_stream_id,
        0
    );

    assert_eq!(
        h.headers_weight,
        0
    );

    assert_eq!(
        h.headers_data,
        b"Hello, world!"
    );

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}

#[test]
fn with_padding_with_priority() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (29 << 8) | 0x1
    );

    // frame frame flags
    pack_u8!(v, 0x8 | 0x20);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // pad length
    pack_u8!(
        v,
        10
    );

    // exclusive bit and stream id
    pack_u32!(
        v,
        0xFFFFFFFF
    );

    // weight
    pack_u8!(
        v,
        99
    );

    // fragment
    pack_bytes!(
        v,
        b"Hello, world!"
    );

    // padding
    pack_bytes!(
        v,
        b"XXXXXXXXXX"
    );

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert!(Flags::from_u8(h.frame_flags).is_padded());
    assert!(Flags::from_u8(h.frame_flags).is_priority());

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::Headers
    );

    assert!(h.headers_exclusive);

    assert_eq!(
        h.headers_stream_id,
        0x7FFFFFFF
    );

    assert_eq!(
        h.headers_weight,
        99
    );

    assert_eq!(
        h.headers_data,
        b"Hello, world!"
    );

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}

#[test]
fn without_padding_without_priority() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (13 << 8) | 0x1
    );

    // frame frame flags
    pack_u8!(v, 0);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // fragment
    pack_bytes!(
        v,
        b"Hello, world!"
    );

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert!(Flags::from_u8(h.frame_flags).is_empty());

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::Headers
    );

    assert!(!h.headers_exclusive);

    assert_eq!(
        h.headers_stream_id,
        0
    );

    assert_eq!(
        h.headers_weight,
        0
    );

    assert_eq!(
        h.headers_data,
        b"Hello, world!"
    );

    assert!(h.headers_data_finished);

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}

#[test]
fn without_padding_with_priority() {
    let mut v = Vec::new();

    // frame payload length and type
    pack_u32!(
        v,
        (18 << 8) | 0x1
    );

    // frame frame flags
    pack_u8!(v, 0x20);

    // frame reserved bit and stream id
    pack_u32!(v, 0x7FFFFFFF);

    // exclusive bit and stream id
    pack_u32!(
        v,
        0xFFFFFFFF
    );

    // weight
    pack_u8!(
        v,
        99
    );

    // fragment
    pack_bytes!(
        v,
        b"Hello, world!"
    );

    let mut h = DebugHandler::new();
    let mut p = Parser::new();

    p.resume(&mut h, &v);

    assert!(Flags::from_u8(h.frame_flags).is_priority());

    assert_eq!(
        FrameType::from_u8(h.frame_type),
        FrameType::Headers
    );

    assert!(h.headers_exclusive);

    assert_eq!(
        h.headers_stream_id,
        0x7FFFFFFF
    );

    assert_eq!(
        h.headers_weight,
        99
    );

    assert_eq!(
        h.headers_data,
        b"Hello, world!"
    );

    assert!(h.headers_data_finished);

    assert_eq!(
        p.state(),
        ParserState::FrameLength1
    );
}
