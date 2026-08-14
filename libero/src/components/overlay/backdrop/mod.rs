use dioxus::prelude::*;

use crate::{
    components::{Input, States, common::class_list},
    context::use_sx,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
};

mod presence;

use presence::use_presence;

// Mirrors MUI's Fade transition: entering (fade in) is slower than leaving
// (fade out), both on MUI's default easeInOut easing curve.
const BACKDROP_TRANSITION_ENTER: &str = "opacity 225ms cubic-bezier(0.4, 0, 0.2, 1)";
const BACKDROP_TRANSITION_EXIT: &str = "opacity 195ms cubic-bezier(0.4, 0, 0.2, 1)";
// Conservative default, high enough to sit above regular page content
// without reaching into territory reserved for things like toasts.
const BACKDROP_Z_INDEX: &str = "100";

// Mirrors MUI's Backdrop: a full-viewport, semi-transparent scrim that narrows
// the user's focus onto whatever floats above it (a dialog, a drawer, ...).
// It only handles the visual layer + open/close transition here - focus
// trapping and escape/outside-click handling belong to the components that
// use a Backdrop (e.g. a future Modal), since a bare Backdrop may have none
// of those needs (e.g. a loading overlay).
static BACKDROP_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .background("rgba(0, 0, 0, 0.5)")
        .opacity("0")
        .pointer_events("none")
        .z_index(BACKDROP_Z_INDEX)
        .transition(BACKDROP_TRANSITION_EXIT)
        .when(
            "open",
            sx().opacity("1")
                .pointer_events("auto")
                .transition(BACKDROP_TRANSITION_ENTER),
        )
});

#[derive(Props, Clone, PartialEq)]
pub struct BackdropProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    open: bool,
    #[props(default, into)]
    z_index: Input<ThemeAwareValue>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    children: Option<Element>,
}

#[component]
pub fn Backdrop(props: BackdropProps) -> Element {
    let presence = use_presence(props.open);

    if !presence.mounted() {
        return rsx! {};
    }

    let framework_class = use_sx(&BACKDROP_BASE_SX, crate::SxLayer::Framework);

    let dynamic_sx = sx().apply_if(props.z_index.as_ref(), |sx, z_index| {
        sx.z_index(z_index.clone())
    });
    let dynamic_class = use_sx(&dynamic_sx, crate::SxLayer::UserDynamic);

    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_sx(sx, crate::SxLayer::UserStatic));

    let class = class_list([props.class, framework_class, dynamic_class, static_class]);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("open", presence.visible());
    let data_state = states.data_state();

    rsx! {
        div {
            class: class,
            "data-state": data_state,
            onclick: move |event| props.onclick.call(event),
            onmounted: move |_| presence.on_mounted(props.open),
            ontransitionend: move |_| presence.on_transition_end(props.open),
            ..props.attributes,
            {props.children}
        }
    }
}
