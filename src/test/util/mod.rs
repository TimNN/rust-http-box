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

#[macro_export]
macro_rules! field {
    ($map:expr, $stream:expr, $length:expr) => ({
        let fun = |s: FieldSegment| {
                      match s {
                          FieldSegment::Name(x) => {
                              let mut n = String::new();
                              let v = String::new();

                              unsafe {
                                  n.as_mut_vec().extend_from_slice(x);
                              }

                              $map.insert(n, v);
                          },
                          FieldSegment::NameValue(x,y) => {
                              let mut n = String::new();
                              let mut v = String::new();

                              unsafe {
                                  n.as_mut_vec().extend_from_slice(x);
                                  v.as_mut_vec().extend_from_slice(y);
                              }

                              $map.insert(n, v);
                          }
                      }
                      true
                 };

        assert!(match parse_field($stream, b';', true, fun) {
            Ok($length) => {
                true
            },
            _ => false
        });
    });
}

#[macro_export]
macro_rules! field_error {
    ($stream:expr, $byte:expr, $error:path) => ({
        let fun = |_: FieldSegment| { true };

        assert!(match parse_field($stream, b';', true, fun) {
            Err($error(x)) => {
                assert_eq!(x, $byte);
                true
            },
            _ => false
        });
    });
}

#[macro_export]
macro_rules! query {
    ($map:expr, $stream:expr, $length:expr) => ({
        let fun = |s: QuerySegment| {
            match s {
                QuerySegment::Name(x) => {
                    let mut f = String::new();
                    let v = String::new();

                    unsafe {
                        f.as_mut_vec().extend_from_slice(x);
                    }

                    $map.insert(f, v);
                },
                QuerySegment::NameValue(x,y) => {
                    let mut f = String::new();
                    let mut v = String::new();

                    unsafe {
                        f.as_mut_vec().extend_from_slice(x);
                        v.as_mut_vec().extend_from_slice(y);
                    }

                    $map.insert(f, v);
                }
            }

            true
        };

        assert!(match parse_query($stream, fun) {
            Ok($length) => {
                true
            },
            _ => false
        });
    });
}

#[macro_export]
macro_rules! query_error {
    ($stream:expr, $byte:expr, $error:path) => ({
        let fun = |_: QuerySegment| { true };

        assert!(match parse_query($stream, fun) {
            Err($error(x)) => {
                assert_eq!(x, $byte);
                true
            },
            _ => false
        });
    });
}

mod decode;
mod parse_field;
mod parse_query_field;
mod parse_query_value;
