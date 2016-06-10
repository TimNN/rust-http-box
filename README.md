# rust-http-box

rust-http-box is a fast push/callback oriented HTTP/1.1 (HTTP/2.0 coming soon) parser that works
only with slices of data, and never copies parsed data. Because of this, it is
possible to parse HTTP data one byte at a time. Parsing can be interrupted during any callback,
and at the end of each parsed chunk.

This is purely an HTTP parsing library and is not tied to any networking framework. Use it to parse
stored HTTP request logs, test data, or to write a server and/or client.

Errors are handled intelligently letting you know what state the parser was in and which byte
triggered the error when it occurred.

## Features

- Understands persistent requests
- Easily upgradable from HTTP/1.1 parsing to HTTP/2.0 in the same stream
- Parses:
  - Requests
  - Responses
  - Headers
  - Chunk encoded data
  - Query strings / URL encoded data
  - Multipart (in the works)

## Access To:

- Request:
  - Method
  - URL
  - Version
- Response:
  - Status
  - Status code
  - Version
- Headers (quoted and multi-line values are supported):
  - Field
  - Value
- Chunk encoded:
  - Size
  - Extension name
  - Extension value
  - Raw data
- Multipart (in the works)
  - Fields
  - Values
  - File support
- URL encoded:
  - Field
  - Value

## Performance

Currently rust-http-box is on par with the speeds seen from [fast-http](https://github.com/fukamachi/fast-http),
a Common Lisp HTTP parser.

## API Documentation

http://metatomic.org/docs/http_box/index.html
