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

#[test]
fn byte_check() {
    // invalid bytes
    loop_non_tokens(b" \t", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        if let ParserError::Method(x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_tokens(b"", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        assert_eos(&mut p, &mut h, &[byte], State::RequestMethod, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_method(&mut self, _method: &[u8]) -> bool {
            false
        }
    }

    let mut h = X{};
    let mut p = Parser::new_request();

    assert_callback(&mut p, &mut h, b"G", State::RequestMethod, 1);
}

#[test]
fn multiple_connect() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"C", State::RequestMethod, 1);
    assert_eq!(h.method, b"C");

    assert_eos(&mut p, &mut h, b"O", State::RequestMethod, 1);
    assert_eq!(h.method, b"CO");

    assert_eos(&mut p, &mut h, b"N", State::RequestMethod, 1);
    assert_eq!(h.method, b"CON");

    assert_eos(&mut p, &mut h, b"N", State::RequestMethod, 1);
    assert_eq!(h.method, b"CONN");

    assert_eos(&mut p, &mut h, b"E", State::RequestMethod, 1);
    assert_eq!(h.method, b"CONNE");

    assert_eos(&mut p, &mut h, b"C", State::RequestMethod, 1);
    assert_eq!(h.method, b"CONNEC");

    assert_eos(&mut p, &mut h, b"T", State::RequestMethod, 1);
    assert_eq!(h.method, b"CONNECT");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"CONNECT");
}

#[test]
fn multiple_delete() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"D", State::RequestMethod, 1);
    assert_eq!(h.method, b"D");

    assert_eos(&mut p, &mut h, b"E", State::RequestMethod, 1);
    assert_eq!(h.method, b"DE");

    assert_eos(&mut p, &mut h, b"L", State::RequestMethod, 1);
    assert_eq!(h.method, b"DEL");

    assert_eos(&mut p, &mut h, b"E", State::RequestMethod, 1);
    assert_eq!(h.method, b"DELE");

    assert_eos(&mut p, &mut h, b"T", State::RequestMethod, 1);
    assert_eq!(h.method, b"DELET");

    assert_eos(&mut p, &mut h, b"E", State::RequestMethod, 1);
    assert_eq!(h.method, b"DELETE");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"DELETE");
}

#[test]
fn multiple_get() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"G", State::RequestMethod, 1);
    assert_eq!(h.method, b"G");

    assert_eos(&mut p, &mut h, b"E", State::RequestMethod, 1);
    assert_eq!(h.method, b"GE");

    assert_eos(&mut p, &mut h, b"T", State::RequestMethod, 1);
    assert_eq!(h.method, b"GET");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"GET");
}

#[test]
fn multiple_head() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"H", State::RequestMethod, 1);
    assert_eq!(h.method, b"H");

    assert_eos(&mut p, &mut h, b"E", State::RequestMethod, 1);
    assert_eq!(h.method, b"HE");

    assert_eos(&mut p, &mut h, b"A", State::RequestMethod, 1);
    assert_eq!(h.method, b"HEA");

    assert_eos(&mut p, &mut h, b"D", State::RequestMethod, 1);
    assert_eq!(h.method, b"HEAD");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"HEAD");
}

#[test]
fn multiple_options() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"O", State::RequestMethod, 1);
    assert_eq!(h.method, b"O");

    assert_eos(&mut p, &mut h, b"P", State::RequestMethod, 1);
    assert_eq!(h.method, b"OP");

    assert_eos(&mut p, &mut h, b"T", State::RequestMethod, 1);
    assert_eq!(h.method, b"OPT");

    assert_eos(&mut p, &mut h, b"I", State::RequestMethod, 1);
    assert_eq!(h.method, b"OPTI");

    assert_eos(&mut p, &mut h, b"O", State::RequestMethod, 1);
    assert_eq!(h.method, b"OPTIO");

    assert_eos(&mut p, &mut h, b"N", State::RequestMethod, 1);
    assert_eq!(h.method, b"OPTION");

    assert_eos(&mut p, &mut h, b"S", State::RequestMethod, 1);
    assert_eq!(h.method, b"OPTIONS");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"OPTIONS");
}

#[test]
fn multiple_post() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"P", State::RequestMethod, 1);
    assert_eq!(h.method, b"P");

    assert_eos(&mut p, &mut h, b"O", State::RequestMethod, 1);
    assert_eq!(h.method, b"PO");

    assert_eos(&mut p, &mut h, b"S", State::RequestMethod, 1);
    assert_eq!(h.method, b"POS");

    assert_eos(&mut p, &mut h, b"T", State::RequestMethod, 1);
    assert_eq!(h.method, b"POST");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"POST");
}

#[test]
fn multiple_put() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"P", State::RequestMethod, 1);
    assert_eq!(h.method, b"P");

    assert_eos(&mut p, &mut h, b"U", State::RequestMethod, 1);
    assert_eq!(h.method, b"PU");

    assert_eos(&mut p, &mut h, b"T", State::RequestMethod, 1);
    assert_eq!(h.method, b"PUT");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"PUT");
}

#[test]
fn multiple_trace() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"T", State::RequestMethod, 1);
    assert_eq!(h.method, b"T");

    assert_eos(&mut p, &mut h, b"R", State::RequestMethod, 1);
    assert_eq!(h.method, b"TR");

    assert_eos(&mut p, &mut h, b"A", State::RequestMethod, 1);
    assert_eq!(h.method, b"TRA");

    assert_eos(&mut p, &mut h, b"C", State::RequestMethod, 1);
    assert_eq!(h.method, b"TRAC");

    assert_eos(&mut p, &mut h, b"E", State::RequestMethod, 1);
    assert_eq!(h.method, b"TRACE");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"TRACE");
}

#[test]
fn multiple_unknown() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"U", State::RequestMethod, 1);
    assert_eq!(h.method, b"U");

    assert_eos(&mut p, &mut h, b"N", State::RequestMethod, 1);
    assert_eq!(h.method, b"UN");

    assert_eos(&mut p, &mut h, b"K", State::RequestMethod, 1);
    assert_eq!(h.method, b"UNK");

    assert_eos(&mut p, &mut h, b"N", State::RequestMethod, 1);
    assert_eq!(h.method, b"UNKN");

    assert_eos(&mut p, &mut h, b"O", State::RequestMethod, 1);
    assert_eq!(h.method, b"UNKNO");

    assert_eos(&mut p, &mut h, b"W", State::RequestMethod, 1);
    assert_eq!(h.method, b"UNKNOW");

    assert_eos(&mut p, &mut h, b"N", State::RequestMethod, 1);
    assert_eq!(h.method, b"UNKNOWN");

    assert_eos(&mut p, &mut h, b" ", State::StripRequestUrl, 1);
    assert_eq!(h.method, b"UNKNOWN");
}

#[test]
fn single_connect() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"CONNECT ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"CONNECT");
}

#[test]
fn single_delete() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"DELETE  ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"DELETE");
}

#[test]
fn single_get() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"GET     ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"GET");
}

#[test]
fn single_head() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"HEAD    ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"HEAD");
}

#[test]
fn single_options() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"OPTIONS ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"OPTIONS");
}

#[test]
fn single_post() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"POST    ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"POST");
}

#[test]
fn single_put() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"PUT     ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"PUT");
}

#[test]
fn single_trace() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"TRACE   ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"TRACE");
}

#[test]
fn single_unknown() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"UNKNOWN ", State::StripRequestUrl, 8);
    assert_eq!(h.method, b"UNKNOWN");
}

#[test]
fn starting_space() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    assert_eos(&mut p, &mut h, b"   ", State::StripRequestMethod, 3);
}
