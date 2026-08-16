use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::class_list},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, TextDefaults},
};

fn navigation_target_href(to: NavigationTarget) -> String {
    match to {
        NavigationTarget::Internal(url) | NavigationTarget::External(url) => url,
    }
}

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

#[derive(Props, Clone, PartialEq)]
pub struct AnchorProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
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

#[component]
pub fn Anchor(props: AnchorProps) -> Element {
    let dynamic_class =
        crate::hooks::use_css(&anchor_dynamic_sx(&props), crate::CssLayer::UserDynamic);
    let class = class_list([props.class, dynamic_class]);
    let is_blank = props.target.as_deref() == Some("_blank");
    let router_can_handle_target = props.target.is_none() || is_blank;

    if router_can_handle_target && try_router().is_some() {
        return rsx! {
            Link {
                to: props.to,
                class,
                new_tab: is_blank,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    rsx! {
        Box {
            component: "a",
            class,
            sx: props.sx,
            states: props.states,
            framework_sx: &ANCHOR_BASE_SX,
            href: Some(navigation_target_href(props.to)),
            target: props.target,
            attributes: props.attributes,
            {props.children}
        }
    }
}
