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

/// The trimmed `text` without its `group` separators when they split the whole into threes,
/// as `12.500,5`; else `None`, so a German `1.5` from a numpad stays a decimal.
pub(crate) fn ungrouped(text: &str, group: &str) -> Option<String> {
    if group.is_empty() {
        return None;
    }
    let text = text.trim();
    let (sign, rest) = match text.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", text),
    };
    let parts: Vec<&str> = rest.split(group).collect();
    let digits = |part: &str| part.chars().take_while(char::is_ascii_digit).count();
    let (first, last) = (parts.first()?, parts.last()?);
    let grouped = parts.len() > 1
        && (1..=3).contains(&first.len())
        && digits(first) == first.len()
        && parts[1..parts.len() - 1]
            .iter()
            .all(|part| part.len() == 3 && digits(part) == 3)
        && digits(last) == 3;
    grouped.then(|| format!("{sign}{}", parts.concat()))
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

    #[test]
    fn plain_text_has_no_groups_to_drop() {
        assert_eq!(ungrouped("1234", ","), None);
        assert_eq!(ungrouped("12", "."), None);
        assert_eq!(ungrouped("1,000", ""), None);
    }

    #[test]
    fn groups_of_three_are_dropped() {
        assert_eq!(ungrouped("1,234", ",").as_deref(), Some("1234"));
        assert_eq!(
            ungrouped(" -12,345,678 ", ",").as_deref(),
            Some("-12345678")
        );
        assert_eq!(ungrouped("1\u{a0}234", "\u{a0}").as_deref(), Some("1234"));
    }

    #[test]
    fn the_decimal_separator_after_the_groups_stays() {
        assert_eq!(ungrouped("12.500,5", ".").as_deref(), Some("12500,5"));
        assert_eq!(ungrouped("1,234.75", ",").as_deref(), Some("1234.75"));
        assert_eq!(ungrouped("1.5", "."), None);
    }

    #[test]
    fn text_not_in_threes_is_no_grouping() {
        assert_eq!(ungrouped("1,23", ","), None);
        assert_eq!(ungrouped("1,2345", ","), None);
        assert_eq!(ungrouped("1234,567", ","), None);
        assert_eq!(ungrouped("1,23,456", ","), None);
        assert_eq!(ungrouped("a,123", ","), None);
        assert_eq!(ungrouped("", ","), None);
    }
}
