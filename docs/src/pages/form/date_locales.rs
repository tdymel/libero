//! The languages and formats libero ships, for the docs' two switches.

use libero::localization::{Formats, Localization};

/// A switch's option, the const's name and the const.
pub type Choice<T> = (&'static str, &'static str, &'static T);

/// The first is the site's language.
pub const LANGUAGES: [Choice<Localization>; 2] = [
    ("English", "ENGLISH", &Localization::ENGLISH),
    ("Deutsch", "GERMAN", &Localization::GERMAN),
];

/// The second is the site's formats; the first is libero's default.
pub const FORMATS: [Choice<Formats>; 2] = [
    ("American", "AMERICAN", &Formats::AMERICAN),
    ("German", "GERMAN", &Formats::GERMAN),
];

/// A switch's options.
pub fn options<T: 'static>(choices: &[Choice<T>; 2]) -> [&'static str; 2] {
    [choices[0].0, choices[1].0]
}

/// The picked option's const: its name and its value.
pub fn picked<T: 'static>(choices: &[Choice<T>; 2], option: &str) -> (&'static str, &'static T) {
    let (_, name, value) = choices
        .iter()
        .find(|(label, ..)| *label == option)
        .unwrap_or(&choices[0]);
    (name, *value)
}
