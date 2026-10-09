use std::{cell::Cell, rc::Rc};

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, ScaleOrCss, States, as_length, base_props, css_px, variables},
        layout::use_box,
    },
    hooks::{use_cache, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        GRID_ITEM_ROWS_VAR, GRID_ROW_UNIT, GRID_ZONE_AREA_VAR, GRID_ZONE_CONTAINER_VAR,
        GRID_ZONE_GAP, SizeCss,
    },
    utils::warn,
};

use super::{AreaName, GridContext};

/// What a `GridItem` needs from its zone. Signal-backed: both context hooks
/// run once, so a plain value would freeze at the first render.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ZoneState {
    pub masonry: bool,
    /// The masonry row quantum in px; an item can't resolve it or the gap itself.
    pub unit_px: u32,
    pub gap_px: u32,
    /// The zone's `@container` name; empty when the zone has no area.
    pub container: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct GridZoneContext {
    pub state: Signal<ZoneState>,
}

/// Only `GridItem` writes this token; bare `[data-state]` would crush any
/// `Box`-derived child, e.g. a stray `Text`, into one row.
pub(crate) const GRID_ITEM_STATE: &str = "grid-item";

/// Prefixed, so it can't collide with a caller's own container name.
pub(crate) fn container_name(area: &str) -> String {
    format!("lsx-zone-{area}")
}

static GRID_ZONE_SX: StaticSx = StaticSx::new(|| {
    let item = format!("& > [data-state~=\"{GRID_ITEM_STATE}\"]");
    let measured = format!("{item}[data-state~=\"measured\"]");

    let gap = GRID_ZONE_GAP.overridable();
    sx().display("grid")
        // Set per breakpoint by an `ImageList`; a nested zone must not inherit it.
        .var(GRID_ZONE_GAP.override_var(), "initial")
        .grid_area(GRID_ZONE_AREA_VAR.value_or("auto"))
        .container_name(GRID_ZONE_CONTAINER_VAR.value_or("none"))
        .grid_template_columns("repeat(12, minmax(0, 1fr))")
        .align_content("start")
        // The track's `minmax(0, 1fr)` protects the track, not the zone in it.
        .min_width("0")
        // Eleven length gaps can't shrink and overflow a narrow zone; the `4%`
        // cap engages only under ~300px.
        .column_gap(format!("min({gap}, 4%)"))
        .row_gap(gap.clone())
        // Only a named zone: containment collapses a shrink-to-fit standalone
        // zone to nothing.
        .when("container", sx().container_type("inline-size"))
        .when("dense", sx().grid_auto_flow("row dense"))
        .when(
            "masonry",
            sx()
                // A row gap would make the quantum ragged; the item carries it,
                // and the zone cancels the last one.
                .row_gap("0")
                .margin_bottom(format!("calc(-1 * {gap})"))
                // `minmax`: an unmeasured item (SSR, native) grows its track
                // instead of overflowing onto the items below.
                .grid_auto_rows(format!("minmax({}, auto)", GRID_ROW_UNIT.value()))
                .selector(
                    &item,
                    // At `stretch` each measurement reports its row span and
                    // the span climbs. Masonry only, or cards lose equal heights.
                    sx().align_self("start").margin_bottom(gap),
                )
                // (0,3,0) beats the item's (0,2,0) `grid-column` rule: framework
                // sheets come in hash order, so `& > *` would lose.
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
        /// Which of the parent `Grid`'s areas this fills; omit it outside a `Grid`.
        #[props(default, into)]
        area: AreaName,
        /// Backfill gaps a wider item left behind.
        #[props(default)]
        dense: bool,
        /// Pack measured item heights with no vertical dead space.
        #[props(default)]
        masonry: bool,
        /// Between items: a size word or any CSS, as `gap: "12px"`. Masonry packs by a
        /// `px` or `rem` length; another CSS value packs by the theme's gap.
        #[props(default, into)]
        gap: Input<ThemeAwareValue>,
        #[props(default, into)]
        component: Input<HtmlTag>,
    }
}

/// A twelve-column packing container, filling one named area of a
/// [`Grid`](super::Grid) or standing on its own.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{GridItem, GridSpan, GridZone};
/// # fn app() -> Element {
/// rsx! {
///     GridZone { masonry: true,
///         GridItem { span: GridSpan::Half, "One" }
///         GridItem { span: GridSpan::Half, "Two" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/grid>
#[component]
pub fn GridZone(props: GridZoneProps) -> Element {
    let theme = use_theme();
    let grid = try_use_context::<GridContext>();
    // Read before this zone provides its own: the zone enclosing a nested grid.
    let enclosing = try_use_context::<GridZoneContext>();
    let warned = use_hook(|| Rc::new(Cell::new(false)));

    let area = props.area;
    if grid.is_none() && area.is_set() {
        warn(&format!(
            "GridZone: area `{}` is set outside a Grid; its container containment can collapse \
             the zone to zero width. Omit `area` for a standalone zone.",
            area.name
        ));
    } else if let Some(grid) = grid.as_ref()
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

    let gap = ScaleOrCss::new(props.gap.as_ref(), theme.grid.zone_gap);
    let custom = gap.custom_css(SizeCss::SPACING);
    let measured = custom.as_deref().and_then(css_px);
    if props.masonry
        && let Some(css) = custom.as_deref().filter(|_| measured.is_none())
        && !warned.replace(true)
    {
        warn(&format!(
            "GridZone: masonry packs by a px or rem gap, so `{css}` packs by the theme's gap \
             and the rows are off by the difference."
        ));
    }
    let gap_px = measured.map_or(theme.spacing.get(gap.size).into(), |px| px.round() as u32);
    let container = if area.is_set() { area.name } else { "" };
    // A nested zone reusing the area name would capture the outer zone's
    // responsive spans.
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

    // Always published: a nested zone must not inherit its parent's gap.
    // Cached per change: building the three vars was ~0.45x `Leaf`.
    // The zone's and an image list's `calc()`s need a length.
    let gap_css = as_length(gap.resolve(SizeCss::SPACING));
    let style = use_cache((container, gap_css), |(container, gap_css)| {
        let named = !container.is_empty();
        variables()
            .with(GRID_ZONE_AREA_VAR, named.then(|| container.to_string()))
            .with(GRID_ZONE_GAP, gap_css.clone())
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

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    use crate::{
        LiberoProvider,
        components::layout::{GridArea, GridZone},
        utils::warnings_of,
    };

    #[derive(Clone, Copy, PartialEq)]
    struct Body;
    impl GridArea for Body {
        fn name(&self) -> &'static str {
            "body"
        }
    }

    #[test]
    fn an_area_outside_a_grid_warns() {
        let orphan = warnings_of(|| rsx! { LiberoProvider { GridZone { area: Body, "x" } } });
        let plain = warnings_of(|| rsx! { LiberoProvider { GridZone { "x" } } });

        assert!(
            orphan.iter().any(|w| w.contains("outside a Grid")),
            "{orphan:?}"
        );
        assert!(plain.is_empty(), "{plain:?}");
    }

    #[test]
    fn masonry_warns_once_a_gap_it_cannot_measure() {
        let calc = warnings_of(|| {
            rsx! { LiberoProvider { GridZone { masonry: true, gap: "calc(1rem + 1px)", "x" } } }
        });
        let px = warnings_of(
            || rsx! { LiberoProvider { GridZone { masonry: true, gap: "12px", "x" } } },
        );

        assert_eq!(
            calc.iter()
                .filter(|w| w.contains("packs by the theme's gap"))
                .count(),
            1,
            "{calc:?}"
        );
        assert!(px.is_empty(), "{px:?}");
    }
}
