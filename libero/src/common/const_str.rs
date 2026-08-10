use std::fmt;

use super::ConstVec;

const DEFAULT_CONST_STR_CAPACITY: usize = 2usize.pow(12);

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct ConstStr {
    bytes: ConstVec<u8, DEFAULT_CONST_STR_CAPACITY>,
}

impl Default for ConstStr {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ConstStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ConstStr").field(&self.as_str()).finish()
    }
}

impl ConstStr {
    pub const fn new() -> Self {
        Self {
            bytes: ConstVec::new_with_max_size(),
        }
    }

    pub const fn from_str(value: &str) -> Self {
        Self::new().push_str(value)
    }

    pub const fn append(mut self, value: ConstStr) -> Self {
        self.bytes.extend(value.as_bytes());
        self
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
        const VALUE: ConstStr = ConstStr::from_str(".button")
            .push_char('{')
            .push_str("color:red;")
            .push_char('}');

        assert_eq!(VALUE.as_str(), ".button{color:red;}");
    }
}
