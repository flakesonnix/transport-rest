//! Internal utilities: URL encoding and tolerant JSON deserialization.

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// Characters that must be escaped inside a single path segment.
const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'/');

/// Percent-encode a value so it can be used as a single path segment.
///
/// Trip IDs and refresh tokens regularly contain `/`, `:` and other
/// characters that must not appear unescaped in a path segment.
pub(crate) fn encode_path_segment(value: &str) -> String {
    utf8_percent_encode(value, PATH_SEGMENT).to_string()
}
