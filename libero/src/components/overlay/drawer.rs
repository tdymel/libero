use dioxus::prelude::*;

use crate::{
    components::{
        common::{Input, ScaleOrCss, base_props, input_from_str, with_own},
        layout::{Float, Placement},
        overlay::Dialog,
    },
    context::{Dismiss, ModalContext},
    hooks::{SwipeDirection, SwipeEvent, SwipeOptions, listener, use_css, use_swipe, use_theme},
    str_enum::str_enum,
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{Color, ColorShade, ColorValue, DRAWER_SIZE, Size},
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
        .when("anchor-start", sx().height("100%"))
        .when("anchor-end", sx().height("100%"))
        // Taller than the viewport, its far edge would sit above the scroll origin (3002).
        .when("anchor-top", sx().width("100%").max_height("100dvh"))
        // Relative: the swipe strip lies over its top padding.
        .when(
            "anchor-bottom",
            sx().width("100%").max_height("100dvh").position("relative"),
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
        #[props(default, into)]
        title: Option<String>,
        #[props(default)]
        ondismiss: Option<Callback<Dismiss, bool>>,
        /// Docked bottom: draws a grab bar in the swipe strip.
        #[props(default)]
        handle: bool,
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
    let swipe = use_swipe(
        use_callback(move |event: SwipeEvent| {
            if let Some(modal) = modal
                && event.direction == SwipeDirection::Down
            {
                modal.dismiss(Dismiss::Swipe);
            }
        }),
        SwipeOptions::default(),
    );
    // A press starts on the strip, its moves are heard on the panel: Blitz sends them
    // to the node under the finger, with no implicit touch capture.
    let mut held = use_hook(|| CopyValue::new(None::<i32>));
    let mut attributes = props.attributes.clone();
    if anchor == DrawerAnchor::Bottom {
        attributes.push(listener("onpointermove", move |event: PointerEvent| {
            // Blitz pans the scrolling panel unless the move is prevented.
            if *held.peek() == Some(event.pointer_id()) {
                event.prevent_default();
            }
            swipe.onpointermove.call(event);
        }));
        let end = move |event: PointerEvent| {
            if *held.peek() == Some(event.pointer_id()) {
                held.set(None);
            }
            swipe.onpointerup.call(event);
        };
        attributes.push(listener("onpointerup", end));
        attributes.push(listener("onpointercancel", end));
    }
    let strip_class = use_css(Some(&DRAWER_HANDLE_SX), crate::CssLayer::Framework);
    let bar_class = use_css(Some(&DRAWER_HANDLE_BAR_SX), crate::CssLayer::Framework);

    rsx! {
        Float {
            placement: Input::Value(placement),
            z_index: props.z_index.clone(),
            sx: float_sx,
            Dialog {
                // A drawer's own content owns its dismissal.
                close_button: false,
                aria_label: props.aria_label.clone(),
                title: props.title.clone(),
                ondismiss: props.ondismiss,
                class,
                sx,
                states,
                attributes,
                // Touch only, beside Back, Escape and the content's own close (WCAG 2.5.1).
                if anchor == DrawerAnchor::Bottom {
                    div {
                        class: strip_class,
                        "data-slot": "handle",
                        aria_hidden: "true",
                        onpointerdown: move |event: PointerEvent| {
                            let touch = event.is_primary() && event.pointer_type() != "mouse";
                            held.set(touch.then(|| event.pointer_id()));
                            swipe.onpointerdown.call(event);
                        },
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
