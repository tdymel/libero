use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{Input, States, common::base_props},
    hooks::use_css,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, TextDefaults},
};

use super::InternalAnchor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorUnderline {
    Always,
    Hover,
    Never,
}

impl Default for AnchorUnderline {
    fn default() -> Self {
        Self::Hover
    }
}

impl From<&str> for AnchorUnderline {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "always" => Self::Always,
            "never" => Self::Never,
            _ => Self::Hover,
        }
    }
}

impl From<String> for AnchorUnderline {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<AnchorUnderline> {
    fn from(value: &str) -> Self {
        Input::Value(AnchorUnderline::from(value))
    }
}

impl From<String> for Input<AnchorUnderline> {
    fn from(value: String) -> Self {
        Input::Value(AnchorUnderline::from(value))
    }
}

static ANCHOR_BASE_SX: StaticSx = StaticSx::new(|| sx().color("primary.6"));

// Reuses Text's own theme-level sizing (TextDefaults), not Text the
// component, since Text has no href/target/rel escape hatch to render a real
// anchor with.
fn anchor_size_sx(size: &ThemeAwareValue) -> Sx {
    match size {
        ThemeAwareValue::Size(Size::Xs) => TextDefaults::xs_sx(),
        ThemeAwareValue::Size(Size::Sm) => TextDefaults::sm_sx(),
        ThemeAwareValue::Size(Size::Lg) => TextDefaults::lg_sx(),
        ThemeAwareValue::Size(Size::Xl) => TextDefaults::xl_sx(),
        _ => TextDefaults::md_sx(),
    }
    .margin("0")
}

fn underline_sx(underline: AnchorUnderline) -> Sx {
    match underline {
        AnchorUnderline::Always => sx().text_decoration("underline"),
        AnchorUnderline::Never => sx().text_decoration("none"),
        AnchorUnderline::Hover => sx()
            .text_decoration("none")
            .hover(sx().text_decoration("underline")),
    }
}

fn anchor_dynamic_sx(props: &AnchorProps) -> Sx {
    let size = props
        .size
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::String("md".to_string()));
    let underline = props.underline.as_ref().copied().unwrap_or_default();

    anchor_size_sx(&size).and(underline_sx(underline))
}

base_props! {
    pub struct AnchorProps {
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        /// A plain path/URL or a typed route (anything `Into<NavigationTarget>`,
        /// e.g. `Route::Foo {}`). Resolves through the app's Dioxus router when
        /// one is mounted and `target` allows it (unset or `"_blank"`) -
        /// internal targets then get SPA navigation instead of a full page
        /// reload. Falls back to a plain `href` otherwise.
        #[props(into)]
        to: NavigationTarget,
        #[props(default)]
        target: Option<String>,
        #[props(default, into)]
        underline: Input<AnchorUnderline>,
        children: Element,
    }
}

#[component]
pub fn Anchor(props: AnchorProps) -> Element {
    let dynamic_class = use_css(&anchor_dynamic_sx(&props), CssLayer::UserDynamic);
    let class = props.class.unwrap_or_default().with(dynamic_class);

    rsx! {
        InternalAnchor {
            to: props.to,
            target: props.target,
            class,
            sx: props.sx,
            framework_sx: &ANCHOR_BASE_SX,
            states: props.states,
            attributes: props.attributes,
            {props.children}
        }
    }
}
