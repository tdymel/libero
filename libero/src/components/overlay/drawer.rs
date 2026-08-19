use dioxus::prelude::*;

use crate::{
    components::{
        Dialog, Float, Input, Modal, Placement, States,
        common::{base_props, input_from_str},
        variables,
    },
    hooks::use_css,
    hooks::use_portal,
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, DRAWER_SIZE, Size},
};

const DRAWER_Z_INDEX_VAR: CssVar = CssVar::new("--lsx-drawer-z-index");

// Rides along as an extra class on `Dialog`'s own. Edge-docking is `Float`'s
// job; this only fills the slot it anchors.
static DRAWER_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .margin("0")
        .border_radius("0")
        .max_width("none")
        .when("anchor-left", sx().height("100%"))
        .when("anchor-right", sx().height("100%"))
        .when("anchor-top", sx().width("100%"))
        .when("anchor-bottom", sx().width("100%"));

    Size::ALL.into_iter().fold(base, |acc, size| {
        let state = size.state_name();
        let value = DRAWER_SIZE.value(size);

        acc.when(
            format!("anchor-left && {state} || anchor-right && {state}"),
            sx().width(value.clone()),
        )
        .when(
            format!("anchor-top && {state} || anchor-bottom && {state}"),
            sx().height(value),
        )
    })
});

/// A corner `Float` placement already pins two adjacent sides with no
/// `transform`, so an anchor only needs the third offset to reach a full edge.
fn drawer_float_placement(anchor: DrawerAnchor) -> (Placement, Sx) {
    match anchor {
        DrawerAnchor::Left => (Placement::TopStart, sx().bottom("0")),
        DrawerAnchor::Right => (Placement::TopEnd, sx().bottom("0")),
        DrawerAnchor::Top => (Placement::TopStart, sx().right("0")),
        DrawerAnchor::Bottom => (Placement::BottomStart, sx().right("0")),
    }
}

str_enum! {
    pub enum DrawerAnchor {
        #[default]
        Left = "left",
        Right = "right",
        Top = "top",
        Bottom = "bottom",
    }
}

input_from_str!(DrawerAnchor);

base_props! {
    pub struct DrawerProps {
        /// The edge the drawer docks to.
        #[props(default, into)]
        anchor: Input<DrawerAnchor>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Requested by Escape or a backdrop click. `Drawer` tracks no
        /// open/closed state.
        #[props(default)]
        onclose: EventHandler<()>,
        children: Element,
    }
}

/// A portaled, dimmed, focus-trapped panel docked to one edge, closing on
/// Escape or backdrop click. For an in-flow panel, see `Sidebar`.
#[component]
pub fn Drawer(props: DrawerProps) -> Element {
    let anchor = props.anchor.copied_or_default();
    let size = props.size.copied_or(Size::Md);
    let variables = variables().with(DRAWER_Z_INDEX_VAR, props.z_index.resolve(None));

    let states = props
        .states
        .unwrap_or_default()
        .active(size.state_name())
        .with("anchor-left", anchor == DrawerAnchor::Left)
        .with("anchor-right", anchor == DrawerAnchor::Right)
        .with("anchor-top", anchor == DrawerAnchor::Top)
        .with("anchor-bottom", anchor == DrawerAnchor::Bottom);

    let drawer_class = use_css(Some(&DRAWER_SX), crate::CssLayer::Framework);
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
