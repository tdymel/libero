use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, variables},
        layout::use_box,
    },
    sx::{StaticSx, sx},
    theme::{GRID_ITEM_ROW_SPAN_VAR, GRID_ITEM_ROWS_VAR},
    utils::warn,
};

use super::{GRID_ITEM_STATE, GridSpan, GridZoneContext, SpanValue, ZoneState, container_name};

static GRID_ITEM_SX: StaticSx = StaticSx::new(|| {
    let base = sx().min_width("0").when(
        ROWS_STATE,
        // (0,2,0), against the zone's (0,3,0) masonry rule on the same
        // property. The token is never emitted inside a masonry zone, so the
        // two do not meet - the specificity is the second guard, not the
        // first.
        sx().grid_row(format!("span {}", GRID_ITEM_ROW_SPAN_VAR.value_or("1"))),
    );

    GridSpan::ALL.iter().fold(base, |base, span| {
        base.when(
            span.state_name(),
            sx().grid_column(format!("span {}", span.columns())),
        )
    })
});

/// Set when the caller named `rows` **and** the zone is not a masonry one.
const ROWS_STATE: &str = "rows";

/// Row units an item of `height_px` spans, counting the gap that follows it.
/// `unit_px` is clamped to 1 by the zone that hands it over.
fn rows_spanned(height_px: f64, unit_px: u32, gap_px: u32) -> u32 {
    let height = height_px.max(0.0) + f64::from(gap_px);
    (height / f64::from(unit_px)).ceil().max(1.0) as u32
}

base_props! {
    pub struct GridItemProps {
        children: Element,
        /// Width, in twelfths of the zone. A `GridSpan`, or `sp()` for a span
        /// that changes with the *zone's* width.
        #[props(default, into)]
        span: Input<SpanValue>,
        /// Height, in rows of the zone's implicit grid.
        ///
        /// **Ignored in a masonry zone**, with a warn: there the row span is
        /// derived from the item's measured height, and honouring a manual one
        /// would leave the item overlapping its neighbours. One property, one
        /// writer.
        #[props(default, into)]
        rows: Input<u8>,
        #[props(default, into)]
        component: Input<HtmlTag>,
    }
}

/// One cell of a [`GridZone`](super::GridZone).
#[component]
pub fn GridItem(props: GridItemProps) -> Element {
    let zone = try_use_context::<GridZoneContext>();
    // Every item keeps its own span, so a late-mounted or never-measured item
    // costs exactly itself - there is no zone-wide invariant to break.
    let mut rows = use_signal(|| None::<u32>);

    if zone.is_none() {
        warn("GridItem: used outside a GridZone - it will not be placed.");
    }

    let masonry = zone.is_some_and(|zone| zone.state.read().masonry);

    // `ResizeObserver` delivers an initial observation for every element that
    // has a box, so this is also the mount-time measurement - no id, no
    // element lookup. An element with no box (`display: none`) never
    // reports and simply stays unmeasured.
    let onresize = masonry.then_some(move |event: Event<ResizeData>| {
        let Some(zone) = zone else { return };
        let Ok(size) = event.get_border_box_size() else {
            return;
        };
        let ZoneState {
            unit_px, gap_px, ..
        } = *zone.state.peek();
        let next = rows_spanned(size.height, unit_px, gap_px);

        // Integer quantisation damps sub-pixel jitter; the explicit no-op
        // is what keeps a container feedback loop (item height -> zone
        // height -> a scrollbar appears -> width changes) from becoming a
        // render loop.
        if *rows.peek() != Some(next) {
            rows.set(Some(next));
        }
    });

    let measured = rows.read().filter(|_| masonry);

    // A caller's own span, which the masonry engine takes precedence over by
    // simply not letting it exist: the two never write `grid-row` at once.
    if masonry && props.rows.as_ref().is_some() {
        warn(
            "GridItem: rows is ignored in a masonry zone, which derives the row span from the measured height.",
        );
    }
    let row_span = props
        .rows
        .as_ref()
        .copied()
        .filter(|_| !masonry)
        .map(|rows| rows.max(1));

    let variables: Input<Variables> = variables()
        .with(GRID_ITEM_ROWS_VAR, measured.map(|rows| rows.to_string()))
        .with(
            GRID_ITEM_ROW_SPAN_VAR,
            row_span.map(|rows| rows.to_string()),
        )
        .into();

    let span = props.span.unwrap_or_default();
    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(GRID_ITEM_STATE, true)
        .with(span.base_span().state_name(), true)
        .with("measured", measured.is_some())
        .with(ROWS_STATE, row_span.is_some())
        .into();

    // The base span rides the recycled framework class; only the breakpoints
    // need a sheet of their own, and only an item that has any pays for one.
    // They go in the user layer because the framework layer's base rule wins
    // on specificity otherwise, and *before* `props.sx` so a caller's own
    // `grid-column` still overrides them.
    let sx = match zone.map(|zone| zone.state.peek().container).unwrap_or("") {
        "" => {
            if zone.is_some() && span.breakpoints().next().is_some() {
                warn(
                    "GridItem: sp() breakpoints are ignored in a GridZone without an area, which has no container to query.",
                );
            }
            props.sx.clone()
        }
        zone_container => span
            .breakpoints()
            .fold(sx(), |base, (size, span)| {
                base.container_breakpoint(
                    container_name(zone_container),
                    size,
                    sx().grid_column(format!("span {}", span.columns())),
                )
            })
            .and(props.sx.clone().unwrap_or_default())
            .into(),
    };

    use_box()
        .framework_sx(&GRID_ITEM_SX)
        .class(&props.class)
        .sx(&sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .event("onresize", onresize)
        .render(
            props.component.copied_or(HtmlTag::Div),
            props.attributes,
            props.children,
        )
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    use super::rows_spanned;
    use crate::{
        LiberoProvider,
        components::{GridItem, GridSpan, GridZone, sp},
        utils::take_warnings,
    };

    fn warnings_of(app: fn() -> Element) -> Vec<String> {
        take_warnings();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        take_warnings()
    }

    #[test]
    fn breakpoints_in_a_zone_without_an_area_warn() {
        let dropped = warnings_of(|| {
            rsx! {
                LiberoProvider {
                    GridZone { GridItem { span: sp().base(GridSpan::Full).md(GridSpan::Half), "x" } }
                }
            }
        });
        let plain = warnings_of(|| {
            rsx! {
                LiberoProvider { GridZone { GridItem { span: GridSpan::Half, "x" } } }
            }
        });

        assert!(
            dropped.iter().any(|w| w.contains("sp() breakpoints")),
            "{dropped:?}"
        );
        assert!(plain.is_empty(), "{plain:?}");
    }

    #[test]
    fn an_item_spans_the_rows_it_covers_plus_its_gap() {
        // 30px tall plus a 10px gap, over a 2px unit.
        assert_eq!(rows_spanned(30.0, 2, 10), 20);
        // A partial unit rounds up, or the next item overlaps this one.
        assert_eq!(rows_spanned(31.0, 2, 10), 21);
        // Never zero, however small: an unplaced item would collapse the flow.
        assert_eq!(rows_spanned(0.0, 2, 0), 1);
    }
}
