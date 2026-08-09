use std::fmt;

use super::ConstVec;

const DEFAULT_CONST_STR_CAPACITY: usize = 2usize.pow(12);

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct ConstStr<const MAX_SIZE: usize = DEFAULT_CONST_STR_CAPACITY> {
    bytes: ConstVec<u8, MAX_SIZE>,
}

impl<const MAX_SIZE: usize> Default for ConstStr<MAX_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_SIZE: usize> fmt::Debug for ConstStr<MAX_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ConstStr").field(&self.as_str()).finish()
    }
}

impl<const MAX_SIZE: usize> ConstStr<MAX_SIZE> {
    pub const fn new() -> Self {
        Self {
            bytes: ConstVec::new_with_max_size(),
        }
    }

    pub const fn push_str(mut self, value: &str) -> Self {
        self.bytes.extend(value.as_bytes());
        self
    }

    pub const fn push_char(mut self, value: char) -> Self {
        if !value.is_ascii() {
            panic!("ConstStr currently only supports ASCII chars");
        }
        self.bytes.push(value as u8);
        self
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    pub const fn as_str(&self) -> &str {
        match std::str::from_utf8(self.as_bytes()) {
            Ok(value) => value,
            Err(_) => panic!("ConstStr contained invalid UTF-8"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_str_happy_path() {
        const VALUE: ConstStr = ConstStr::new()
            .push_str(".button")
            .push_char('{')
            .push_str("color:red;")
            .push_char('}');

        assert_eq!(VALUE.as_str(), ".button{color:red;}");
    }
}
