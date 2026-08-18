use dioxus::prelude::*;

use crate::{
    components::{
        Box, Dialog, Float, Input, Modal, Placement, States, Variables, common::base_props,
        variables,
    },
    hooks::use_css,
    hooks::use_portal,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss},
};

const DRAWER_SIZE_VAR: &str = "--lsx-drawer-size-override";
const DRAWER_Z_INDEX_VAR: &str = "--lsx-drawer-z-index";

// `Static` is documented as usable for a sidebar, which is almost always a
// flex item that needs to not get squeezed and to scroll its own content
// rather than growing past its allotted space.
static DRAWER_STATIC_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .min_height("0")
        .padding("lg")
        .overflow("auto")
        .z_index(format!("var({DRAWER_Z_INDEX_VAR}, auto)"))
        .when(
            "anchor-left",
            sx().width(format!("var({DRAWER_SIZE_VAR}, auto)"))
                .border_right("1px solid var(--lsx-grey-3)"),
        )
        .when(
            "anchor-right",
            sx().width(format!("var({DRAWER_SIZE_VAR}, auto)"))
                .border_left("1px solid var(--lsx-grey-3)"),
        )
        .when(
            "anchor-top",
            sx().height(format!("var({DRAWER_SIZE_VAR}, auto)"))
                .border_bottom("1px solid var(--lsx-grey-3)"),
        )
        .when(
            "anchor-bottom",
            sx().height(format!("var({DRAWER_SIZE_VAR}, auto)"))
                .border_top("1px solid var(--lsx-grey-3)"),
        )
});

// `Temporary`'s surface, layered onto `Dialog`'s own class/vars (Dialog owns
// its own `framework_sx`, so this rides along as an extra class rather than
// replacing it). Edge-docking itself is `Float`'s job (see `Drawer`) - this
// only sizes the panel to fill the slot `Float` anchors it into.
static DRAWER_TEMPORARY_SX: StaticSx = StaticSx::new(|| {
    let default_size = SizeCss::DRAWER_SIZE.value(Size::Md);

    sx().margin("0")
        .border_radius("0")
        .max_width("none")
        .when(
            "anchor-left",
            sx().height("100%")
                .width(format!("var({DRAWER_SIZE_VAR}, {default_size})")),
        )
        .when(
            "anchor-right",
            sx().height("100%")
                .width(format!("var({DRAWER_SIZE_VAR}, {default_size})")),
        )
        .when(
            "anchor-top",
            sx().width("100%")
                .height(format!("var({DRAWER_SIZE_VAR}, {default_size})")),
        )
        .when(
            "anchor-bottom",
            sx().width("100%")
                .height(format!("var({DRAWER_SIZE_VAR}, {default_size})")),
        )
});

/// Maps a `DrawerAnchor` to a corner `Float` placement plus the one extra
/// offset that stretches it to a full edge - a corner placement anchors two
/// adjacent sides with no `transform` involved, so this only adds the third.
fn drawer_float_placement(anchor: DrawerAnchor) -> (Placement, Sx) {
    match anchor {
        DrawerAnchor::Left => (Placement::TopStart, sx().bottom("0")),
        DrawerAnchor::Right => (Placement::TopEnd, sx().bottom("0")),
        DrawerAnchor::Top => (Placement::TopStart, sx().right("0")),
        DrawerAnchor::Bottom => (Placement::BottomStart, sx().right("0")),
    }
}

/// `xs`-`xl` resolve through the drawer size scale, distinct from
/// `Dialog`'s; anything else passes through unchanged.
fn drawer_size(value: &ThemeAwareValue) -> ThemeAwareValue {
    match value {
        ThemeAwareValue::Size(size) => SizeCss::DRAWER_SIZE.value(*size).into(),
        other => other.clone(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawerVariant {
    /// Portaled, dimmed, focus-trapped, closes on Escape/backdrop - built on
    /// `Modal`.
    Temporary,
    /// In-place plain panel, no portal/backdrop/focus-trap, e.g. a sidebar.
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

// TODO: `anchor` here only picks the border/axis, not real placement - callers
// must still order `Drawer` correctly themselves. Consider an API that also
// positions it (e.g. order/margin-auto) so `anchor` is authoritative here too.

fn drawer_variables(props: &DrawerProps) -> Variables {
    variables()
        .with(
            DRAWER_SIZE_VAR,
            props.size.as_ref().map(drawer_size).and_then(|v| v.raw()),
        )
        .with(
            DRAWER_Z_INDEX_VAR,
            props.z_index.as_ref().and_then(ThemeAwareValue::raw),
        )
}

base_props! {
    pub struct DrawerProps {
        #[props(default, into)]
        variant: Input<DrawerVariant>,
        /// On `Static`, only picks the border side/size axis, not placement -
        /// position it yourself in your own layout. Full effect on `Temporary`.
        #[props(default, into)]
        anchor: Input<DrawerAnchor>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Requested by Escape/backdrop on `Temporary`; on `Static` it's only
        /// for your own close button - `Drawer` tracks no open/closed state.
        #[props(default)]
        onclose: EventHandler<()>,
        children: Element,
    }
}

/// A panel anchored to one edge - `Temporary` (default) is a modal drawer;
/// `Static` is a plain in-flow panel (e.g. a sidebar). Compose both, hidden
/// via CSS on opposite breakpoints, for a responsive sidebar-becomes-drawer.
#[component]
pub fn Drawer(props: DrawerProps) -> Element {
    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let anchor = props.anchor.as_ref().copied().unwrap_or_default();
    let variables = drawer_variables(&props);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("anchor-left", anchor == DrawerAnchor::Left)
        .with("anchor-right", anchor == DrawerAnchor::Right)
        .with("anchor-top", anchor == DrawerAnchor::Top)
        .with("anchor-bottom", anchor == DrawerAnchor::Bottom);

    if variant == DrawerVariant::Static {
        return rsx! {
            Box {
                class: props.class,
                sx: props.sx,
                states,
                variables,
                framework_sx: &DRAWER_STATIC_BASE_SX,
                // Chromium makes a scrollable `overflow: auto` region with
                // actual overflowing content an implicit tab stop of its own
                // (for arrow-key/Page-Down scrolling) unless opted out -
                // right here since this is otherwise exactly that region,
                // and its own content (e.g. nav links) is already
                // separately focusable, making the extra stop redundant.
                tabindex: "-1",
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    let drawer_class = use_css(&DRAWER_TEMPORARY_SX, crate::CssLayer::Framework);
    let class = props.class.clone().unwrap_or_default().with(drawer_class);
    let (placement, float_sx) = drawer_float_placement(anchor);

    let onclose = props.onclose;
    let z_index = props.z_index.clone();
    let sx = props.sx.clone();
    let attributes = props.attributes.clone();
    let children = props.children.clone();

    use_portal(Some(rsx! {
        Modal {
            onclose: move |_| onclose.call(()),
            Float {
                placement: Input::Value(placement),
                z_index: z_index.clone(),
                sx: float_sx.clone(),
                Dialog {
                    class: class.clone(),
                    sx: sx.clone(),
                    states: states.clone(),
                    variables: variables.clone(),
                    attributes: attributes.clone(),
                    {children.clone()}
                }
            }
        }
    }));

    rsx! {}
}
