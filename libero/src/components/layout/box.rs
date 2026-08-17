use dioxus::prelude::*;

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
    /// underlying `onmounted` itself, merging it with `onmounted` below if
    /// the caller also sets that - both run, `element_ref` first.
    #[props(default, into)]
    element_ref: Option<ElementRef>,
    /// Declared explicitly (rather than left to `extends = GlobalAttributes`
    /// above) so `Box` can merge it with `element_ref`'s own listener via a
    /// plain `EventHandler::call` - forwarding a caller's `onmounted`
    /// obtained any other way (extracted from a `Vec<Attribute>` as a raw
    /// `ListenerCallback`, invoked directly) silently receives the wrong
    /// event data, a dioxus_core footgun reproduced independently of this
    /// library. An explicit field of this name here takes priority over
    /// `extends` for the same attribute, so this doesn't change how callers
    /// already write `onmounted:` at a `Box` call site.
    #[props(default)]
    onmounted: EventHandler<MountedEvent>,
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

    let mut element_ref = props.element_ref;
    let caller_onmounted = props.onmounted;
    let onmounted = move |event: Event<MountedData>| {
        if let Some(element_ref) = element_ref.as_mut() {
            element_ref.set(event.data.clone());
        }
        caller_onmounted.call(event);
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
        attributes,
        onmounted,
        props.children,
    )
}
