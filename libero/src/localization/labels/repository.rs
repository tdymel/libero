/// A `Repository`'s name: the host and repo, plus the star count once one shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct RepositoryLabels {
    /// The count as heard after the host and repo: the exact count, for the
    /// plural, and the shortened one on screen (`1.2k`). A fn, for plural forms.
    ///
    /// ```
    /// use libero::localization::RepositoryLabels;
    ///
    /// assert_eq!((RepositoryLabels::ENGLISH.stars)(1_234, "1.2k"), "1.2k stars");
    /// assert_eq!((RepositoryLabels::GERMAN.stars)(1, "1"), "1 Stern");
    /// ```
    pub stars: fn(u64, &str) -> String,
    /// The count shortened for the button, given the decimal separator from
    /// `Formats`. Rounds down, so it never shows more than it is.
    ///
    /// ```
    /// use libero::localization::RepositoryLabels;
    ///
    /// assert_eq!((RepositoryLabels::ENGLISH.compact)(1_234, "."), "1.2k");
    /// assert_eq!((RepositoryLabels::GERMAN.compact)(1_234, ","), "1,2\u{a0}Tsd.");
    /// ```
    pub compact: fn(u64, &str) -> String,
}

/// `count` in `unit`s, one decimal below ten of them: `1.2k`, `12k`.
fn scaled(count: u64, unit: u64, separator: &str, suffix: &str) -> String {
    let tenths = count * 10 / unit;
    if tenths >= 100 || tenths.is_multiple_of(10) {
        format!("{}{suffix}", tenths / 10)
    } else {
        format!("{}{separator}{}{suffix}", tenths / 10, tenths % 10)
    }
}

/// `RepositoryLabels::ENGLISH.compact`: `999`, `1.2k`, `12k`, `1.2M`.
fn english_compact(count: u64, separator: &str) -> String {
    match count {
        0..1_000 => count.to_string(),
        1_000..1_000_000 => scaled(count, 1_000, separator, "k"),
        _ => scaled(count, 1_000_000, separator, "M"),
    }
}

/// `RepositoryLabels::GERMAN.compact`: `999`, `1,2 Tsd.`, `1,2 Mio.`.
fn german_compact(count: u64, separator: &str) -> String {
    match count {
        0..1_000 => count.to_string(),
        1_000..1_000_000 => scaled(count, 1_000, separator, "\u{a0}Tsd."),
        _ => scaled(count, 1_000_000, separator, "\u{a0}Mio."),
    }
}

/// `RepositoryLabels::ENGLISH.stars`. A named fn, so every copy compares equal.
fn english_stars(count: u64, shown: &str) -> String {
    match count {
        1 => format!("{shown} star"),
        _ => format!("{shown} stars"),
    }
}

/// `RepositoryLabels::GERMAN.stars`.
fn german_stars(count: u64, shown: &str) -> String {
    match count {
        1 => format!("{shown} Stern"),
        _ => format!("{shown} Sterne"),
    }
}

impl RepositoryLabels {
    pub const ENGLISH: Self = Self {
        stars: english_stars,
        compact: english_compact,
    };

    pub const GERMAN: Self = Self {
        stars: german_stars,
        compact: german_compact,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_steps_through_units() {
        let english = RepositoryLabels::ENGLISH.compact;
        let cases = [
            (0, "0"),
            (999, "999"),
            (1_000, "1k"),
            (1_234, "1.2k"),
            (9_999, "9.9k"),
            (12_345, "12k"),
            (999_999, "999k"),
            (1_250_000, "1.2M"),
        ];
        for (count, expected) in cases {
            assert_eq!(english(count, "."), expected, "{count}");
        }
        assert_eq!(english(1_234, ","), "1,2k");
    }

    #[test]
    fn german_compact_names_its_own_units() {
        let german = RepositoryLabels::GERMAN.compact;
        assert_eq!(german(999, ","), "999");
        assert_eq!(german(1_234, ","), "1,2\u{a0}Tsd.");
        assert_eq!(german(1_250_000, ","), "1,2\u{a0}Mio.");
    }
}
