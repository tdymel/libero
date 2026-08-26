use dioxus::prelude::*;

use crate::{
    components::{
        Dialog, Float, Input, Placement, States,
        common::{base_props, input_from_str},
    },
    hooks::use_css,
    str_enum::str_enum,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{DRAWER_SIZE, Size},
};

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
// Static, so the CSS is built once for the process rather than per render,
// and `Float`'s props settle by address instead of walking the entry tree.
static DRAWER_FLOAT_VERTICAL_SX: StaticSx = StaticSx::new(|| sx().bottom("0"));
static DRAWER_FLOAT_HORIZONTAL_SX: StaticSx = StaticSx::new(|| sx().right("0"));

fn drawer_float_placement(anchor: DrawerAnchor) -> (Placement, &'static StaticSx) {
    match anchor {
        DrawerAnchor::Left => (Placement::TopStart, &DRAWER_FLOAT_VERTICAL_SX),
        DrawerAnchor::Right => (Placement::TopEnd, &DRAWER_FLOAT_VERTICAL_SX),
        DrawerAnchor::Top => (Placement::TopStart, &DRAWER_FLOAT_HORIZONTAL_SX),
        DrawerAnchor::Bottom => (Placement::BottomStart, &DRAWER_FLOAT_HORIZONTAL_SX),
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
        children: Element,
    }
}

/// The docked panel itself. It knows nothing about being open - the layer,
/// the portal and the dismissal are [`crate::hooks::use_drawer`]'s.
#[component]
pub(crate) fn Drawer(props: DrawerProps) -> Element {
    let anchor = props.anchor.copied_or_default();
    let size = props.size.copied_or(Size::Md);

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

    rsx! {
        Float {
            placement: Input::Value(placement),
            z_index: props.z_index.clone(),
            sx: float_sx,
            Dialog {
                // A drawer's own content owns its dismissal.
                close_button: false,
                class,
                sx: props.sx.clone(),
                states,
                attributes: props.attributes.clone(),
                {props.children}
            }
        }
    }
}
