use dioxus::prelude::*;

use crate::{
    components::{
        common::{Input, ScaleOrCss, base_props, input_from_str, with_own},
        layout::{Float, Placement},
        overlay::Dialog,
    },
    context::Dismiss,
    hooks::{use_css, use_theme},
    str_enum::str_enum,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{DRAWER_SIZE, Size},
};

// Rides along as an extra class on `Dialog`'s own. Edge-docking is `Float`'s
// job; this only fills the slot it anchors.
static DRAWER_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        // Behind a state, to outrank `Dialog`'s base: two bare classes tie, and
        // the CSS hash order decides.
        .when(
            "anchor-start || anchor-end || anchor-top || anchor-bottom",
            // The viewport caps a wide size: a phone is narrower than `xl`.
            sx().margin("0").border_radius("0").max_width("100vw"),
        )
        .when("anchor-start", sx().height("100%"))
        .when("anchor-end", sx().height("100%"))
        .when("anchor-top", sx().width("100%"))
        .when("anchor-bottom", sx().width("100%"))
        // A drawer opened from inside one must not inherit its custom size.
        .var(DRAWER_SIZE.override_var(), "initial");

    Size::ALL.into_iter().fold(base, |acc, size| {
        let state = size.state_name();
        let value = DRAWER_SIZE.overridable(size);

        acc.when(
            format!("anchor-start && {state} || anchor-end && {state}"),
            sx().width(value.clone()),
        )
        .when(
            format!("anchor-top && {state} || anchor-bottom && {state}"),
            sx().height(value),
        )
    })
});

/// A corner `Float` placement pins two sides; the third offset makes a full edge.
static DRAWER_FLOAT_VERTICAL_SX: StaticSx = StaticSx::new(|| sx().bottom("0"));
static DRAWER_FLOAT_HORIZONTAL_SX: StaticSx = StaticSx::new(|| sx().right("0"));

fn drawer_float_placement(anchor: DrawerAnchor) -> (Placement, &'static StaticSx) {
    match anchor {
        DrawerAnchor::Start => (Placement::TopStart, &DRAWER_FLOAT_VERTICAL_SX),
        DrawerAnchor::End => (Placement::TopEnd, &DRAWER_FLOAT_VERTICAL_SX),
        DrawerAnchor::Top => (Placement::TopStart, &DRAWER_FLOAT_HORIZONTAL_SX),
        DrawerAnchor::Bottom => (Placement::BottomStart, &DRAWER_FLOAT_HORIZONTAL_SX),
    }
}

str_enum! {
    /// The edge a drawer docks to. Logical: `Start` is the right under `dir="rtl"`.
    pub enum DrawerAnchor {
        #[default]
        Start = "start",
        End = "end",
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
        /// The width (or height, docked top or bottom): a size word or any CSS, as `size: "24rem"`.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        #[props(default, into)]
        aria_label: Option<String>,
        #[props(default)]
        ondismiss: Option<Callback<Dismiss, bool>>,
        children: Element,
    }
}

/// The docked panel itself. It knows nothing about being open - the layer,
/// the portal and the dismissal are [`crate::hooks::use_drawer`]'s.
#[component]
pub(crate) fn Drawer(props: DrawerProps) -> Element {
    let anchor = props.anchor.copied_or_default();
    let theme = use_theme();
    let size = ScaleOrCss::new(props.size.as_ref(), theme.drawer.size);
    let sx = with_own(
        size.custom_css(DRAWER_SIZE)
            .map(|css| sx().var(DRAWER_SIZE.override_var(), css)),
        &props.sx,
    );

    let states = props
        .states
        .unwrap_or_default()
        .active(size.size.state_name())
        .with("anchor-start", anchor == DrawerAnchor::Start)
        .with("anchor-end", anchor == DrawerAnchor::End)
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
                aria_label: props.aria_label.clone(),
                ondismiss: props.ondismiss,
                class,
                sx,
                states,
                attributes: props.attributes.clone(),
                {props.children}
            }
        }
    }
}
