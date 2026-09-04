use crate::theme::Size;

/// What `PhoneField` does not share with every other field. The frame's
/// numbers live on `FieldDefaults` and the picker's list on
/// `ComboboxDefaults`, so a phone field lines up with a `TextField` above it
/// and with a `Select`'s list below it by construction.
///
/// `country` is a real theme knob and not a per-field prop with a constant:
/// a German app sets its starting country once instead of on every field. The
/// country *names* deliberately stay out - 240 of them would dwarf the whole
/// theme, so they come from the component's table with `country_label` as the
/// live override.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhoneFieldDefaults {
    pub size: Size,
    pub radius: Size,
    /// ISO 3166-1 alpha-2. An unknown code falls back to the first country in
    /// the table rather than panicking.
    pub country: &'static str,
}

impl PhoneFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        country: "US",
    };
}
