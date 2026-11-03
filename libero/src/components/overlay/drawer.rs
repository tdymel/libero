use dioxus::prelude::*;

use crate::{
    components::{
        common::{Input, ScaleOrCss, base_props, input_from_str, safe_area_padding, with_own},
        layout::{Float, Placement},
        overlay::Dialog,
    },
    context::{Dismiss, ModalContext},
    hooks::{
        SwipeClose, listener, swipe_close_sx, use_css, use_element, use_swipe_close, use_theme,
    },
    str_enum::str_enum,
    sx::{FORCED_COLORS, StaticSx, Sx, ThemeAwareValue, Variables, sx},
    theme::{Color, ColorShade, ColorValue, DRAWER_SIZE, Size, SizeCss},
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
            // The panel scrolls inside its own box, not the whole layer (3003).
            sx().margin("0")
                .border_radius("0")
                .max_width("100vw")
                .overflow_y("auto"),
        )
        .when("anchor-start", sx().height("100%").and(safe_edges(true)))
        .when("anchor-end", sx().height("100%").and(safe_edges(false)))
        // Taller than the viewport, its far edge would sit above the scroll origin (3002).
        .when(
            "anchor-top",
            sx().width("100%")
                .max_height("100dvh")
                .and(safe_sides("top")),
        )
        // Relative: the swipe strip lies over its top padding. Follows a swipe down (3970).
        .when(
            "anchor-bottom",
            sx().width("100%")
                .max_height("100dvh")
                .position("relative")
                .and(safe_sides("bottom"))
                .and(swipe_close_sx()),
        )
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

/// A side panel clears the notch on its inline edge and the bars above and below (5443).
fn safe_edges(start: bool) -> Sx {
    let padding = SizeCss::SPACING.value(Size::Lg);
    let side = |physical: &str| {
        let property = if start {
            "padding-inline-start"
        } else {
            "padding-inline-end"
        };
        sx().with(property, safe_area_padding(&padding, physical))
    };
    let (ltr, rtl) = if start {
        ("left", "right")
    } else {
        ("right", "left")
    };
    sx().with("padding-top", safe_area_padding(&padding, "top"))
        .with("padding-bottom", safe_area_padding(&padding, "bottom"))
        .and(side(ltr))
        .rtl(side(rtl))
}

/// A top or bottom panel clears the bar at its edge and the notch on both sides.
fn safe_sides(edge: &str) -> Sx {
    let padding = SizeCss::SPACING.value(Size::Lg);
    sx().with(format!("padding-{edge}"), safe_area_padding(&padding, edge))
        .with("padding-left", safe_area_padding(&padding, "left"))
        .with("padding-right", safe_area_padding(&padding, "right"))
}

// As tall as the panel's top padding, whatever sets it: no padding, no strip.
static DRAWER_HANDLE_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("0")
        .left("0")
        .right("0")
        .height("0")
        .padding_top("inherit")
        .box_sizing("content-box")
        .touch_action("none")
        // A pressed look also makes Chromium's touch adjustment keep a finger on the strip,
        // not snap it to a header button below (3971).
        .selector("&:active", sx().cursor("grabbing"))
        .selector(
            "&:active > *",
            sx().background(ColorValue::Shade(Color::Muted, ColorShade::S8).value())
                .media(FORCED_COLORS, sx().background("CanvasText")),
        )
});

static DRAWER_HANDLE_BAR_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left("50%")
        .width("2rem")
        .height("4px")
        .transform("translate(-50%, -50%)")
        .border_radius("999px")
        .background(ColorValue::Shade(Color::Muted, ColorShade::S6).value())
        .media(FORCED_COLORS, sx().background("CanvasText"))
});

/// A corner `Float` placement pins two sides; the third offset makes a full edge.
static DRAWER_FLOAT_VERTICAL_SX: StaticSx = StaticSx::new(|| sx().bottom("0"));
// Both sides: under RTL the placement pins the right already, and `left` would stay auto.
static DRAWER_FLOAT_HORIZONTAL_SX: StaticSx = StaticSx::new(|| sx().left("0").right("0"));

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
        #[props(default, into)]
        title: Option<String>,
        #[props(default)]
        ondismiss: Option<Callback<Dismiss, bool>>,
        /// Docked bottom: draws a grab bar in the swipe strip.
        #[props(default)]
        handle: bool,
        /// A header close button, which asks `ondismiss` with `Dismiss::CloseButton`.
        #[props(default)]
        close_button: bool,
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
    let modal = try_use_context::<ModalContext>();
    // The panel follows a swipe down anywhere on it and slides off, as the phone sheet's (4994).
    let panel = use_element();
    let SwipeClose {
        onpointerdown,
        listeners,
        variables,
    } = use_swipe_close(
        panel,
        Some(use_callback(move |()| {
            modal.is_some_and(|modal| modal.may_dismiss(Dismiss::Swipe))
        })),
        use_callback(move |()| {
            if let Some(modal) = modal {
                modal.close();
            }
        }),
    );
    let mut attributes = props.attributes.clone();
    let bottom = anchor == DrawerAnchor::Bottom;
    if bottom {
        attributes.push(listener("onmounted", panel.mount()));
        attributes.extend(listeners);
    }
    let strip_class = use_css(Some(&DRAWER_HANDLE_SX), crate::CssLayer::Framework);
    let bar_class = use_css(Some(&DRAWER_HANDLE_BAR_SX), crate::CssLayer::Framework);

    rsx! {
        Float {
            placement: Input::Value(placement),
            z_index: props.z_index.clone(),
            sx: float_sx,
            Dialog {
                // Off unless asked: the content may draw its own close.
                close_button: props.close_button,
                aria_label: props.aria_label.clone(),
                title: props.title.clone(),
                ondismiss: props.ondismiss,
                class,
                sx,
                states,
                variables: if bottom { variables } else { Variables::new() },
                attributes,
                // Touch only, beside Back, Escape and a close button (WCAG 2.5.1).
                if bottom {
                    div {
                        class: strip_class,
                        "data-slot": "handle",
                        "data-swipe-handle": "true",
                        aria_hidden: "true",
                        onpointerdown: move |event| onpointerdown.call(event),
                        if props.handle {
                            span { class: bar_class }
                        }
                    }
                }
                {props.children}
            }
        }
    }
}
