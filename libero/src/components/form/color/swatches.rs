//! The preset colors a `ColorPicker` offers under its panel.

use super::ColorCode;
use crate::utils::warn;

/// A list of preset colors, built from `ColorCode`s or from CSS strings.
///
/// Strings are parsed at runtime - the list usually comes from config or a
/// design token file. One that parses as no color is skipped with a dev
/// warning, never a panic.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{ColorCode, ColorPicker};
/// # fn app() -> Element {
/// # let color = use_signal(ColorCode::default);
/// # rsx! {
/// ColorPicker { value: color(), swatches: ["#2e2e2e", "#868e96", "rgba(250, 82, 82, 0.5)"] }
/// # } }
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Swatches {
    colors: Vec<ColorCode>,
    /// One per color; `None` names the swatch by its hex.
    labels: Vec<Option<String>>,
}

impl Swatches {
    pub fn new(colors: Vec<ColorCode>) -> Self {
        let labels = vec![None; colors.len()];
        Self { colors, labels }
    }

    /// Colors with the names a screen reader hears instead of their hex.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::components::{ColorCode, ColorPicker, Swatches};
    /// # fn app() -> Element {
    /// # let color = use_signal(ColorCode::default);
    /// # rsx! {
    /// ColorPicker {
    ///     value: color(),
    ///     swatches: Swatches::labelled([("#fa5252", "Red"), ("#40c057", "Green")]),
    /// }
    /// # } }
    /// ```
    pub fn labelled<C: AsRef<str>, L: Into<String>>(
        entries: impl IntoIterator<Item = (C, L)>,
    ) -> Self {
        let mut swatches = Self::default();
        for (text, label) in entries {
            if let Some(color) = parse(text.as_ref()) {
                swatches.colors.push(color);
                swatches.labels.push(Some(label.into()));
            }
        }
        swatches
    }

    pub fn into_vec(self) -> Vec<ColorCode> {
        self.colors
    }

    /// Swatch `index`'s caller-given name, if it has one.
    pub(crate) fn label(&self, index: usize) -> Option<&str> {
        self.labels.get(index)?.as_deref()
    }

    fn parsed<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        texts.into_iter().filter_map(parse).collect()
    }
}

/// One color, or `None` with a dev warning for a string that is none.
fn parse(text: &str) -> Option<ColorCode> {
    match text.parse() {
        Ok(color) => Some(color),
        Err(_) => {
            warn(&format!(
                "ColorPicker: swatch `{text}` is not a color and is skipped."
            ));
            None
        }
    }
}

impl std::ops::Deref for Swatches {
    type Target = [ColorCode];

    fn deref(&self) -> &Self::Target {
        &self.colors
    }
}

impl From<Vec<ColorCode>> for Swatches {
    fn from(colors: Vec<ColorCode>) -> Self {
        Self::new(colors)
    }
}

impl From<Vec<&str>> for Swatches {
    fn from(texts: Vec<&str>) -> Self {
        Self::parsed(texts)
    }
}

impl From<Vec<String>> for Swatches {
    fn from(texts: Vec<String>) -> Self {
        Self::parsed(texts.iter().map(String::as_str))
    }
}

impl From<&[&str]> for Swatches {
    fn from(texts: &[&str]) -> Self {
        Self::parsed(texts.iter().copied())
    }
}

impl<const N: usize> From<[&str; N]> for Swatches {
    fn from(texts: [&str; N]) -> Self {
        Self::parsed(texts)
    }
}

impl<const N: usize> From<[ColorCode; N]> for Swatches {
    fn from(colors: [ColorCode; N]) -> Self {
        Self::new(colors.into())
    }
}

impl FromIterator<ColorCode> for Swatches {
    fn from_iter<T: IntoIterator<Item = ColorCode>>(colors: T) -> Self {
        Self::new(colors.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_parse_and_nonsense_is_skipped() {
        let swatches = Swatches::from(["#ff0000", "not a color", "rgba(0, 0, 255, 0.5)"]);
        assert_eq!(swatches.len(), 2);
        assert_eq!(swatches[0].to_hex(), "#ff0000");
        assert_eq!(swatches[1].to_rgba(), "rgba(0, 0, 255, 0.5)");
    }

    #[test]
    fn labels_follow_their_colors_past_a_skipped_one() {
        let swatches = Swatches::labelled([("#ff0000", "Red"), ("nope", "No"), ("#00f", "Blue")]);
        assert_eq!(swatches.len(), 2);
        assert_eq!(swatches.label(1), Some("Blue"));
        assert_eq!(Swatches::from(["#ff0000"]).label(0), None);
    }
}
