use dioxus::prelude::*;

use super::{CarouselPart, numbered, state::CarouselView};
use crate::{
    components::{
        common::{Input, Part, States, focus_ring_sx, has_shortcut_modifier, states},
        layout::Box,
    },
    hooks::{ElementHandle, id_selector},
    platform::{ElementApi, logical_key},
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, sx},
    theme::{
        CAROUSEL_INDICATOR_COLOR, CAROUSEL_INDICATOR_CURRENT_COLOR,
        CAROUSEL_INDICATOR_CURRENT_LENGTH, CAROUSEL_INDICATOR_LENGTH, CAROUSEL_INDICATOR_THICKNESS,
        CAROUSEL_INDICATORS_GAP,
    },
};

static CAROUSEL_INDICATORS_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .justify_content("center")
        .align_items("center")
        .gap(CAROUSEL_INDICATORS_GAP.value())
        // Ten dots outgrow 320px; they wrap rather than overflow or shrink
        // into overlapping hit boxes (1.4.10, 2.5.8, todo 1566).
        .flex_wrap("wrap")
        .when("horizontal", sx().row_gap(cross_gap()))
        .when(
            "vertical",
            sx().flex_direction("column").column_gap(cross_gap()),
        )
        .margin_top("sm")
});

/// Wrapped lines sit 24px apart centre to centre, so their hit boxes touch
/// without overlapping.
fn cross_gap() -> String {
    format!(
        "max({}, calc(24px - {}))",
        CAROUSEL_INDICATORS_GAP.value(),
        CAROUSEL_INDICATOR_THICKNESS.value()
    )
}

pub(super) static CAROUSEL_INDICATOR_SX: StaticSx = StaticSx::new(|| {
    sx().flex_shrink("0")
        .padding("0")
        .border_width("0")
        .border_radius("999px")
        // A var, not a literal, so the ring contrasts with the page, not the dot.
        .background(CAROUSEL_INDICATOR_COLOR.value())
        .cursor("pointer")
        .transition("background-color 150ms")
        .when(
            "horizontal",
            sx().width(CAROUSEL_INDICATOR_LENGTH.value())
                .height(CAROUSEL_INDICATOR_THICKNESS.value()),
        )
        .when(
            "vertical",
            sx().height(CAROUSEL_INDICATOR_LENGTH.value())
                .width(CAROUSEL_INDICATOR_THICKNESS.value()),
        )
        // Not colour alone (1.4.1): the current dot is also longer.
        .when(
            "current",
            sx().background(CAROUSEL_INDICATOR_CURRENT_COLOR.value())
                .when(
                    "horizontal",
                    sx().width(CAROUSEL_INDICATOR_CURRENT_LENGTH.value()),
                )
                .when(
                    "vertical",
                    sx().height(CAROUSEL_INDICATOR_CURRENT_LENGTH.value()),
                ),
        )
        .media(REDUCED_MOTION, sx().transition("none"))
        // Forced colours paint every dot `Canvas`, so the strip vanished.
        .media(
            FORCED_COLORS,
            sx().background("CanvasText")
                .when("current", sx().background("Highlight")),
        )
        .focus_visible(focus_ring_sx())
        // 2.5.8: an invisible 24px box centred on the thin dot takes the pointer.
        .position("relative")
        .selector(
            "&::before",
            sx().content("\"\"")
                .position("absolute")
                .top("50%")
                .left("50%")
                .width("max(100%, 24px)")
                .height("max(100%, 24px)")
                .transform("translate(-50%, -50%)"),
        )
});

/// A dot's id, derived from the track's so two carousels do not collide.
fn indicator_id(track_id: &str, index: usize) -> String {
    format!("{track_id}-indicator-{index}")
}

/// Moves focus onto the dot the arrows just made current.
fn focus_indicator(root: ElementHandle, track_id: &str, index: usize) {
    let selector = id_selector(&indicator_id(track_id, index));
    let _ = root.query_selector(&selector).and_then(|dot| dot.focus());
}

/// The dot strip: one dot per reachable position, which under a centred or end
/// alignment does not start at zero. A looping carousel is one dot per real
/// slide instead.
#[component]
pub(super) fn CarouselIndicators(view: CarouselView) -> Element {
    let CarouselView {
        setup,
        state,
        labels,
        track_id,
        root,
        ..
    } = view;
    let nav = state.nav;
    let count = setup.count;
    let orientation = setup.orientation;
    let current = nav.current;
    let looping = nav.clones > 0;
    let (low, high) = match looping {
        true => (0, count.saturating_sub(1)),
        false => (setup.first, setup.last),
    };
    let strip_states: Input<States> = states().with(orientation.state_name(), true).into();

    rsx! {
        // A named group, so the dots read as one set (todo 549).
        Box {
            framework_sx: &CAROUSEL_INDICATORS_SX,
            states: strip_states,
            "data-slot": CarouselPart::Indicators.slot(),
            role: "group",
            aria_label: labels.indicators,
            for index in low..=high {
                Box {
                    key: "{index}",
                    component: "button",
                    r#type: "button",
                    framework_sx: &CAROUSEL_INDICATOR_SX,
                    "data-slot": CarouselPart::Indicator.slot(),
                    states: states()
                        .with(orientation.state_name(), true)
                        .with("current", index == current()),
                    // By the first slide it shows; a looping dot by its own (todo 550).
                    aria_label: numbered(
                        labels.indicator,
                        if looping { index } else { nav.showing(index).0 },
                        count,
                    ),
                    aria_current: (index == current()).then(|| "true".to_string()),
                    // Roving: one tab stop; arrows move the focus with the slide.
                    id: indicator_id(&track_id(), index),
                    tabindex: if index == current() { "0" } else { "-1" },
                    onclick: move |_| nav.go_to(index),
                    onkeydown: move |event: Event<KeyboardData>| {
                        if has_shortcut_modifier(&event) {
                            return;
                        }
                        let Some(target) = indicator_key_target(logical_key(&event), index, low, high, looping)
                        else {
                            return;
                        };
                        event.prevent_default();
                        nav.go_to(target);
                        focus_indicator(root, &track_id(), target);
                    },
                }
            }
        }
    }
}

/// Where an arrow, `Home` or `End` on a dot goes. Wraps only when the carousel
/// does, so dots and track agree.
fn indicator_key_target(
    key: Key,
    index: usize,
    low: usize,
    high: usize,
    looping: bool,
) -> Option<usize> {
    // A lone dot has nowhere to go: the keys scroll the page (todo 2465).
    if low >= high {
        return None;
    }
    Some(match key {
        Key::ArrowRight | Key::ArrowDown => match index >= high {
            true if looping => low,
            true => high,
            false => index + 1,
        },
        Key::ArrowLeft | Key::ArrowUp => match index <= low {
            true if looping => high,
            true => low,
            false => index - 1,
        },
        Key::Home => low,
        Key::End => high,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 2465: a lone dot leaves the keys to the page; two still wrap or clamp.
    #[test]
    fn a_lone_dot_takes_no_key() {
        for key in [Key::ArrowRight, Key::ArrowLeft, Key::Home, Key::End] {
            assert_eq!(indicator_key_target(key, 0, 0, 0, false), None);
        }
        assert_eq!(
            indicator_key_target(Key::ArrowRight, 1, 0, 1, true),
            Some(0)
        );
        assert_eq!(
            indicator_key_target(Key::ArrowRight, 1, 0, 1, false),
            Some(1)
        );
    }
}
