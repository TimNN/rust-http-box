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

//! Stream collection macros.

/// Collect and convert all digit bytes into an integer variable.
///
/// Exit the collection loop upon finding a non-digit byte. Return `$error` if `$digit` exceeds
/// `$max`.
macro_rules! collect_digits {
    ($context:expr, $error:expr, $digit:expr, $max:expr, $eos:expr) => ({
        bs_collect_digits!($context, $digit,
            if $digit > $max {
                return Err($error($context.byte));
            },
            $eos
        );
    });
}

/// Collect `$length` bytes as long as `$allow` yields `true`. Otherwise return `$error`.
///
/// This macro assumes that `$length` bytes are available for reading.
///
/// End-of-stream returns `$error`.
macro_rules! collect_length {
    ($context:expr, $error:expr, $length:expr, $allow:expr) => ({
        bs_collect_length!($context, {
            if !$allow {
                return Err($error($context.byte));
            }
        }, {
            return Err($error($context.byte))
        });
    });
}

/// Collect all token bytes.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_tokens {
    ($context:expr, $error:expr, $eos:expr, $stop:expr) => ({
        bs_collect!($context,
            if $stop {
                break;
            } else if !is_token($context.byte) {
                return Err($error($context.byte));
            },
            $eos
        );
    });

    ($context:expr, $error:expr, $eos:expr) => ({
        bs_collect!($context,
            if !is_token($context.byte) {
                return Err($error($context.byte));
            },
            $eos
        );
    });
}

/// Collect all visible 7-bit bytes. Visible bytes are 0x21 thru 0x7E.
///
/// Exit the collection loop when `$stop` yields `true`.
macro_rules! collect_visible {
    ($context:expr, $error:expr, $eos:expr, $stop:expr) => ({
        bs_collect!($context,
            if $stop {
                break;
            } else if is_not_visible_7bit!($context.byte) {
                return Err($error($context.byte));
            },
            $eos
        );
    });

    ($context:expr, $error:expr, $eos:expr) => ({
        bs_collect!($context,
            if is_not_visible_7bit!($context.byte) {
                return Err($error($context.byte));
            },
            $eos
        );
    });
}
