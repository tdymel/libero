use dioxus::prelude::*;

use crate::{
    SxLayer,
    components::{
        HtmlTag, Input, States,
        common::{class_list, render_polymorphic},
    },
    context::use_sx,
    sx::{StaticSx, Sx, sx},
};

// Only ever shows up for a Box that received a tabindex (e.g. because it was
// made clickable via `onclick`), since a plain div isn't keyboard-focusable
// on its own.
static BOX_FOCUS_SX: StaticSx = StaticSx::new(|| {
    sx().focus_visible(
        sx().outline("2px solid var(--lsx-primary-6)")
            .outline_offset("2px"),
    )
});

#[derive(Props, Clone, PartialEq)]
pub struct BoxProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    /// Which element to render as - `div` (default), `span`, `a`, `button`,
    /// or any other tag [`HtmlTag`] supports.
    #[props(default, into)]
    component: Input<HtmlTag>,
    /// A framework-layer `Sx` for whatever's using `Box` as its base (e.g.
    /// `Divider`'s own base styles) - registered alongside `Box`'s own,
    /// rather than every caller registering its own Framework-layer `Sx`
    /// and threading the class into `class` by hand.
    #[props(default)]
    framework_sx: Option<&'static StaticSx>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Element,
}

#[component]
pub fn Box(props: BoxProps) -> Element {
    let focus_class = use_sx(&BOX_FOCUS_SX, SxLayer::Framework);
    let framework_class = props
        .framework_sx
        .and_then(|sx| use_sx(sx, SxLayer::Framework));
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, SxLayer::UserStatic));

    let data_state = props.states.as_ref().and_then(States::data_state);

    let class = class_list([props.class, framework_class, focus_class, static_class]);
    let component = props.component.as_ref().copied().unwrap_or_default();

    render_polymorphic(
        component,
        class,
        data_state,
        props.attributes,
        props.onclick,
        props.children,
    )
}
