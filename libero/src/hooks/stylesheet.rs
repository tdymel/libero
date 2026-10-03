mod registration;
#[cfg(test)]
mod registration_tests;

use crate::{CssLayer, css::Stylesheet};

pub(crate) use registration::{SxSource, use_box_css, use_css};

/// Registers anything that converts into a [`Stylesheet`] and returns its
/// class name, on the `UserCustom` layer so it always wins the cascade. Raw
/// `&str`/`String` CSS has no single selector, so it returns no class name.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::{sx::sx, use_stylesheet};
/// # fn app() -> Element {
/// let class = use_stylesheet(&sx().padding("md").border_radius("sm"));
///
/// rsx! {
///     div { class: class.unwrap_or_default(), "Styled once, shared by every instance" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-stylesheet>
pub fn use_stylesheet(stylesheet: impl Into<Stylesheet>) -> Option<String> {
    let stylesheet: Stylesheet = stylesheet.into();
    use_css(Some(stylesheet), CssLayer::UserCustom)
}
