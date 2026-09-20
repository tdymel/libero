use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States},
        layout::use_box,
    },
    hooks::{ElementHandle, drag_handle_sx},
    sx::{StaticSx, sx},
    theme::{ColorCss, ColorShade, CssVar, SPLITTER_DIVIDER_SIZE, SPLITTER_HIT_SIZE, Size},
};

pub(super) const SPLITTER_DIVIDER_COLOR_VAR: CssVar = CssVar::new("--lsx-splitter-divider-color");

// Only as thick as `divider_size`, so the panes sit flush. The larger hit
// target is a child overlay taking no flex space of its own.
static SPLITTER_BAR_SX: StaticSx = StaticSx::new(|| {
    let base = sx()
        .position("relative")
        .flex_shrink("0")
        .align_self("stretch")
        .background(SPLITTER_DIVIDER_COLOR_VAR.value_or(ColorCss::MUTED.value(ColorShade::S4)));

    Size::ALL.into_iter().fold(base, |acc, size| {
        acc.when(
            format!("vertical && {}", size.state_name()),
            sx().width(SPLITTER_DIVIDER_SIZE.value(size)),
        )
        .when(
            format!("horizontal && {}", size.state_name()),
            sx().height(SPLITTER_DIVIDER_SIZE.value(size)),
        )
    })
});

// Invisible, and negatively inset past the bar into both panes, so the drag
// target is larger than the visible line without the panes leaving a gap.
static SPLITTER_HIT_SX: StaticSx = StaticSx::new(|| {
    let base = sx().position("absolute").and(drag_handle_sx());

    Size::ALL.into_iter().fold(base, |acc, size| {
        let inset = format!(
            "calc(({} - {}) / 2)",
            SPLITTER_DIVIDER_SIZE.value(size),
            SPLITTER_HIT_SIZE.value(size)
        );
        acc.when(
            format!("vertical && {}", size.state_name()),
            sx().top("0")
                .bottom("0")
                .left(inset.clone())
                .right(inset.clone())
                .cursor("col-resize"),
        )
        .when(
            format!("horizontal && {}", size.state_name()),
            sx().left("0")
                .right("0")
                .top(inset.clone())
                .bottom(inset)
                .cursor("row-resize"),
        )
    })
});

/// The divider line and its oversized drag/keyboard target. Its own scope, so
/// a `Splitter` render that didn't move it skips ~2900 ns here.
#[component]
pub(super) fn SplitterDivider(
    element: ElementHandle,
    /// Pane A's percentage; a signal, so a move re-renders this alone.
    a: ReadSignal<f64>,
    vertical: bool,
    size: Size,
    min_size: f64,
    aria_label: Option<String>,
    /// Pane A's id.
    controls: String,
    onpointerdown: Callback<Event<PointerData>>,
    onkeydown: Callback<Event<KeyboardData>>,
) -> Element {
    let states: Input<States> = States::default()
        .with("vertical", vertical)
        .with("horizontal", !vertical)
        .with(size.state_name(), true)
        .into();

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
