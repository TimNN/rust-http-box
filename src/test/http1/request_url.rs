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
use url::*;

macro_rules! setup {
    ($parser:expr, $handler:expr) => ({
        setup(&mut $parser, &mut $handler, b"GET ", State::StripRequestUrl);
    });
}

#[test]
fn byte_check() {
    // invalid bytes
    loop_unsafe(b" \t", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        if let ParserError::Url(_,x) = assert_error(&mut p, &mut h, &[byte]).unwrap() {
            assert_eq!(x, byte);
        } else {
            panic!();
        }
    });

    // valid bytes
    loop_safe(b" ", |byte| {
        let mut h = DebugHandler::new();
        let mut p = Parser::new_request();

        setup!(p, h);

        assert_eof(&mut p, &mut h, &[byte], State::RequestUrl, 1);
    });
}

#[test]
fn callback_exit() {
    struct X;

    impl HttpHandler for X {
        fn on_url(&mut self, _url: &[u8]) -> bool {
            false
        }
    }

    impl ParamHandler for X {}

    let mut h = X{};
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_callback(&mut p, &mut h, b"/", State::RequestUrl, 1);
}

#[test]
fn with_schema() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"http://host.com:443/path?query_string#fragment ",
               State::StripRequestHttp, 47);
    vec_eq(h.url, b"http://host.com:443/path?query_string#fragment");
}

#[test]
fn without_schema() {
    let mut h = DebugHandler::new();
    let mut p = Parser::new_request();

    setup!(p, h);

    assert_eof(&mut p, &mut h, b"/path?query_string#fragment ",
               State::StripRequestHttp, 28);
    vec_eq(h.url, b"/path?query_string#fragment");
}
