/// The zeros of the decimal digit runs a phone keyboard or IME types: Arabic-Indic,
/// Persian, the Indic scripts, Thai, Lao, Tibetan, Myanmar, Khmer, Mongolian, full width.
const ZEROS: &[u32] = &[
    0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6, 0x0D66,
    0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0xFF10,
];

/// A decimal digit of any of those scripts as its ASCII digit.
pub(crate) fn ascii_digit(c: char) -> Option<char> {
    if c.is_ascii_digit() {
        return Some(c);
    }
    let code = u32::from(c);
    ZEROS
        .iter()
        .find(|zero| (**zero..**zero + 10).contains(&code))
        .and_then(|zero| char::from_digit(code - zero, 10))
}

/// The character with a digit of another script as its ASCII digit.
pub(crate) fn fold_digit(c: char) -> char {
    ascii_digit(c).unwrap_or(c)
}

/// The text with every digit of another script as its ASCII digit.
pub(crate) fn fold_digits(text: &str) -> String {
    text.chars().map(fold_digit).collect()
}

/// Only the digits, as ASCII.
pub(crate) fn digits_of(text: &str) -> String {
    text.chars().filter_map(ascii_digit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_of_other_scripts_fold_to_ascii() {
        assert_eq!(fold_digits("\u{0661}\u{06F2}\u{FF13}x7"), "123x7");
        assert_eq!(digits_of("(\u{0966}\u{0967}) 2-\u{E53}"), "0123");
        assert_eq!(ascii_digit('a'), None);
    }
}
