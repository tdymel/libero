//! What every chart puts around its drawing: title, toggle legend, summary, data table,
//! and the interactive overlay on the plot.

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
        CHART_ASPECT_RATIO, CHART_LEGEND_GAP, CHART_SWATCH_SIZE, ChartTable, HexColor,
        LegendPosition, Theme, chart_color, chart_palette,
    },
};

/// Before the first measure, and on the server: the SVG scales to its box from here.
pub(super) const FIRST_WIDTH: f32 = 640.0;
/// Sizes snap to this, so a drag of the window redraws every few pixels, not every one.
const SIZE_STEP: f32 = 8.0;

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
        // The hidden summary and table sit inside, not at the page's corner.
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
            sx().position("absolute")
                .with("inset", "0")
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .text_align("center")
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
        .selector(ChartPart::Table.selector(), visually_hidden_sx())
        .when(
            "table-visible",
            sx().selector(
                ChartPart::Table.selector(),
                sx().position("static")
                    .width("auto")
                    .height("auto")
                    .margin("0")
                    .overflow("auto")
                    .clip("auto")
                    .white_space("normal")
                    .opacity("1"),
            ),
        )
        .selector(
            format!("{} table", ChartPart::Table.selector()),
            sx().border_collapse("collapse").font_size("sm"),
        )
        .selector(
            format!("{} :is(th, td)", ChartPart::Table.selector()),
            sx().padding("4px 8px").text_align("end"),
        )
        // The x labels' column reads as text; the values line up on their ends.
        .selector(
            format!("{} :is(th, td):first-child", ChartPart::Table.selector()),
            sx().text_align("start"),
        )
        .selector(
            format!("{} caption", ChartPart::Table.selector()),
            sx().text_align("start").font_weight("600"),
        )
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

/// The table's column heads after the first, then per row its label and cells.
pub(super) type TableRows = (Vec<String>, Vec<(String, Vec<String>)>);

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
    pub(super) table: ChartTable,
    pub(super) rows: Option<TableRows>,
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

    /// The table, unless it is off, the chart has nothing to show or it is past the cap.
    pub(super) fn rows(
        table: ChartTable,
        drawable: bool,
        cells: usize,
        theme: &Theme,
        rows: impl FnOnce() -> TableRows,
    ) -> Option<TableRows> {
        (table != ChartTable::Off && drawable && cells <= theme.chart.table_cell_cap).then(rows)
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
            table,
            rows,
            plot,
            aspect_ratio,
            interaction,
        } = self;
        let states: Input<States> = states
            .clone()
            .unwrap_or_default()
            .with(legend_state(legend), legend != LegendPosition::None)
            .with("table-visible", table == ChartTable::Visible)
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
        let caption = title.clone().unwrap_or_else(|| labels.table.to_string());
        let hidden_names = hidden.get();
        let entries: Vec<LegendEntry> = series
            .iter()
            .map(|series| LegendEntry {
                name: series.name.clone(),
                color: series.color.clone(),
                hidden: hidden_names.contains(&series.name),
            })
            .collect();
        let names: Vec<String> = series.iter().map(|series| series.name.clone()).collect();
        let ontoggle = move |name: String| hidden.toggle(&name, &names);
        let svg = drawn.read().as_ref().map(|drawn| drawn.svg.clone());
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
                        // The figure's summary and table speak for the drawing.
                        div { "data-chart-drawing": true, "aria-hidden": "true", dangerous_inner_html: "{svg}" }
                        if let Some(center) = center {
                            div { "data-slot": ChartPart::Center.slot(), {center} }
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
            if let Some((head, rows)) = rows {
                div {
                    "data-slot": ChartPart::Table.slot(),
                    // Visible, it scrolls: keyboard users need a stop to scroll it.
                    role: (table == ChartTable::Visible).then_some("region"),
                    tabindex: (table == ChartTable::Visible).then_some("0"),
                    "aria-labelledby": (table == ChartTable::Visible).then(|| format!("{id}-caption")),
                    table {
                        caption { id: "{id}-caption", "{caption}" }
                        thead {
                            tr {
                                th { scope: "col", "{labels.category}" }
                                for (i, name) in head.into_iter().enumerate() {
                                    th { key: "{i}", scope: "col", "{name}" }
                                }
                            }
                        }
                        tbody {
                            for (i, (x, cells)) in rows.into_iter().enumerate() {
                                tr { key: "{i}",
                                    th { scope: "row", "{x}" }
                                    for (j, cell) in cells.into_iter().enumerate() {
                                        td { key: "{j}", "{cell}" }
                                    }
                                }
                            }
                        }
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
            // `figure`, not `img`: an image's children are presentational, the table with them.
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

/// The series' names, then per x value its label and each series' value; a gap is empty.
pub(super) fn table_rows(
    x: &[String],
    series: &[Series],
    write: impl Fn(&Series, f64) -> String,
) -> TableRows {
    let head = series.iter().map(|series| series.name.clone()).collect();
    let rows = x
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let cells = series
                .iter()
                .map(|series| {
                    series
                        .values
                        .get(i)
                        .copied()
                        .flatten()
                        .map(|value| write(series, value))
                        .unwrap_or_default()
                })
                .collect();
            (x.clone(), cells)
        })
        .collect();
    (head, rows)
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
