use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, variables},
        layout::use_box,
    },
    sx::{StaticSx, Sx, sx},
    theme::GRID_ITEM_ROWS_VAR,
    utils::warn,
};

use super::{GRID_ITEM_STATE, GridSpan, GridZoneContext, ZoneState};

static GRID_ITEM_SX: StaticSx = StaticSx::new(|| {
    let base = sx().min_width("0");

    GridSpan::ALL.iter().fold(base, |base, span| {
        base.when(
            span.state_name(),
            sx().grid_column(format!("span {}", span.columns())),
        )
    })
});

/// Row units an item of `height_px` spans, counting the gap that follows it.
/// `unit_px` is clamped to 1 by the zone that hands it over.
fn rows_spanned(height_px: f64, unit_px: u32, gap_px: u32) -> u32 {
    let height = height_px.max(0.0) + f64::from(gap_px);
    (height / f64::from(unit_px)).ceil().max(1.0) as u32
}

base_props! {
    pub struct GridItemProps {
        children: Element,
        /// Width, in twelfths of the zone.
        #[props(default, into)]
        span: Input<GridSpan>,
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
    // `dom_api()` lookup. An element with no box (`display: none`) never
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
    let variables: Input<Variables> = variables()
        .with(GRID_ITEM_ROWS_VAR, measured.map(|rows| rows.to_string()))
        .into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(GRID_ITEM_STATE, true)
        .with(props.span.copied_or_default().state_name(), true)
        .with("measured", measured.is_some())
        .into();

    use_box()
        .framework_sx(&GRID_ITEM_SX)
        .class(&props.class)
        .sx(&props.sx)
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
    use super::rows_spanned;

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
