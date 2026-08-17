use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        ClassList, HtmlTag, Input, States,
        common::{attr, focus_ring_sx, render_polymorphic},
    },
    hooks::use_css,
    sx::{StaticSx, Sx, sx},
};

// Only ever shows up for a Box that received a tabindex (e.g. because it was
// made clickable via `onclick`), since a plain div isn't keyboard-focusable
// on its own.
static BOX_FOCUS_SX: StaticSx = StaticSx::new(|| sx().focus_visible(focus_ring_sx()));

#[derive(Props, Clone, PartialEq)]
pub struct BoxProps {
    /// Also carries event handlers (`onclick`, `onkeydown`, ...) - `extends
    /// = GlobalAttributes` covers any DOM event a caller writes, not just
    /// plain attributes, so `Box` doesn't curate its own allowlist of them.
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    /// Which element to render as - `div` by default.
    #[props(default, into)]
    component: Input<HtmlTag>,
    /// Framework-layer `Sx` for a component built on top of `Box` (e.g.
    /// `Divider`'s own base styles), registered alongside `Box`'s own.
    #[props(default)]
    framework_sx: Option<&'static StaticSx>,
    /// Non-global attributes for specific tags `component` can select -
    /// e.g. `src`/`alt` on `img`, `href`/`target` on `a`.
    #[props(default)]
    src: Option<String>,
    #[props(default)]
    alt: Option<String>,
    #[props(default)]
    href: Option<String>,
    #[props(default)]
    target: Option<String>,
    #[props(default)]
    value: Option<String>,
    #[props(default)]
    disabled: Option<bool>,
    #[props(default)]
    r#type: Option<String>,
    children: Element,
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let focus_class = use_css(&BOX_FOCUS_SX, CssLayer::Framework);
    let framework_class = props
        .framework_sx
        .and_then(|sx| use_css(sx, CssLayer::Framework));
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_css(sx, CssLayer::UserStatic));

    let data_state = props.states.as_ref().and_then(States::data_state);

    let class = props
        .class
        .unwrap_or_default()
        .with(framework_class)
        .with(focus_class)
        .with(static_class);
    let component = props.component.as_ref().copied().unwrap_or_default();

    let attributes = props
        .attributes
        .into_iter()
        .chain(
            [
                props.src.map(|value| attr("src", value)),
                props.alt.map(|value| attr("alt", value)),
                props.href.map(|value| attr("href", value)),
                props.target.map(|value| attr("target", value)),
                props.value.map(|value| attr("value", value)),
                props.disabled.map(|value| attr("disabled", value)),
                props.r#type.map(|value| attr("type", value)),
            ]
            .into_iter()
            .flatten(),
        )
        .collect::<Vec<_>>();

    render_polymorphic(component, class, data_state, attributes, props.children)
}
