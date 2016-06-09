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

//! Handler implementations.

use http1::Http1Handler;

use std::str;

pub struct DebugHttp1Handler {
    pub chunk_data:            Vec<u8>,
    pub chunk_extension_name:  Vec<u8>,
    pub chunk_extension_value: Vec<u8>,
    pub chunk_size:            u64,
    pub header_field:          Vec<u8>,
    pub header_value:          Vec<u8>,
    pub headers_finished:      bool,
    pub method:                Vec<u8>,
    pub multipart_data:        Vec<u8>,
    pub status:                Vec<u8>,
    pub status_code:           u16,
    pub url:                   Vec<u8>,
    pub url_encoded_field:     Vec<u8>,
    pub url_encoded_value:     Vec<u8>,
    pub version_major:         u16,
    pub version_minor:         u16
}

impl DebugHttp1Handler {
    pub fn new() -> DebugHttp1Handler {
        DebugHttp1Handler{ chunk_data:            Vec::new(),
                          chunk_extension_name:  Vec::new(),
                          chunk_extension_value: Vec::new(),
                          chunk_size:            0,
                          header_field:          Vec::new(),
                          header_value:          Vec::new(),
                          headers_finished:      false,
                          method:                Vec::new(),
                          multipart_data:        Vec::new(),
                          status:                Vec::new(),
                          status_code:           0,
                          url:                   Vec::new(),
                          url_encoded_field:     Vec::new(),
                          url_encoded_value:     Vec::new(),
                          version_major:         0,
                          version_minor:         0 }
    }

    pub fn reset(&mut self) {
        self.chunk_data            = Vec::new();
        self.chunk_extension_name  = Vec::new();
        self.chunk_extension_value = Vec::new();
        self.chunk_size            = 0;
        self.header_field          = Vec::new();
        self.header_value          = Vec::new();
        self.headers_finished      = false;
        self.method                = Vec::new();
        self.multipart_data        = Vec::new();
        self.status                = Vec::new();
        self.status_code           = 0;
        self.url                   = Vec::new();
        self.url_encoded_field     = Vec::new();
        self.url_encoded_value     = Vec::new();
        self.version_major         = 0;
        self.version_minor         = 0;
    }
}

impl Http1Handler for DebugHttp1Handler {
    fn on_chunk_data(&mut self, data: &[u8]) -> bool {
        self.chunk_data.extend_from_slice(data);

        for byte in data {
            if *byte > 127 {
                println!("on_chunk_data [{}]: *hidden*", data.len());
                return true;
            }
        }

        println!("on_chunk_data [{}]: {:?}", data.len(), str::from_utf8(data).unwrap());
        true
    }

    fn on_chunk_extension_name(&mut self, name: &[u8]) -> bool {
        println!("on_chunk_extension_name [{}]: {:?}", name.len(), str::from_utf8(name).unwrap());
        self.chunk_extension_name.extend_from_slice(name);
        true
    }

    fn on_chunk_extension_value(&mut self, value: &[u8]) -> bool {
        println!("on_chunk_extension_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        self.chunk_extension_value.extend_from_slice(value);
        true
    }

    fn on_chunk_size(&mut self, size: u64) -> bool {
        println!("on_chunk_size: {}", size);
        self.chunk_size = size;
        true
    }

    fn on_header_field(&mut self, field: &[u8]) -> bool {
        println!("on_header_field [{}]: {:?}", field.len(), str::from_utf8(field).unwrap());
        self.header_field.extend_from_slice(field);
        true
    }

    fn on_header_value(&mut self, value: &[u8]) -> bool {
        println!("on_header_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        self.header_value.extend_from_slice(value);
        true
    }

    fn on_headers_finished(&mut self) {
        println!("on_headers_finished");
        self.headers_finished = true;
    }

    fn on_method(&mut self, method: &[u8]) -> bool {
        println!("on_method [{}]: {:?}", method.len(), str::from_utf8(method).unwrap());
        self.method.extend_from_slice(method);
        true
    }

    fn on_multipart_data(&mut self, data: &[u8]) -> bool {
        println!("on_multipart_data [{}]: {:?}", data.len(), str::from_utf8(data).unwrap());
        self.multipart_data.extend_from_slice(data);
        true
    }

    fn on_status(&mut self, status: &[u8]) -> bool {
        println!("on_status [{}]: {:?}", status.len(), str::from_utf8(status).unwrap());
        self.status.extend_from_slice(status);
        true
    }

    fn on_status_code(&mut self, code: u16) -> bool {
        println!("on_status_code: {}", code);
        self.status_code = code;
        true
    }

    fn on_url(&mut self, url: &[u8]) -> bool {
        println!("on_url [{}]: {:?}", url.len(), str::from_utf8(url).unwrap());
        self.url.extend_from_slice(url);
        true
    }

    fn on_url_encoded_field(&mut self, field: &[u8]) -> bool {
        println!("on_url_encoded_field [{}]: {:?}", field.len(), str::from_utf8(field).unwrap());
        self.url_encoded_field.extend_from_slice(field);
        true
    }

    fn on_url_encoded_value(&mut self, value: &[u8]) -> bool {
        println!("on_url_encoded_value [{}]: {:?}", value.len(), str::from_utf8(value).unwrap());
        self.url_encoded_value.extend_from_slice(value);
        true
    }

    fn on_version(&mut self, major: u16, minor: u16) -> bool {
        println!("on_version: {}.{}", major, minor);
        self.version_major = major;
        self.version_minor = minor;
        true
    }
}
