use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, base_props, variables},
        layout::use_box,
    },
    hooks::{use_cache, use_theme},
    sx::{StaticSx, sx},
    theme::{
        GRID_ITEM_ROWS_VAR, GRID_ROW_UNIT, GRID_ZONE_AREA_VAR, GRID_ZONE_CONTAINER_VAR,
        GRID_ZONE_GAP, Size, SizeCss,
    },
    utils::warn,
};

use super::{AreaName, GridContext};

/// What a `GridItem` needs from its zone. Signal-backed: both context hooks
/// run once, so a plain value would freeze at the first render.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ZoneState {
    pub masonry: bool,
    /// The masonry row quantum, in pixels. An item cannot resolve the theme's
    /// row unit or its zone's gap on its own, so the zone hands both over.
    pub unit_px: u32,
    pub gap_px: u32,
    /// The zone's `@container` name, for an item keying a span off it.
    /// Empty when the zone has no area, i.e. nothing to name it after.
    pub container: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct GridZoneContext {
    pub state: Signal<ZoneState>,
}

/// Only `GridItem` writes this token. Selecting bare `[data-state]` would hit
/// every `Box`-derived child - it is the library's universal state attribute -
/// and crush a stray `Text` into one row.
pub(crate) const GRID_ITEM_STATE: &str = "grid-item";

/// The zone's `@container` name. Prefixed, so it cannot collide with a name
/// the caller chose for a container of their own.
pub(crate) fn container_name(area: &str) -> String {
    format!("lsx-zone-{area}")
}

static GRID_ZONE_SX: StaticSx = StaticSx::new(|| {
    let item = format!("& > [data-state~=\"{GRID_ITEM_STATE}\"]");
    let measured = format!("{item}[data-state~=\"measured\"]");

    sx().display("grid")
        .grid_area(GRID_ZONE_AREA_VAR.value_or("auto"))
        .container_name(GRID_ZONE_CONTAINER_VAR.value_or("none"))
        .grid_template_columns("repeat(12, minmax(0, 1fr))")
        .align_content("start")
        // A grid item defaults to `min-width: auto`, which refuses to shrink
        // below its content - the outer track's `minmax(0, 1fr)` protects the
        // *track*, not the zone sitting in it.
        .min_width("0")
        // Twelve tracks always carry eleven gaps, whatever spans them, and a
        // length gap cannot shrink: below `11 * gap` the tracks bottom out at
        // zero and the zone overflows its area. At the theme's 12px that
        // floor is 132px, which a quarter-width sidebar hits on a phone. The
        // percentage resolves against the zone's own width, so the cap only
        // engages under ~300px and wider zones keep the themed gap exactly.
        .column_gap(format!("min({}, 4%)", GRID_ZONE_GAP.value()))
        .row_gap(GRID_ZONE_GAP.value())
        // Only a zone filling a named area, i.e. one sitting in a `Grid`'s
        // `minmax(0, 1fr)` track, whose width therefore comes from its parent.
        // Inline-size containment zeroes an element's intrinsic contribution,
        // so on a shrink-to-fit box - a standalone zone as a flex item - it
        // collapses the zone to nothing. A zone with no area has no name to be
        // queried by either, so it gains nothing from being a container.
        .when("container", sx().container_type("inline-size"))
        .when("dense", sx().grid_auto_flow("row dense"))
        .when(
            "masonry",
            sx()
                // The quantum is the row unit *plus* the row gap, so a gapped
                // masonry zone is ragged by up to a whole gap however small
                // the unit gets. Zero it and carry the spacing on the item.
                .row_gap("0")
                // Which leaves the last item in every column carrying a bottom
                // margin the zone does not want. Cancel it once, here.
                .margin_bottom(format!("calc(-1 * {})", GRID_ZONE_GAP.value()))
                // `minmax`, not the bare unit: an item that has not measured
                // yet sits at `grid-row: auto`, and a flat 2px track would
                // make it overflow onto everything below it - permanently
                // under SSR and off the web. The `auto` maximum lets that one
                // track grow to its content instead, so an unmeasured zone is
                // an ordinary grid rather than a pile.
                .grid_auto_rows(format!("minmax({}, auto)", GRID_ROW_UNIT.value()))
                .selector(
                    &item,
                    // Load-bearing: at the default `stretch` an item's border
                    // box would be its whole row span, the next measurement
                    // would report *that*, and the span would climb. Scoped to
                    // masonry, or ordinary zones silently lose equal-height
                    // cards.
                    sx().align_self("start")
                        .margin_bottom(GRID_ZONE_GAP.value()),
                )
                // (0,3,0), against the item's own (0,2,0) `grid-column` rule
                // in the other stylesheet. Both are `CssLayer::Framework` and
                // the registry emits those in hash order, so specificity is
                // the only thing settling this - `& > *` would invert it.
                .selector(
                    &measured,
                    sx().grid_row(format!("span {}", GRID_ITEM_ROWS_VAR.value_or("1"))),
                ),
        )
});

base_props! {
    pub struct GridZoneProps {
        /// `GridItem`s.
        children: Element,
        /// Which of the parent `Grid`'s areas this fills. Omit it to use the
        /// zone on its own, without a `Grid` - a plain masonry wall needs no
        /// template.
        #[props(default, into)]
        area: AreaName,
        /// Backfill gaps a wider item left behind. Pure CSS, no measurement.
        #[props(default)]
        dense: bool,
        /// Measure item heights and pack them with no vertical dead space.
        /// Costs a `ResizeObserver` per item.
        #[props(default)]
        masonry: bool,
        /// Between items.
        #[props(default, into)]
        gap: Input<Size>,
        #[props(default, into)]
        component: Input<HtmlTag>,
    }
}

/// An independent twelfths-scale packing container. Inside a
/// [`Grid`](super::Grid) it fills one named area; on its own it is just a
/// twelve-column grid.
#[component]
pub fn GridZone(props: GridZoneProps) -> Element {
    let theme = use_theme();
    let grid = try_use_context::<GridContext>();
    // Read before this zone provides its own: an *enclosing* zone, i.e. a grid
    // nested inside a `GridItem`.
    let enclosing = try_use_context::<GridZoneContext>();

    let area = props.area;
    if let Some(grid) = grid.as_ref()
        && area.is_set()
        && !grid.template.read().contains(&area)
    {
        if grid.template.read().0.source == area.source {
            warn(&format!(
                "GridZone: area `{}` is not in the parent Grid's template.",
                area.name
            ));
        } else {
            warn(&format!(
                "GridZone: area `{}` comes from a different GridArea enum than the parent \
                 Grid's template.",
                area.name
            ));
        }
    }

    let gap = props.gap.copied_or(theme.grid.zone_gap);
    let gap_px = theme.spacing.get(gap).into();
    let container = if area.is_set() { area.name } else { "" };
    // Zone container names come from the area name, so a nested grid reusing
    // one leaves the inner zone as the nearest match for both - every
    // responsive span in the outer zone would silently key off the inner one.
    if !container.is_empty()
        && enclosing.is_some_and(|zone| zone.state.peek().container == container)
    {
        warn(&format!(
            "GridZone: a nested zone reuses the area name `{container}`, so a responsive \
             GridItem span in the outer zone would resolve against the inner one.",
        ));
    }

    let state = ZoneState {
        masonry: props.masonry,
        unit_px: theme.grid.row_unit.max(1),
        gap_px,
        container,
    };

    let context = use_context_provider(|| GridZoneContext {
        state: Signal::new(state),
    });
    if *context.state.peek() != state {
        let mut signal = context.state;
        signal.set(state);
    }

    // Always published, never only on override: `gap`, the item's
    // `margin-bottom` and the zone's cancelling margin all read this one var,
    // and a nested zone must not inherit its parent's.
    // Rendered once per change: building the three vars was ~0.45x `Leaf`.
    let style = use_cache((container, gap), |&(container, gap)| {
        let named = !container.is_empty();
        variables()
            .with(GRID_ZONE_AREA_VAR, named.then(|| container.to_string()))
            .with(GRID_ZONE_GAP, SizeCss::SPACING.value(gap))
            .with(
                GRID_ZONE_CONTAINER_VAR,
                named.then(|| container_name(container)),
            )
            .render()
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("container", !container.is_empty())
        .with("dense", props.dense)
        .with("masonry", props.masonry)
        .into();

    use_box()
        .framework_sx(&GRID_ZONE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(Some(style))
        .prepare()
        .render(
            props.component.copied_or(HtmlTag::Div),
            props.attributes,
            props.children,
        )
}
