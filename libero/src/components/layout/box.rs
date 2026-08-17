use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        ClassList, HtmlTag, Input, States, Variables,
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
    /// CSS custom properties set directly on this element's `style`
    /// attribute - lets `sx`/`framework_sx` reference a per-instance value
    /// via `var(--name, fallback)` without generating a new class for every
    /// distinct value a caller passes.
    #[props(default, into)]
    variables: Input<Variables>,
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
    /// A raw literal `style` - merged with (not overwritten by) whatever
    /// `variables` produces, `variables`' declarations first.
    #[props(default)]
    style: Option<String>,
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
    let variables_style = props
        .variables
        .as_ref()
        .map(Variables::to_string)
        .filter(|style| !style.is_empty());

    let class = props
        .class
        .unwrap_or_default()
        .with(framework_class)
        .with(focus_class)
        .with(static_class);
    let component = props.component.as_ref().copied().unwrap_or_default();

    let style = match (variables_style, props.style) {
        (Some(variables), Some(raw)) => Some(format!("{variables}{raw}")),
        (Some(style), None) | (None, Some(style)) => Some(style),
        (None, None) => None,
    };

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

    render_polymorphic(
        component,
        class,
        data_state,
        style,
        attributes,
        props.children,
    )
}
