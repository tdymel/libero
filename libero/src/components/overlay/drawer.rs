use dioxus::prelude::*;

use crate::{
    components::{Box, Dialog, Input, Modal, States, common::class_list},
    context::use_sx,
    hooks::use_portal,
    sx::{Sx, ThemeAwareValue, sx},
    theme::SizeCss,
};

/// `xs`/`sm`/`md`/`lg`/`xl` resolve through the drawer size scale - its own
/// scale, distinct from `Dialog`'s (a drawer is typically a narrower nav
/// panel, not a centered surface) - anything else passes through unchanged.
fn drawer_size(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::DRAWER_SIZE.value(*size).into(),
        other => other.clone(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawerVariant {
    /// Portaled, dimmed, focus-trapped, closes on Escape/backdrop click -
    /// built on `Modal`. Regardless of screen size, this is the modal-like
    /// drawer.
    Temporary,
    /// Renders in place, no portal/backdrop/focus-trap - a plain panel that
    /// occupies space in the layout, e.g. a desktop sidebar.
    Static,
}

impl Default for DrawerVariant {
    fn default() -> Self {
        Self::Temporary
    }
}

impl From<&str> for DrawerVariant {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "static" => Self::Static,
            _ => Self::Temporary,
        }
    }
}

impl From<String> for DrawerVariant {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<DrawerVariant> {
    fn from(value: &str) -> Self {
        Input::Value(DrawerVariant::from(value))
    }
}

impl From<String> for Input<DrawerVariant> {
    fn from(value: String) -> Self {
        Input::Value(DrawerVariant::from(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawerAnchor {
    Left,
    Right,
    Top,
    Bottom,
}

impl Default for DrawerAnchor {
    fn default() -> Self {
        Self::Left
    }
}

impl DrawerAnchor {
    const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

impl From<&str> for DrawerAnchor {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "right" => Self::Right,
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            _ => Self::Left,
        }
    }
}

impl From<String> for DrawerAnchor {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<DrawerAnchor> {
    fn from(value: &str) -> Self {
        Input::Value(DrawerAnchor::from(value))
    }
}

impl From<String> for Input<DrawerAnchor> {
    fn from(value: String) -> Self {
        Input::Value(DrawerAnchor::from(value))
    }
}

/// The dialog surface for `Temporary`: flush against the anchored edge and
/// the two cross edges (no gap, square corners), sized on the main axis by
/// `size`, filling the cross axis.
fn temporary_dialog_sx(anchor: DrawerAnchor, size: Option<&ThemeAwareValue>) -> Sx {
    let size = size
        .map(drawer_size)
        .unwrap_or_else(|| drawer_size(&ThemeAwareValue::from("md")));

    let sx = sx().margin("0").border_radius("0").max_width("none");

    if anchor.is_horizontal() {
        sx.height("100%").width(size)
    } else {
        sx.width("100%").height(size)
    }
    .apply_if(matches!(anchor, DrawerAnchor::Left).then_some("auto"), |sx, auto| {
        sx.margin_right(auto)
    })
    .apply_if(matches!(anchor, DrawerAnchor::Right).then_some("auto"), |sx, auto| {
        sx.margin_left(auto)
    })
    .apply_if(matches!(anchor, DrawerAnchor::Top).then_some("auto"), |sx, auto| {
        sx.margin_bottom(auto)
    })
    .apply_if(matches!(anchor, DrawerAnchor::Bottom).then_some("auto"), |sx, auto| {
        sx.margin_top(auto)
    })
}

/// The panel itself for `Static`: sized on the main axis by `size`, a border
/// on the edge facing the rest of the layout.
fn static_dynamic_sx(anchor: DrawerAnchor, size: Option<&ThemeAwareValue>) -> Sx {
    let sx = sx().apply_if(size.map(drawer_size), |sx, size| {
        if anchor.is_horizontal() {
            sx.width(size)
        } else {
            sx.height(size)
        }
    });

    match anchor {
        DrawerAnchor::Left => sx.border_right("1px solid var(--lsx-grey-3)"),
        DrawerAnchor::Right => sx.border_left("1px solid var(--lsx-grey-3)"),
        DrawerAnchor::Top => sx.border_bottom("1px solid var(--lsx-grey-3)"),
        DrawerAnchor::Bottom => sx.border_top("1px solid var(--lsx-grey-3)"),
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DrawerProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    variant: Input<DrawerVariant>,
    #[props(default, into)]
    anchor: Input<DrawerAnchor>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    #[props(default, into)]
    z_index: Input<ThemeAwareValue>,
    /// Requested by Escape/backdrop click on `Temporary` - `Static` never
    /// calls this itself, it's only there for the consumer's own close
    /// button. Doesn't imply `Drawer` tracks an open/closed state: like the
    /// rest of the library, callers control whether it exists at all.
    #[props(default)]
    onclose: EventHandler<()>,
    children: Element,
}

/// A panel anchored to one edge - `Temporary` (default) is a modal drawer on
/// top of the page regardless of screen size; `Static` is a plain in-flow
/// panel (e.g. a desktop sidebar) with no portal/backdrop/focus-trap. Compose
/// both, each hidden via CSS on the opposite side of your own breakpoint, for
/// a responsive sidebar-that-becomes-a-drawer without detecting the
/// breakpoint in Rust.
#[component]
pub fn Drawer(props: DrawerProps) -> Element {
    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let anchor = props.anchor.as_ref().copied().unwrap_or_default();

    if variant == DrawerVariant::Static {
        let dynamic_sx = static_dynamic_sx(anchor, props.size.as_ref()).apply_if(
            props.z_index.as_ref(),
            |sx, z_index| sx.z_index(z_index.clone()),
        );
        let dynamic_class = use_sx(&dynamic_sx, crate::SxLayer::UserDynamic);
        let class = class_list([props.class, dynamic_class]);

        return rsx! {
            Box {
                class: class,
                sx: props.sx,
                states: props.states,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    let dialog_sx = temporary_dialog_sx(anchor, props.size.as_ref());
    let onclose = props.onclose;
    let children = props.children.clone();

    use_portal(move || {
        Some(rsx! {
            Modal {
                onclose: move |_| onclose.call(()),
                Dialog {
                    sx: dialog_sx.clone(),
                    {children.clone()}
                }
            }
        })
    });

    rsx! {}
}
