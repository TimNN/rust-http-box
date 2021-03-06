extern crate http_box;

use http_box::fsm::Success;
use http_box::http1::{ HttpHandler,
                       Parser,
                       State };

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

struct HeadHandler;

impl HttpHandler for HeadHandler {
}

struct UrlEncodedHandler {
    pub name_buf:   Vec<u8>,
    pub parameters: HashMap<String, String>,
    pub state:      State,
    pub value_buf:  Vec<u8>
}

impl UrlEncodedHandler {
    pub fn new() -> UrlEncodedHandler {
        UrlEncodedHandler{
            name_buf:   Vec::new(),
            parameters: HashMap::new(),
            state:      State::None,
            value_buf:  Vec::new()
        }
    }

    fn flush_parameter(&mut self) {
        if self.name_buf.len() > 0 && self.value_buf.len() > 0 {
            self.parameters.insert(
                unsafe {
                    let mut s = String::with_capacity(self.name_buf.len());

                    s.as_mut_vec().extend_from_slice(&self.name_buf);
                    s
                },
                unsafe {
                    let mut s = String::with_capacity(self.value_buf.len());

                    s.as_mut_vec().extend_from_slice(&self.value_buf);
                    s
                }
            );
        }

        self.name_buf.clear();
        self.value_buf.clear();
    }
}

impl HttpHandler for UrlEncodedHandler {
    fn on_body_finished(&mut self) -> bool {
        self.flush_parameter();

        true
    }

    fn on_url_encoded_name(&mut self, name: &[u8]) -> bool {
        if self.state == State::UrlEncodedValue {
            self.flush_parameter();
        }

        self.name_buf.extend_from_slice(name);

        self.state = State::UrlEncodedName;
        true
    }

    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        self.value_buf.extend_from_slice(value);

        self.state = State::UrlEncodedValue;
        true
    }
}

#[test]
fn url_encoded() {
    let mut d = Vec::new();

    File::open("tests/http1_data/url_encoded.dat").unwrap().read_to_end(&mut d);

    let mut s  = d.as_slice();
    let mut hp = Parser::new();

    // parse head
    match hp.resume(&mut HeadHandler, &s) {
        Ok(Success::Finished(length)) => {
            // adjust the slice since we've parsed the head already
            s = &s[length..];
        },
        _ => panic!()
    }

    let mut uh = UrlEncodedHandler::new();
    let mut up = Parser::new();

    up.init_url_encoded();

    // set url encoded data length, but subtract 2 since the CRLF still need handled
    up.set_length(s.len() - 2);

    match up.resume(&mut uh, &s) {
        Ok(Success::Finished(_)) => {
        },
        _ => panic!()
    }

    assert_eq!(
        uh.parameters.get("first_name").unwrap(),
        "Ada"
    );

    assert_eq!(
        uh.parameters.get("last_name").unwrap(),
        "Lovelace"
    );

    assert_eq!(
        uh.parameters.get("age").unwrap(),
        "36"
    );

    assert_eq!(
        uh.parameters.get("gender").unwrap(),
        "Female"
    );
}
