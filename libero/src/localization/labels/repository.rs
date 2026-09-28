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
    };

    pub const GERMAN: Self = Self {
        stars: german_stars,
    };
}
