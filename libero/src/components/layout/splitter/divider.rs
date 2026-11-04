use dioxus::prelude::*;

use crate::{
    components::{
        common::{COARSE_POINTER, COARSE_TARGET, HtmlTag, Input, Orientation, States},
        layout::use_box,
    },
    hooks::{ElementHandle, drag_handle_sx, sideways_drag_sx, use_range_css},
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{
        ColorCss, ColorShade, CssVar, Responsive, SPLITTER_DIVIDER_SIZE, SPLITTER_HIT_SIZE, Size,
    },
};

pub(super) const SPLITTER_DIVIDER_COLOR_VAR: CssVar = CssVar::new("--lsx-splitter-divider-color");

// Only as thick as `divider_size`, so the panes sit flush. The larger hit
// target is a child overlay taking no flex space of its own.
static SPLITTER_BAR_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .position("relative")
        .flex_shrink("0")
        .align_self("stretch")
        // Shade 6, 3:1 on page and paper: shade 4 was about 1.5:1, under WCAG 1.4.11.
        .background(SPLITTER_DIVIDER_COLOR_VAR.value_or(ColorCss::MUTED.value(ColorShade::S6)))
        // Forced colours turn an author fill into `Canvas`; a system colour stays.
        .media(FORCED_COLORS, sx().background("CanvasText"));

    Size::ALL.into_iter().fold(base, |acc, size| {
        acc.when(
            format!("vertical && {}", size.state_name()),
            bar_sx(Orientation::Vertical, size),
        )
        .when(
            format!("horizontal && {}", size.state_name()),
            bar_sx(Orientation::Horizontal, size),
        )
    })
});

fn bar_sx(orientation: Orientation, size: Size) -> Sx {
    match orientation {
        Orientation::Vertical => sx().width(SPLITTER_DIVIDER_SIZE.value(size)),
        Orientation::Horizontal => sx().height(SPLITTER_DIVIDER_SIZE.value(size)),
    }
}

/// A responsive range's copy of `block`, one rule per size.
fn per_size_copy(orientation: Orientation, block: fn(Orientation, Size) -> Sx) -> Sx {
    Size::ALL.into_iter().fold(sx(), |acc, size| {
        acc.when(size.state_name(), block(orientation, size))
    })
}

// Invisible, and negatively inset past the bar: for a fine pointer into pane B only,
// so pane A's own scrollbar stays clear; for a finger into both panes.
static SPLITTER_HIT_SX: StaticSx = StaticSx::new(|| {
    // A vertical swipe over a vertical divider scrolls the page (1039).
    // Above a positioned child of pane B, which would take the zone's presses (5278).
    let base = sx()
        .position("absolute")
        .z_index("1")
        .and(drag_handle_sx())
        .when("vertical", sideways_drag_sx());

    Size::ALL.into_iter().fold(base, |acc, size| {
        acc.when(
            format!("vertical && {}", size.state_name()),
            hit_sx(Orientation::Vertical, size),
        )
        .when(
            format!("horizontal && {}", size.state_name()),
            hit_sx(Orientation::Horizontal, size),
        )
    })
});

fn hit_sx(orientation: Orientation, size: Size) -> Sx {
    let divider = SPLITTER_DIVIDER_SIZE.value(size);
    let inset = format!("calc({divider} - {})", SPLITTER_HIT_SIZE.value(size));
    // A finger's 44px, centred on the line: a phone's scrollbars overlay, so none to clear.
    let coarse = format!(
        "calc(({divider} - max({}, {COARSE_TARGET})) / 2)",
        SPLITTER_HIT_SIZE.value(size)
    );
    match orientation {
        Orientation::Vertical => sx()
            .top("0")
            .bottom("0")
            .inset_inline_start("0")
            .inset_inline_end(inset)
            .cursor("col-resize")
            .media(
                COARSE_POINTER,
                sx().inset_inline_start(coarse.clone())
                    .inset_inline_end(coarse),
            ),
        Orientation::Horizontal => sx()
            .left("0")
            .right("0")
            .top("0")
            .bottom(inset)
            .cursor("row-resize")
            .media(COARSE_POINTER, sx().top(coarse.clone()).bottom(coarse)),
    }
}

/// The hit zone's copy: a vertical range also takes the sideways-only touch drag.
fn hit_copy(orientation: Orientation) -> Sx {
    let base = match orientation {
        Orientation::Vertical => sideways_drag_sx(),
        Orientation::Horizontal => sx(),
    };
    base.and(per_size_copy(orientation, hit_sx))
}

/// The divider line and its oversized drag/keyboard target. Its own scope, so
/// a `Splitter` render that didn't move it skips ~2900 ns here.
#[component]
pub(super) fn SplitterDivider(
    element: ElementHandle,
    /// Pane A's percentage; a signal, so a move re-renders this alone.
    a: ReadSignal<f64>,
    vertical: bool,
    /// Plain, the one word `vertical` resolves; responsive, run words and range copies.
    orientations: Responsive<Orientation>,
    size: Size,
    min_size: f64,
    aria_label: Option<String>,
    /// Pane A's id.
    controls: String,
    onpointerdown: Callback<Event<PointerData>>,
    onkeydown: Callback<Event<KeyboardData>>,
) -> Element {
    let states: Input<States> = States::default()
        .responsive(&orientations, |orientation| orientation.state_name())
        .with(size.state_name(), true)
        .into();
    use_range_css(
        &SPLITTER_BAR_SX,
        orientations.range_copies(
            |orientation| orientation.state_name(),
            |orientation| per_size_copy(orientation, bar_sx),
        ),
    );
    use_range_css(
        &SPLITTER_HIT_SX,
        orientations.range_copies(|orientation| orientation.state_name(), hit_copy),
    );

    let bar = use_box()
        .framework_sx(&SPLITTER_BAR_SX)
        .states(&states)
        .prepare();
    let hit = use_box()
        .framework_sx(&SPLITTER_HIT_SX)
        .states(&states)
        .prepare();

    let hit = hit
        .element(&element)
        .attr("role", "separator")
        .attr("tabindex", "0")
        .attr("aria-label", aria_label)
        .attr("aria-controls", controls)
        .attr(
            "aria-orientation",
            if vertical { "vertical" } else { "horizontal" },
        )
        .attr(
            "aria-valuenow",
            (a().clamp(min_size, 100.0 - min_size) as i64).to_string(),
        )
        .attr("aria-valuemin", (min_size as i64).to_string())
        .attr("aria-valuemax", ((100.0 - min_size) as i64).to_string())
        .event("onpointerdown", onpointerdown)
        .event("onkeydown", onkeydown)
        .render(HtmlTag::Div, Vec::new(), ());

    bar.render(HtmlTag::Div, Vec::new(), hit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// Todo 2404: the line is a fill, which forced colours would paint `Canvas`.
    #[test]
    fn the_line_turns_canvas_text_in_forced_colours() {
        let css = Stylesheet::from(&SPLITTER_BAR_SX).as_str().to_string();
        assert!(
            css.contains("@media (forced-colors: active)") && css.contains("background:CanvasText"),
            "{css}"
        );
    }
}
