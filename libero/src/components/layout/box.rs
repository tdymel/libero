use dioxus::{core::AttributeValue, prelude::*};

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States,
        common::{attr, class_list, focus_ring_sx, render_polymorphic},
    },
    hooks::{ElementRef, use_css},
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
    #[props(default)]
    class: Option<String>,
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
    /// Binds this element to an [`ElementRef`] (via `use_element_ref`) - the
    /// same role React's `ref`/Yew's `NodeRef` play. `Box` wires the
    /// underlying `onmounted` itself, merging it with a caller-supplied
    /// `onmounted` (extended via `GlobalAttributes`) if there is one -
    /// both run, `element_ref` first.
    #[props(default, into)]
    element_ref: Option<ElementRef>,
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

    let class = class_list([props.class, framework_class, focus_class, static_class]);
    let component = props.component.as_ref().copied().unwrap_or_default();

    // A caller can also set `onmounted` directly (via `extends =
    // GlobalAttributes`), which lands in `props.attributes` as its own
    // `Attribute`. Pull it out here so it can be merged with `element_ref`'s
    // own `onmounted` below instead of the two silently overwriting each
    // other once both are spread onto the same element (dioxus's attribute
    // diffing keeps only the last-written value for a given name).
    let mut attributes = props.attributes;
    let caller_onmounted = attributes
        .iter()
        .position(|attribute| attribute.name == "onmounted")
        .map(|index| attributes.remove(index))
        .and_then(|attribute| match attribute.value {
            AttributeValue::Listener(listener) => Some(listener),
            _ => None,
        });

    let onmounted = match (props.element_ref, caller_onmounted) {
        (None, None) => None,
        (None, Some(listener)) => Some(attr("onmounted", AttributeValue::Listener(listener))),
        (Some(mut element_ref), None) => Some(attr(
            "onmounted",
            AttributeValue::listener(move |event: Event<MountedData>| {
                element_ref.set(event.data.clone());
            }),
        )),
        (Some(mut element_ref), Some(listener)) => Some(attr(
            "onmounted",
            AttributeValue::listener(move |event: Event<MountedData>| {
                element_ref.set(event.data.clone());
                listener.call(event.into_any());
            }),
        )),
    };

    let attributes = attributes
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
                onmounted,
            ]
            .into_iter()
            .flatten(),
        )
        .collect::<Vec<_>>();

    render_polymorphic(component, class, data_state, attributes, props.children)
}
