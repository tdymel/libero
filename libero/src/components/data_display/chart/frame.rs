//! What every chart puts around its drawing: title, toggle legend, summary, and the
//! interactive overlay on the plot.

use dioxus::prelude::*;

use super::{
    ChartColor, ChartPart, ChartPoint, Series, ValueFormat,
    engine::{Drawn, SeriesSpec},
    legend::{ChartLegend, HiddenSeries, LegendEntry, legend_sx},
    overlay::{ChartOverlay, OverlaySeries, overlay_sx},
};
use crate::{
    components::{
        accessibility::visually_hidden_sx,
        common::{ClassList, HtmlTag, Input, Part, Parts, States, Variables, variables},
        feedback::Skeleton,
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_resize_fallback},
    localization::{ChartLabels, Formats, fill},
    platform::ElementApi,
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{
        CHART_ASPECT_RATIO, CHART_LEGEND_GAP, CHART_SWATCH_SIZE, HexColor, LegendPosition, Theme,
        chart_color, chart_palette,
    },
};

/// Before the first measure, and on the server: the SVG scales to its box from here.
pub(super) const FIRST_WIDTH: f32 = 640.0;
/// Sizes snap to this, so a drag of the window redraws every few pixels, not every one.
const SIZE_STEP: f32 = 8.0;
/// Between a donut's centre and the ring.
const CENTER_GAP: f32 = 4.0;

static CHART_BASE_SX: StaticSx = StaticSx::new(|| {
    let column = |direction| sx().flex_direction(direction);
    let side = |direction| {
        column(direction).selector(
            "& > [data-slot='legend']",
            sx().flex_direction("column").justify_content("center"),
        )
    };
    sx().display("flex")
        .flex_direction("column")
        .gap("xs")
        // The hidden summary sits inside, not at the page's corner.
        .position("relative")
        .min_width("0")
        .selector(ChartPart::Title.selector(), sx().font_weight("600"))
        .selector(ChartPart::Description.selector(), sx().color("text-dimmed"))
        .selector(
            ChartPart::Body.selector(),
            sx().display("flex")
                .flex_direction("column")
                .gap(CHART_LEGEND_GAP.value())
                .min_width("0"),
        )
        .selector(
            ChartPart::Plot.selector(),
            sx().position("relative")
                .flex("1 1 auto")
                .min_width("0")
                .with("aspect-ratio", CHART_ASPECT_RATIO.value())
                .selector("& > [data-chart-drawing]", drawing_sx())
                .selector("& svg", sx().display("block"))
                .and(overlay_sx()),
        )
        .selector(
            ChartPart::Center.selector(),
            // A circle inside the hole, sized by `center_style`; what does not fit is clipped.
            sx().position("absolute")
                .left("50%")
                .top("50%")
                .transform("translate(-50%, -50%)")
                .with("aspect-ratio", "1")
                .box_sizing("border-box")
                .border_radius("50%")
                .overflow("hidden")
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .text_align("center")
                .line_height("1.2")
                .overflow_wrap("break-word")
                // The slices under it stay the pointer's.
                .pointer_events("none"),
        )
        .selector(
            ChartPart::Legend.selector(),
            sx().display("flex")
                .flex_wrap("wrap")
                .justify_content("center")
                .gap("xs")
                .column_gap("md")
                .margin("0")
                .padding("0")
                .with("list-style", "none")
                .font_size("sm")
                .and(legend_sx()),
        )
        .selector(
            ChartPart::LegendItem.selector(),
            sx().display("inline-flex").align_items("center"),
        )
        .selector(
            "& [data-chart-swatch]",
            sx().flex_shrink("0")
                .width(CHART_SWATCH_SIZE.value())
                .height(CHART_SWATCH_SIZE.value())
                .border_radius("2px")
                .background_color("var(--lsx-chart-swatch)")
                // The swatch is the series' only key: forced colours would blank it.
                .forced_color_adjust("none"),
        )
        .selector(
            "& [data-chart-swatch][data-hidden]",
            sx().background_color("transparent")
                .box_shadow("inset 0 0 0 2px var(--lsx-chart-swatch)"),
        )
        // A dashed line's key is a dash, as drawn.
        .selector(
            "& [data-chart-swatch][data-dashed]",
            sx().height("3px")
                .border_radius("0")
                .box_shadow("none")
                .with(
                    "background",
                    "repeating-linear-gradient(90deg, var(--lsx-chart-swatch) 0 4px, transparent 4px 7px)",
                ),
        )
        .selector(
            "& [data-chart-swatch][data-dashed][data-hidden]",
            sx().opacity("0.5"),
        )
        .selector(
            ChartPart::Empty.selector(),
            sx().position("absolute")
                .with("inset", "0")
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .color("text-dimmed"),
        )
        .selector("& > [data-chart-summary]", visually_hidden_sx())
        .when(
            "legend-top",
            sx().selector(ChartPart::Body.selector(), column("column-reverse")),
        )
        // `row-reverse` puts the legend at the start edge in either direction.
        .when(
            "legend-start",
            sx().selector(ChartPart::Body.selector(), side("row-reverse")),
        )
        .when(
            "legend-end",
            sx().selector(ChartPart::Body.selector(), side("row")),
        )
});

/// The drawing fills the plot. Forced colours keep its series' colours, which name them
/// (drawn for a black or white `Canvas`); its ink is drawn in system colours.
pub(super) fn drawing_sx() -> Sx {
    sx().width("100%").height("100%").media(
        FORCED_COLORS,
        sx().forced_color_adjust("none").background("Canvas"),
    )
}

/// A chart's drawing and everything said about it, ready to render.
pub(super) struct Frame {
    pub(super) id: Signal<String>,
    pub(super) labels: ChartLabels,
    pub(super) title: Option<String>,
    pub(super) description: Option<String>,
    pub(super) loading: bool,
    /// No value at all: `empty_slot`, or the localized "No data".
    pub(super) empty: bool,
    pub(super) empty_slot: Option<Element>,
    pub(super) drawn: Memo<Option<Drawn>>,
    /// Over the drawing's middle, as a donut's centre.
    pub(super) center: Option<Element>,
    pub(super) legend: LegendPosition,
    /// The legend's accessible name: `labels.legend`, a pie's `labels.slices`.
    pub(super) legend_label: &'static str,
    /// Every series, or a pie's slices, hidden ones too: the legend's entries.
    pub(super) series: Vec<OverlaySeries>,
    pub(super) hidden: HiddenSeries,
    /// The x values as written; a pie's slice names.
    pub(super) x: Vec<String>,
    pub(super) summary: Vec<String>,
    pub(super) plot: PlotSize,
    pub(super) aspect_ratio: Option<f32>,
    pub(super) interaction: Interaction,
}

/// What the overlay needs beyond the drawing: how to write values and whom to tell.
pub(super) struct Interaction {
    pub(super) format: ValueFormat,
    pub(super) unit: Option<String>,
    pub(super) formats: &'static Formats,
    pub(super) tooltip: bool,
    pub(super) tooltip_content: Option<Callback<ChartPoint, Element>>,
    pub(super) onpointclick: Option<EventHandler<ChartPoint>>,
    pub(super) onactivechange: Option<EventHandler<Option<ChartPoint>>>,
    /// A horizontal bar chart's: up and down walk the categories, as drawn.
    pub(super) swap_axes: bool,
}

impl Frame {
    /// The legend's place: given, else the theme's once there is more than one entry.
    pub(super) fn legend(
        given: &Input<LegendPosition>,
        theme: &Theme,
        entries: usize,
    ) -> LegendPosition {
        match given.as_ref() {
            Some(position) => *position,
            None if entries > 1 => theme.chart.legend,
            None => LegendPosition::None,
        }
    }

    pub(super) fn render(
        self,
        class: &Input<ClassList>,
        sx: &Input<Sx>,
        parts: &Input<Parts<ChartPart>>,
        states: &Input<States>,
        attributes: Vec<Attribute>,
    ) -> Element {
        let Self {
            id,
            labels,
            title,
            description,
            loading,
            empty,
            empty_slot,
            drawn,
            center,
            legend,
            legend_label,
            series,
            hidden,
            x,
            summary,
            plot,
            aspect_ratio,
            interaction,
        } = self;
        let states: Input<States> = states
            .clone()
            .unwrap_or_default()
            .with(legend_state(legend), legend != LegendPosition::None)
            .into();
        let vars: Input<Variables> = variables()
            .with(
                CHART_ASPECT_RATIO,
                aspect_ratio.map(|ratio| ratio.to_string()),
            )
            .into();

        let title_id = title.as_ref().map(|_| format!("{id}-title"));
        let description_id = description.as_ref().map(|_| format!("{id}-description"));
        let summary_id = (!summary.is_empty()).then(|| format!("{id}-summary"));
        let described_by = [description_id, summary_id]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let hidden_names = hidden.get();
        let entries: Vec<LegendEntry> = series
            .iter()
            .map(|series| LegendEntry {
                name: series.name.clone(),
                color: series.color.clone(),
                dashed: series.dashed,
                hidden: hidden_names.contains(&series.name),
            })
            .collect();
        let names: Vec<String> = series.iter().map(|series| series.name.clone()).collect();
        let ontoggle = move |name: String| hidden.toggle(&name, &names);
        let svg = drawn.read().as_ref().map(|drawn| drawn.svg.clone());
        let center_style = drawn
            .read()
            .as_ref()
            .map(|drawn| center_style(drawn.geometry.hole, drawn.geometry.width));
        let element = plot.element;
        let Interaction {
            format,
            unit,
            formats,
            tooltip,
            tooltip_content,
            onpointclick,
            onactivechange,
            swap_axes,
        } = interaction;

        let children = rsx! {
            if let Some(title) = title {
                div { "data-slot": ChartPart::Title.slot(), id: "{id}-title", "{title}" }
            }
            if let Some(description) = description {
                div { "data-slot": ChartPart::Description.slot(), id: "{id}-description", "{description}" }
            }
            div { "data-slot": ChartPart::Body.slot(),
                div {
                    "data-slot": ChartPart::Plot.slot(),
                    onmounted: element.mount(),
                    ..element.attributes(),
                    if loading {
                        Skeleton { height: "100%" }
                    } else if empty {
                        div { "data-slot": ChartPart::Empty.slot(),
                            match empty_slot {
                                Some(empty) => empty,
                                None => rsx! { "{labels.no_data}" },
                            }
                        }
                    } else if let Some(svg) = svg {
                        // The figure's summary speaks for the drawing.
                        div { "data-chart-drawing": true, "aria-hidden": "true", dangerous_inner_html: "{svg}" }
                        if let Some(center) = center {
                            div { "data-slot": ChartPart::Center.slot(), style: center_style, {center} }
                        }
                        ChartOverlay {
                            drawn,
                            box_size: plot.box_size,
                            series,
                            x,
                            format,
                            unit,
                            formats,
                            labels,
                            title_id: title_id.clone(),
                            keys_id: format!("{id}-keys"),
                            tooltip,
                            tooltip_content,
                            onpointclick,
                            onactivechange,
                            swap_axes,
                        }
                    }
                }
                if legend != LegendPosition::None && !loading && !empty {
                    ChartLegend { entries, label: legend_label, ontoggle }
                }
            }
            if !summary.is_empty() {
                div { "data-chart-summary": true, id: "{id}-summary",
                    for (i, sentence) in summary.into_iter().enumerate() {
                        p { key: "{i}", "{sentence}" }
                    }
                }
            }
        };

        use_box()
            .framework_sx(&CHART_BASE_SX)
            .class(class)
            .sx(sx)
            .parts(parts)
            .states(&states)
            .variables(&vars)
            .prepare()
            // `figure`, not `img`: an image's children are presentational, the summary with them.
            .attr("role", "figure")
            .attr("id", id())
            .attr("aria-labelledby", title_id)
            .attr(
                "aria-describedby",
                (!described_by.is_empty()).then_some(described_by),
            )
            .attr("aria-busy", loading.then_some("true"))
            .render(HtmlTag::Div, attributes, children)
    }
}

/// A donut's centre: a circle a gap inside the hole, its content in the square within,
/// its text shrinking with a small hole. In percent of the plot's `width`, as drawn.
fn center_style(hole: f32, width: f32) -> String {
    let diameter = 2.0 * (hole - CENTER_GAP).max(0.0);
    let percent = |px: f32| px / width.max(1.0) * 100.0;
    let padding = diameter * (1.0 - std::f32::consts::FRAC_1_SQRT_2) / 2.0;
    let font = diameter * 0.16;
    format!(
        "width:{:.2}%;padding:{:.2}%;font-size:clamp(0.75rem,{font:.1}px,1em)",
        percent(diameter),
        percent(padding)
    )
}

fn legend_state(legend: LegendPosition) -> &'static str {
    match legend {
        LegendPosition::Top => "legend-top",
        LegendPosition::Start => "legend-start",
        LegendPosition::End => "legend-end",
        LegendPosition::Bottom | LegendPosition::None => "legend-bottom",
    }
}

pub(super) fn snap(length: f32) -> f32 {
    ((length / SIZE_STEP).round() * SIZE_STEP).max(SIZE_STEP)
}

/// The hex of the `i`th series or slice: its own (a theme colour stepped to 3:1, a hex as
/// given), else the drawn palette's ([`chart_palette`]) for its place.
pub(super) fn hex(
    theme: &Theme,
    forced: bool,
    palette: &[HexColor],
    i: usize,
    color: Option<ChartColor>,
) -> HexColor {
    match color {
        Some(ChartColor::Hex(hex)) => hex,
        Some(ChartColor::Palette(color, shade)) => chart_color(theme, color, shade, forced),
        None => palette[i % palette.len()],
    }
}

/// The chart's unit: its own, else the one every series shares, for the y axis.
pub(super) fn chart_unit(unit: &Option<String>, series: &[Series]) -> Option<String> {
    unit.clone().or_else(|| {
        let shared = series.first()?.unit.as_ref()?;
        series
            .iter()
            .all(|series| series.unit.as_ref() == Some(shared))
            .then(|| shared.clone())
    })
}

/// Each series with its hex; past the palette's end a line repeats a colour dashed.
pub(super) fn series_specs(theme: &Theme, forced: bool, series: &[Series]) -> Vec<SeriesSpec> {
    let palette = chart_palette(theme, forced);
    series
        .iter()
        .enumerate()
        .map(|(i, series)| SeriesSpec {
            name: series.name.clone(),
            values: series.values.clone(),
            color: hex(theme, forced, &palette, i, series.color),
            dashed: series.dashed || (series.color.is_none() && i >= palette.len()),
        })
        .collect()
}

/// Every series' name, colour and own unit, for the legend and the tooltip.
pub(super) fn overlay_series(specs: &[SeriesSpec], series: &[Series]) -> Vec<OverlaySeries> {
    specs
        .iter()
        .enumerate()
        .map(|(i, spec)| OverlaySeries {
            name: spec.name.clone(),
            color: spec.color.to_string(),
            dashed: spec.dashed,
            unit: series.get(i).and_then(|series| series.unit.clone()),
        })
        .collect()
}

/// The places of the specs the legend leaves shown.
pub(super) fn shown_places(specs: &[SeriesSpec], hidden: &[String]) -> Vec<usize> {
    (0..specs.len())
        .filter(|i| !hidden.contains(&specs[*i].name))
        .collect()
}

/// The range of x, then each series' lowest and highest value.
pub(super) fn summary(
    labels: &ChartLabels,
    kind: &str,
    x: &[String],
    series: &[Series],
    write: impl Fn(&Series, f64) -> String,
) -> Vec<String> {
    let mut sentences = Vec::new();
    if let (Some(first), Some(last)) = (x.first(), x.last()) {
        sentences.push(fill(
            labels.range,
            &[("kind", &kind), ("first", first), ("last", last)],
        ));
    }
    for series in series {
        let values = series
            .values
            .iter()
            .enumerate()
            .filter_map(|(i, value)| value.map(|value| (i, value)));
        let min = values.clone().map(|(_, value)| value).reduce(f64::min);
        // The first of equal highs.
        let max = values.reduce(|high, next| if next.1 > high.1 { next } else { high });
        sentences.push(match (min, max) {
            (Some(min), Some((at, max))) => fill(
                labels.series,
                &[
                    ("name", &series.name),
                    ("min", &write(series, min)),
                    ("max", &write(series, max)),
                    ("x", &x.get(at).cloned().unwrap_or_default()),
                ],
            ),
            _ => fill(labels.series_empty, &[("name", &series.name)]),
        });
    }
    sentences
}

/// The plot's size, snapped, once it is measured: by its resizes, and at mount in case
/// the first resize comes late.
#[derive(Clone, Copy)]
pub(super) struct PlotSize {
    pub(super) element: ElementHandle,
    size: Signal<Option<(f32, f32)>>,
    /// Unsnapped, in CSS px: the overlay maps the pointer with it.
    pub(super) box_size: Signal<Option<(f64, f64)>>,
}

impl PlotSize {
    /// The measured size, else [`FIRST_WIDTH`] at `ratio`.
    pub(super) fn get(&self, ratio: f32) -> (f32, f32) {
        self.size
            .read()
            .unwrap_or((FIRST_WIDTH, snap(FIRST_WIDTH / ratio)))
    }
}

pub(super) fn use_plot_size() -> PlotSize {
    let element = use_element();
    let mut size = use_signal(|| None);
    let mut box_size = use_signal(|| None);
    let mut measured = move |width: f64, height: f64| {
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let next = Some((snap(width as f32), snap(height as f32)));
        if *size.peek() != next {
            size.set(next);
        }
        if *box_size.peek() != Some((width, height)) {
            box_size.set(Some((width, height)));
        }
    };
    use_resize_fallback(element, move |event| {
        if let Ok(border) = event.get_border_box_size() {
            measured(border.width, border.height);
        }
    });
    use_effect(move || {
        if !element.is_mounted() {
            return;
        }
        let dimensions = element.dimensions();
        spawn(async move {
            if let Ok(rect) = dimensions.await
                && size.peek().is_none()
            {
                measured(rect.width, rect.height);
            }
        });
    });
    PlotSize {
        element,
        size,
        box_size,
    }
}
