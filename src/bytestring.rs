// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::hash;
use core::ops;
use std::fmt;
use std::string::FromUtf8Error;

/// Represents a borrowed string value that is not necessarily UTF-8 encoded,
/// e.g. an HTTP header value.
pub struct ByteStr {
    bytes: [u8],
}

impl ByteStr {
    #[inline]
    fn from_bytes(slice: &[u8]) -> &ByteStr {
        unsafe { &*(slice as *const [u8] as *const ByteStr) }
    }

    #[inline]
    fn from_bytes_mut(slice: &mut [u8]) -> &mut ByteStr {
        unsafe { &mut *(slice as *mut [u8] as *mut ByteStr) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl ops::Deref for ByteStr {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.bytes
    }
}

impl ops::DerefMut for ByteStr {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl AsRef<[u8]> for ByteStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsMut<[u8]> for ByteStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

// Implementation borrowed from https://github.com/tokio-rs/bytes/blob/master/src/fmt/debug.rs
impl fmt::Debug for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b\"")?;
        for &b in &self.bytes {
            // https://doc.rust-lang.org/reference/tokens.html#byte-escapes
            if b == b'\n' {
                write!(f, "\\n")?;
            } else if b == b'\r' {
                write!(f, "\\r")?;
            } else if b == b'\t' {
                write!(f, "\\t")?;
            } else if b == b'\\' || b == b'"' {
                write!(f, "\\{}", b as char)?;
            } else if b == b'\0' {
                write!(f, "\\0")?;
            // ASCII printable
            } else if (0x20..0x7f).contains(&b) {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{:02x}", b)?;
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}

impl fmt::Display for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&String::from_utf8_lossy(&self.bytes), f)
    }
}

impl ops::Index<usize> for ByteStr {
    type Output = u8;

    #[inline]
    fn index(&self, idx: usize) -> &u8 {
        &self.as_bytes()[idx]
    }
}

impl ops::Index<ops::RangeFull> for ByteStr {
    type Output = ByteStr;

    #[inline]
    fn index(&self, _: ops::RangeFull) -> &ByteStr {
        self
    }
}

impl ops::Index<ops::Range<usize>> for ByteStr {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: ops::Range<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.as_bytes()[r.start..r.end])
    }
}

impl ops::Index<ops::RangeInclusive<usize>> for ByteStr {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: ops::RangeInclusive<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.as_bytes()[*r.start()..=*r.end()])
    }
}

impl ops::Index<ops::RangeFrom<usize>> for ByteStr {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: ops::RangeFrom<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.as_bytes()[r.start..])
    }
}

impl ops::Index<ops::RangeTo<usize>> for ByteStr {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: ops::RangeTo<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.as_bytes()[..r.end])
    }
}

impl ops::Index<ops::RangeToInclusive<usize>> for ByteStr {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: ops::RangeToInclusive<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.as_bytes()[..=r.end])
    }
}

impl ops::IndexMut<usize> for ByteStr {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.bytes[idx]
    }
}

impl ops::IndexMut<ops::RangeFull> for ByteStr {
    #[inline]
    fn index_mut(&mut self, _: ops::RangeFull) -> &mut ByteStr {
        self
    }
}

impl ops::IndexMut<ops::Range<usize>> for ByteStr {
    #[inline]
    fn index_mut(&mut self, r: ops::Range<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.bytes[r.start..r.end])
    }
}

impl ops::IndexMut<ops::RangeInclusive<usize>> for ByteStr {
    #[inline]
    fn index_mut(&mut self, r: ops::RangeInclusive<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.bytes[*r.start()..=*r.end()])
    }
}

impl ops::IndexMut<ops::RangeFrom<usize>> for ByteStr {
    #[inline]
    fn index_mut(&mut self, r: ops::RangeFrom<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.bytes[r.start..])
    }
}

impl ops::IndexMut<ops::RangeTo<usize>> for ByteStr {
    #[inline]
    fn index_mut(&mut self, r: ops::RangeTo<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.bytes[..r.end])
    }
}

impl ops::IndexMut<ops::RangeToInclusive<usize>> for ByteStr {
    #[inline]
    fn index_mut(&mut self, r: ops::RangeToInclusive<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.bytes[..=r.end])
    }
}

impl Eq for ByteStr {}

impl PartialEq<ByteStr> for ByteStr {
    #[inline]
    fn eq(&self, other: &ByteStr) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<String> for ByteStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<Vec<u8>> for ByteStr {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_bytes() == other.as_slice()
    }
}

impl PartialEq<str> for ByteStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for ByteStr {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<[u8]> for ByteStr {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_bytes() == other
    }
}

impl PartialEq<&[u8]> for ByteStr {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_bytes() == *other
    }
}

impl PartialEq<ByteStr> for String {
    #[inline]
    fn eq(&self, other: &ByteStr) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteStr> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &ByteStr) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteStr> for str {
    #[inline]
    fn eq(&self, other: &ByteStr) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteStr> for &str {
    #[inline]
    fn eq(&self, other: &ByteStr) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteStr> for &[u8] {
    #[inline]
    fn eq(&self, other: &ByteStr) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteStr> for [u8] {
    #[inline]
    fn eq(&self, other: &ByteStr) -> bool {
        *other == *self
    }
}

impl hash::Hash for ByteStr {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_bytes().hash(state);
    }
}

/// Represents a string value that is not necessarily UTF-8 encoded,
/// e.g. an HTTP header value.
#[derive(Default, Eq, Clone)]
pub struct ByteString {
    bytes: Vec<u8>,
}

impl ByteString {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn into_string(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.bytes)
    }
}

impl ops::Deref for ByteString {
    type Target = ByteStr;

    #[inline]
    fn deref(&self) -> &ByteStr {
        ByteStr::from_bytes(&self.bytes)
    }
}

impl ops::DerefMut for ByteString {
    #[inline]
    fn deref_mut(&mut self) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.bytes)
    }
}

impl AsRef<[u8]> for ByteString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsMut<[u8]> for ByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl fmt::Display for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl From<Vec<u8>> for ByteString {
    #[inline]
    fn from(bytes: Vec<u8>) -> Self {
        ByteString { bytes }
    }
}

impl From<&[u8]> for ByteString {
    #[inline]
    fn from(bytes: &[u8]) -> Self {
        bytes.to_owned().into()
    }
}

impl From<String> for ByteString {
    #[inline]
    fn from(text: String) -> Self {
        text.into_bytes().into()
    }
}

impl From<&str> for ByteString {
    #[inline]
    fn from(text: &str) -> Self {
        text.to_owned().into()
    }
}

impl From<&ByteString> for ByteString {
    #[inline]
    fn from(data: &ByteString) -> Self {
        data.clone()
    }
}

impl PartialEq for ByteString {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
        self.bytes == other.bytes
    }
}

impl PartialEq<String> for ByteString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other.as_bytes()
    }
}

impl PartialEq<Vec<u8>> for ByteString {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_ref() == other.as_slice()
    }
}

impl PartialEq<str> for ByteString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other.as_bytes()
    }
}

impl PartialEq<&str> for ByteString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == other.as_bytes()
    }
}

impl PartialEq<[u8]> for ByteString {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<&[u8]> for ByteString {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<ByteString> for String {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteString> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteString> for str {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteString> for &str {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteString> for [u8] {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
        *other == *self
    }
}

impl PartialEq<ByteString> for &[u8] {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
        *other == *self
    }
}

impl hash::Hash for ByteString {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        (&**self).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn test_bytestring_bstr_utf8() {
        use bstr::ByteSlice;

        let string: ByteString = "hello".into();
        assert_eq!(string.is_utf8(), true);
        assert_eq!(string.starts_with_str("hel"), true);
        assert_eq!(string.ends_with_str("lo"), true);
    }

    #[test]
    fn test_bytestring_bstr_bytes() {
        use bstr::ByteSlice;

        let bytes: ByteString = vec![144u8, 145u8, 146u8].into();
        assert_eq!(bytes.is_utf8(), false);
        assert_eq!(bytes.starts_with_str(b"\x90"), true);
        assert_eq!(bytes.ends_with_str(b"\x92"), true);
    }

    #[test]
    fn test_bytestring_display_utf8() {
        let string: ByteString = "utf-8 encoded string".into();

        assert_eq!(format!("{}", string), "utf-8 encoded string");
    }

    #[test]
    fn test_bytestring_debug_utf8() {
        let string: ByteString = "utf-8 encoded string".into();

        assert_eq!(format!("{:?}", string), "b\"utf-8 encoded string\"");
    }

    #[test]
    fn test_bytestring_display_bytes() {
        let bytes: ByteString = vec![144u8, 145u8, 146u8].into();

        assert_eq!(format!("{}", bytes), "���");
    }

    #[test]
    fn test_bytestring_debug_bytes() {
        let bytes: ByteString = vec![144u8, 145u8, 146u8].into();

        assert_eq!(format!("{:?}", bytes), "b\"\\x90\\x91\\x92\"");
    }

    #[test]
    fn test_bytestring_as_ref() {
        fn receive<T>(value: T)
        where
            T: AsRef<[u8]>,
        {
            value.as_ref();
        }

        let string: ByteString = "utf-8 encoded string".into();
        receive(string);

        let bytes: ByteString = vec![144u8, 145u8, 146u8].into();
        receive(bytes);
    }

    #[test]
    fn test_bytestring_substr_eq_utf8() {
        let string: ByteString = "hello".into();

        assert_eq!(string[1..=3], "ell");
        assert_eq!(string[1..=3], "ell".to_owned());

        assert_eq!("ell", string[1..=3]);
        assert_eq!("ell".to_owned(), string[1..=3]);
    }

    #[test]
    fn test_bytestring_substr_eq_bytes() {
        let bytes: ByteString = vec![144u8, 145u8, 146u8].into();

        assert_eq!(bytes[1..2], b"\x91" as &[u8]);
        assert_eq!(bytes[1..2], b"\x91".to_vec());

        assert_eq!(b"\x91" as &[u8], bytes[1..2]);
        assert_eq!(b"\x91".to_vec(), bytes[1..2]);
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn test_bytestring_eq_string() {
        let string: ByteString = "utf-8 encoded string".into();

        assert_eq!(string, "utf-8 encoded string");
        assert_eq!(string, b"utf-8 encoded string" as &[u8]);

        assert_eq!("utf-8 encoded string", string);
        assert_eq!(b"utf-8 encoded string" as &[u8], string);

        assert_eq!(string, string);
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn test_bytestring_eq_bytes() {
        let bytes: ByteString = vec![144u8, 145u8, 146u8].into();

        assert_eq!(bytes, vec![144u8, 145u8, 146u8]);
        assert_eq!(bytes, b"\x90\x91\x92" as &[u8]);

        assert_eq!(vec![144u8, 145u8, 146u8], bytes);
        assert_eq!(b"\x90\x91\x92" as &[u8], bytes);

        assert_eq!(bytes, bytes);
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        h.finish()
    }

    #[test]
    fn test_bytestring_hash_string() {
        let string: ByteString = "utf-8 encoded string".into();

        assert_ne!(hash(&string), hash(&"utf-8 encoded string"));
        assert_eq!(hash(&string), hash(&b"utf-8 encoded string"));

        assert_ne!(hash(&"utf-8 encoded string"), hash(&string));
        assert_eq!(hash(&b"utf-8 encoded string"), hash(&string));
    }

    #[test]
    fn test_bytestring_hash_bytes() {
        let bytes: ByteString = vec![144u8, 145u8, 146u8].into();

        assert_eq!(hash(&bytes), hash(&vec![144u8, 145u8, 146u8]));
        assert_eq!(hash(&bytes), hash(&[144u8, 145u8, 146u8]));

        assert_eq!(hash(&vec![144u8, 145u8, 146u8]), hash(&bytes));
        assert_eq!(hash(&[144u8, 145u8, 146u8]), hash(&bytes));
    }
}
