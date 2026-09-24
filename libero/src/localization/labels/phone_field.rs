/// `PhoneField`'s country picker. Its search placeholder is
/// [`CommonLabels::search`](super::CommonLabels::search).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhoneFieldLabels {
    /// Names the country search box.
    pub search: &'static str,
    /// Names the country button: `{name}`, `{iso}` and `{dial}`.
    pub country: &'static str,
    /// Country names by upper-case ISO code; a code missing here keeps its
    /// English name. `country_label` on the field wins over both.
    ///
    /// ```
    /// use libero::localization::PhoneFieldLabels;
    ///
    /// static LABELS: PhoneFieldLabels = PhoneFieldLabels {
    ///     country_names: &[("DE", "Deutschland"), ("AT", "Österreich")],
    ///     ..PhoneFieldLabels::GERMAN
    /// };
    /// ```
    pub country_names: &'static [(&'static str, &'static str)],
}

impl PhoneFieldLabels {
    pub const ENGLISH: Self = Self {
        search: "Search countries",
        country: "Country: {name}, {iso} +{dial}",
        country_names: &[],
    };

    pub const GERMAN: Self = Self {
        search: "Länder durchsuchen",
        country: "Land: {name}, {iso} +{dial}",
        country_names: crate::localization::country_names::GERMAN,
    };

    /// The name `country_names` gives `iso`, if any.
    pub(crate) fn country_name(&self, iso: &str) -> Option<&'static str> {
        self.country_names
            .iter()
            .find(|(code, _)| code.eq_ignore_ascii_case(iso))
            .map(|(_, name)| *name)
    }
}
