/// Every string `Pagination` puts in front of a reader.
///
/// The `<nav>`'s own name is **not** here. `aria_label` is a required prop, so
/// the caller supplies it and localises it themselves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaginationLabels {
    /// A page the reader is not on. `{n}` is the page number.
    pub page: &'static str,
    /// The current page. `aria-current` already says "current", so repeating
    /// "go to" on the page you are on is a lie.
    pub current_page: &'static str,
    pub previous: &'static str,
    pub next: &'static str,
    pub first: &'static str,
    pub last: &'static str,
}

impl PaginationLabels {
    pub const ENGLISH: Self = Self {
        page: "Go to page {n}",
        current_page: "Page {n}",
        previous: "Go to previous page",
        next: "Go to next page",
        first: "Go to first page",
        last: "Go to last page",
    };

    pub const GERMAN: Self = Self {
        page: "Zu Seite {n}",
        current_page: "Seite {n}",
        previous: "Zur vorherigen Seite",
        next: "Zur nächsten Seite",
        first: "Zur ersten Seite",
        last: "Zur letzten Seite",
    };
}
