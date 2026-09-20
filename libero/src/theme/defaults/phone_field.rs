use crate::theme::Size;

/// Theme defaults for `PhoneField`, set on [`Theme`](crate::theme::Theme).
/// The frame lives on `FieldDefaults`, the list on `ComboboxDefaults`, names on `PhoneFieldLabels`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhoneFieldDefaults {
    pub size: Size,
    pub radius: Size,
    /// ISO 3166-1 alpha-2; an unknown code falls back to the table's first country.
    pub country: &'static str,
    /// Offers the country picker. Off pins `country` behind a static prefix.
    pub country_select: bool,
}

impl PhoneFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        country: "US",
        country_select: true,
    };
}
