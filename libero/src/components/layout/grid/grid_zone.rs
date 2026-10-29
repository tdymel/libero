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
        GRID_ZONE_GAP, GRID_ZONE_MASONRY_GAP, SizeCss,
    },
    utils::use_warn_once,
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
    let masonry_gap = GRID_ZONE_MASONRY_GAP.value_or(&gap);
    sx().display("grid")
        // Set per breakpoint by an `ImageList`; a nested zone must not inherit it.
        .var(GRID_ZONE_GAP.override_var(), "initial")
        .var(GRID_ZONE_MASONRY_GAP, "initial")
        .grid_area(GRID_ZONE_AREA_VAR.value_or("auto"))
        .container_name(GRID_ZONE_CONTAINER_VAR.value_or("none"))
        .grid_template_columns("repeat(12, minmax(0, 1fr))")
        .align_content("start")
        // The track's `minmax(0, 1fr)` protects the track, not the zone in it.
        .min_width("0")
        // Eleven length gaps can't shrink and overflow a narrow zone. The `4%` cap
        // engages below ~300px at `md` and ~600px at `xl`, so on a phone; a row gap's
        // `%` resolves against the height, so only masonry caps it, from a measured width.
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
                .margin_bottom(format!("calc(-1 * {masonry_gap})"))
                // `minmax`: an unmeasured item (SSR, native) grows its track
                // instead of overflowing onto the items below.
                .grid_auto_rows(format!("minmax({}, auto)", GRID_ROW_UNIT.value()))
                .selector(
                    &item,
                    // At `stretch` each measurement reports its row span and
                    // the span climbs. Masonry only, or cards lose equal heights.
                    sx().align_self("start").margin_bottom(masonry_gap),
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
        /// `px` or `rem` length, counting a `rem` as 16px; another CSS value packs by the
        /// theme's gap. The column gap shrinks to 4% of the zone's width, and a masonry zone's
        /// vertical gap with it; a plain row gap keeps its size.
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
    let warned = use_warn_once();
    // 4% of the zone's width in px, as the column gap's cap; masonry only.
    let mut gap_cap = use_signal(|| None::<u32>);

    let area = props.area;
    if grid.is_none() && area.is_set() {
        warned.once("orphan-area", || {
            format!(
                "GridZone: area `{}` is set outside a Grid; its container containment can collapse \
                 the zone to zero width. Omit `area` for a standalone zone.",
                area.name
            )
        });
    } else if let Some(grid) = grid.as_ref()
        && area.is_set()
        && !grid.template.read().contains(&area)
    {
        if grid.template.read().0.source == area.source {
            warned.once("unknown-area", || {
                format!(
                    "GridZone: area `{}` is not in the parent Grid's template.",
                    area.name
                )
            });
        } else {
            warned.once("other-enum", || {
                format!(
                    "GridZone: area `{}` comes from a different GridArea enum than the parent \
                     Grid's template.",
                    area.name
                )
            });
        }
    } else if grid.is_some() && !area.is_set() {
        warned.once("no-area", || {
            "GridZone: a zone inside a Grid needs an `area`, or it is placed by auto-flow and \
             ignores the template."
                .to_string()
        });
    }

    let gap = ScaleOrCss::new(props.gap.as_ref(), theme.grid.zone_gap);
    let custom = gap.custom_css(SizeCss::SPACING);
    let measured = custom.as_deref().and_then(css_px);
    if props.masonry
        && let Some(css) = custom.as_deref().filter(|_| measured.is_none())
    {
        warned.once("masonry-gap", || {
            format!(
                "GridZone: masonry packs by a px or rem gap, so `{css}` packs by the theme's gap \
                 and the rows are off by the difference."
            )
        });
    }
    let full_gap_px = measured.map_or(theme.spacing.get(gap.size).into(), |px| px.round() as u32);
    let gap_px = match *gap_cap.read() {
        Some(cap) if props.masonry => full_gap_px.min(cap),
        _ => full_gap_px,
    };
    // Only a capped gap is written: at full size the CSS keeps the caller's own value.
    let masonry_gap = (gap_px < full_gap_px).then(|| format!("{gap_px}px"));
    let onresize = props.masonry.then_some(move |event: Event<ResizeData>| {
        let Ok(size) = event.get_content_box_size() else {
            return;
        };
        let cap = Some((size.width * 0.04).round() as u32);
        if *gap_cap.peek() != cap {
            gap_cap.set(cap);
        }
    });
    let container = if area.is_set() { area.name } else { "" };
    // A nested zone reusing the area name would capture the outer zone's
    // responsive spans.
    if !container.is_empty()
        && enclosing.is_some_and(|zone| zone.state.peek().container == container)
    {
        warned.once("nested-area", || {
            format!(
                "GridZone: a nested zone reuses the area name `{container}`, so a responsive \
                 GridItem span in the outer zone would resolve against the inner one."
            )
        });
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
    let style = use_cache(
        (container, gap_css, masonry_gap),
        |(container, gap_css, masonry_gap)| {
            let named = !container.is_empty();
            variables()
                .with(GRID_ZONE_AREA_VAR, named.then(|| container.to_string()))
                .with(GRID_ZONE_GAP, gap_css.clone())
                .with(GRID_ZONE_MASONRY_GAP, masonry_gap.clone())
                .with(
                    GRID_ZONE_CONTAINER_VAR,
                    named.then(|| container_name(container)),
                )
                .render()
        },
    );

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
        .event("onresize", onresize)
        .render(
            props.component.copied_or(HtmlTag::Div),
            props.attributes,
            props.children,
        )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use dioxus::prelude::*;

    use crate::{
        LiberoProvider,
        components::layout::{Grid, GridArea, GridItem, GridTemplate, GridZone},
        utils::{take_warnings, warnings_of},
    };

    static RENDERS: AtomicUsize = AtomicUsize::new(0);

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

    /// Todo 4644: a re-render with the same misuse does not warn again.
    #[test]
    fn a_misuse_warns_once_across_renders() {
        fn app() -> Element {
            // A new class each render, so every component re-renders with the same misuse.
            let class = format!("n{}", RENDERS.fetch_add(1, Ordering::Relaxed));
            let template = GridTemplate::new()
                .row(|row| row.cell(Body))
                .build()
                .unwrap();
            rsx! {
                LiberoProvider {
                    Grid { template,
                        GridZone { class: class.clone(), GridItem { class: class.clone(), "x" } }
                    }
                    GridItem { class, "orphan" }
                }
            }
        }
        take_warnings();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        for _ in 0..2 {
            dom.mark_dirty(ScopeId::APP);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let warnings = take_warnings();
        let count = |needle: &str| warnings.iter().filter(|w| w.contains(needle)).count();
        assert!(RENDERS.load(Ordering::Relaxed) >= 3);
        assert_eq!(count("needs an `area`"), 1, "{warnings:?}");
        assert_eq!(count("outside a GridZone"), 1, "{warnings:?}");
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
