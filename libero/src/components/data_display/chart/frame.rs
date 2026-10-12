//! What every chart puts around its drawing: title, toggle legend, summary, and the
//! interactive overlay on the plot.

use dioxus::prelude::*;

use super::{
    ChartColor, ChartHandle, ChartPart, ChartPoint, ChartRange, Series, ValueFormat,
    data::{MarkLineAt, MarkPoint},
    engine::{Drawn, Plot, SeriesExtras, SeriesSpec},
    export::{ExportSource, file_name},
    legend::{ChartLegend, HiddenSeries, LegendEntry, legend_sx},
    overlay::{ChartOverlay, Layout, OverlaySeries, overlay_sx},
    zoom::Zoom,
};
use crate::{
    components::{
        accessibility::{use_announcer, visually_hidden_sx},
        buttons::Button,
        common::{
            ClassList, HtmlTag, Input, Part, Parts, States, Variables, names_itself, variables,
        },
        feedback::Skeleton,
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_resize_fallback},
    localization::{ChartLabels, Formats, fill},
    platform::{ElementApi, SaveOutcome, save_file},
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{
        CHART_ASPECT_RATIO, CHART_LEGEND_GAP, CHART_SWATCH_SIZE, HexColor, LegendPosition, Theme,
        chart_color, chart_palette,
    },
    tokens::Size,
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
        // Its width, not its content's: a loading or empty plot in a flex row keeps its size.
        .width("100%")
        .box_sizing("border-box")
        .min_width("0")
        .selector(ChartPart::Title.selector(), sx().font_weight("600"))
        .selector(ChartPart::Description.selector(), sx().color("text-dimmed"))
        .selector(
            ChartPart::Toolbar.selector(),
            sx().display("flex")
                .flex_wrap("wrap")
                .justify_content("flex-end")
                .gap("xs"),
        )
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
        // A line among bars is keyed by a stroke, the bars by squares.
        .selector(
            "& [data-chart-swatch][data-line]",
            sx().height("3px").border_radius("0"),
        )
        .selector(
            "& [data-chart-swatch][data-line][data-hidden]",
            sx().box_shadow("none").opacity("0.5").background_color("var(--lsx-chart-swatch)"),
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
        // A line's points' shape, as drawn; it wins over the dash.
        .selector(
            "& [data-chart-swatch][data-symbol]",
            sx().height(CHART_SWATCH_SIZE.value())
                .border_radius("0")
                .box_shadow("none")
                .with("background", "var(--lsx-chart-swatch)"),
        )
        .selector(
            "& [data-chart-swatch][data-symbol='circle']",
            sx().border_radius("50%"),
        )
        .selector(
            "& [data-chart-swatch][data-symbol='triangle']",
            sx().with("clip-path", "polygon(50% 0, 100% 100%, 0 100%)"),
        )
        .selector(
            "& [data-chart-swatch][data-symbol='diamond']",
            sx().with("clip-path", "polygon(50% 0, 100% 50%, 50% 100%, 0 50%)"),
        )
        .selector(
            "& [data-chart-swatch][data-symbol][data-hidden]",
            sx().opacity("0.35"),
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
        // At the top and the start the legend comes first in the DOM too, so Tab meets it as
        // drawn; `row` puts the first at the start edge in either direction.
        .when(
            "legend-top",
            sx().selector(ChartPart::Body.selector(), column("column")),
        )
        .when(
            "legend-start",
            sx().selector(ChartPart::Body.selector(), side("row"))
                .container_query(CHART_CONTAINER, NARROW, stacked()),
        )
        .when(
            "legend-end",
            sx().selector(ChartPart::Body.selector(), side("row"))
                .container_query(CHART_CONTAINER, NARROW, stacked()),
        )
        .container(CHART_CONTAINER)
});

const CHART_CONTAINER: &str = "lsx-chart";
/// Under this, a legend beside the plot would leave it too small: it goes above or below.
const NARROW: &str = "(max-width: 300px)";

/// A side legend over or under the plot, in DOM order: the start's above, the end's below.
fn stacked() -> Sx {
    sx().selector(
        ChartPart::Body.selector(),
        sx().flex_direction("column").selector(
            "& > [data-slot='legend']",
            sx().flex_direction("row").justify_content("center"),
        ),
    )
}

/// The drawing fills the plot. Forced colours keep its series' colours, which name them
/// (drawn for a black or white `Canvas`); its ink is drawn in system colours. It runs left
/// to right in a right-to-left page too: there its labels' `start` anchor would flip them.
pub(super) fn drawing_sx() -> Sx {
    sx().width("100%")
        .height("100%")
        .with("direction", "ltr")
        .media(
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
    /// A fresh drawing for the handle and the export buttons.
    pub(super) export: ExportSource,
    pub(super) handle: Option<ChartHandle>,
    pub(super) export_button: bool,
    /// Set on a line or bar chart with `zoom`.
    pub(super) zoom: Option<Zoom>,
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
    pub(super) layout: Layout,
    /// Set, a drag or Shift with the arrows selects a band of x values.
    pub(super) onrange: Option<EventHandler<Option<ChartRange>>>,
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
            export,
            handle,
            export_button,
            zoom,
        } = self;
        if let Some(handle) = handle {
            handle.attach(export.clone(), zoom);
        }
        let held = use_hook(|| CopyValue::new(handle));
        if *held.peek() != handle {
            let mut held = held;
            held.set(handle);
        }
        use_drop(move || {
            if let Some(handle) = held.try_peek().ok().and_then(|handle| *handle) {
                handle.detach();
            }
        });
        let announcer = use_announcer();
        // After the first render, so the region exists when it speaks.
        use_effect(use_reactive!(|(loading,)| {
            match loading {
                true => announcer.say(labels.loading.to_string()),
                false => announcer.clear(),
            }
        }));
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

        let hidden_names = hidden.get();
        // Every series hidden: the plot and the summary say so; the legend stays to show one.
        let all_hidden = !loading
            && !empty
            && !series.is_empty()
            && series
                .iter()
                .all(|series| hidden_names.contains(&series.name));
        let summary = match all_hidden {
            true => vec![labels.all_hidden.to_string()],
            false => summary,
        };
        let title_id = title.as_ref().map(|_| format!("{id}-title"));
        let unnamed = title.is_none() && !names_itself(&attributes);
        let description_id = description.as_ref().map(|_| format!("{id}-description"));
        let summary_id = (!summary.is_empty()).then(|| format!("{id}-summary"));
        let described_by = [description_id, summary_id]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        let entries: Vec<LegendEntry> = series
            .iter()
            .map(|series| LegendEntry {
                name: series.name.clone(),
                color: series.color.clone(),
                dashed: series.dashed,
                symbol: series.symbol,
                line: series.line,
                hidden: hidden_names.contains(&series.name),
            })
            .collect();
        let names: Vec<String> = series.iter().map(|series| series.name.clone()).collect();
        let ontoggle = move |name: String| hidden.toggle(&name, &names);
        let legend_element = (legend != LegendPosition::None && !loading && !empty)
            .then(|| rsx! { ChartLegend { entries, label: legend_label, ontoggle } });
        // Drawn first at the top or the start, so first in the DOM there.
        let (legend_before, legend_after) = match legend {
            LegendPosition::Top | LegendPosition::Start => (legend_element, None),
            _ => (None, legend_element),
        };
        let svg = drawn.read().as_ref().map(|drawn| drawn.svg.clone());
        let center_style = drawn.read().as_ref().map(|drawn| {
            let geometry = &drawn.geometry;
            center_style(geometry.plot, geometry.width, geometry.height)
        });
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
            layout,
            onrange,
        } = interaction;
        let tools = !loading && !empty && !all_hidden && (zoom.is_some() || export_button);
        let save = move |extension: &'static str| {
            let export = export.clone();
            let name = file_name(export.title.as_deref(), extension);
            move |_| {
                let (export, name) = (export.clone(), name.clone());
                spawn(async move {
                    let outcome = match extension {
                        "png" => match export.png().await {
                            Some(png) => save_file(&name, "image/png", png).await,
                            None => SaveOutcome::Failed(String::new()),
                        },
                        _ => match export.svg() {
                            Some((svg, ..)) => {
                                save_file(&name, "image/svg+xml", svg.into_bytes()).await
                            }
                            None => SaveOutcome::Failed(String::new()),
                        },
                    };
                    match outcome {
                        SaveOutcome::Saved | SaveOutcome::Shared => {
                            announcer.say(fill(labels.saved, &[("name", &name)]));
                        }
                        SaveOutcome::Failed(_) => {
                            announcer.say(fill(labels.save_failed, &[("name", &name)]));
                        }
                        SaveOutcome::Cancelled => {}
                    }
                });
            }
        };

        let children = rsx! {
            if let Some(title) = title {
                div { "data-slot": ChartPart::Title.slot(), id: "{id}-title", "{title}" }
            }
            if let Some(description) = description {
                div { "data-slot": ChartPart::Description.slot(), id: "{id}-description", "{description}" }
            }
            if tools {
                div { "data-slot": ChartPart::Toolbar.slot(),
                    if let Some(zoom) = zoom {
                        Button {
                            variant: "standard",
                            size: Size::Sm,
                            disabled: !zoom.is_zoomed(),
                            focusable_when_disabled: true,
                            "data-chart-tool": "reset",
                            onclick: move |_| zoom.reset(),
                            "{labels.reset_zoom}"
                        }
                    }
                    if export_button {
                        Button {
                            variant: "standard",
                            size: Size::Sm,
                            "data-chart-tool": "svg",
                            onclick: save("svg"),
                            "{labels.save_svg}"
                        }
                        Button {
                            variant: "standard",
                            size: Size::Sm,
                            "data-chart-tool": "png",
                            onclick: save("png"),
                            "{labels.save_png}"
                        }
                    }
                }
            }
            div { "data-slot": ChartPart::Body.slot(),
                {legend_before}
                div {
                    "data-slot": ChartPart::Plot.slot(),
                    onmounted: element.mount(),
                    onresize: move |event| plot.onresize.call(event),
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
                    } else if all_hidden {
                        div { "data-slot": ChartPart::Empty.slot(), "{labels.all_hidden}" }
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
                            layout,
                            zoom,
                            onrange,
                        }
                    }
                }
                {legend_after}
            }
            // Says that the chart loads and what the save buttons did; the plot's own speaks for the plot.
            if loading || tools && export_button {
                {announcer.render()}
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
            .attr("aria-label", unnamed.then_some(labels.chart))
            .attr(
                "aria-describedby",
                (!described_by.is_empty()).then_some(described_by),
            )
            .attr("aria-busy", loading.then_some("true"))
            .render(HtmlTag::Div, attributes, children)
    }
}

/// A donut's centre: a circle a gap inside `room` (the hole, or a half ring's half of it),
/// its content in the square within, its text shrinking with a small hole. In percent of
/// the drawing's `width` and `height`.
fn center_style(room: Plot, width: f32, height: f32) -> String {
    let diameter = (room.right - room.left - 2.0 * CENTER_GAP).max(0.0);
    let percent = |px: f32| px / width.max(1.0) * 100.0;
    let padding = diameter * (1.0 - std::f32::consts::FRAC_1_SQRT_2) / 2.0;
    let font = diameter * 0.16;
    format!(
        "left:{:.2}%;top:{:.2}%;width:{:.2}%;padding:{:.2}%;font-size:clamp(0.75rem,{font:.1}px,1em)",
        percent((room.left + room.right) / 2.0),
        (room.top + room.bottom) / 2.0 / height.max(1.0) * 100.0,
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
    chart_unit_of(unit, series.iter().map(|series| &series.unit))
}

/// [`chart_unit`] over each series' own unit.
pub(super) fn chart_unit_of<'a>(
    unit: &Option<String>,
    mut units: impl Iterator<Item = &'a Option<String>> + Clone,
) -> Option<String> {
    unit.clone().or_else(|| {
        let shared = units.clone().next()?.as_ref()?;
        units
            .all(|unit| unit.as_ref() == Some(shared))
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
            extras: SeriesExtras {
                symbol: series.symbol.unwrap_or_default(),
                mark_lines: Vec::new(),
                mark_points: Vec::new(),
                secondary: series.secondary_axis,
                line: false,
            },
        })
        .collect()
}

/// Every series' name, colour and own unit, for the legend and the tooltip; with
/// `symbols`, the shape of its points too.
pub(super) fn overlay_series(
    specs: &[SeriesSpec],
    series: &[Series],
    symbols: bool,
) -> Vec<OverlaySeries> {
    specs
        .iter()
        .enumerate()
        .map(|(i, spec)| OverlaySeries {
            name: spec.name.clone(),
            color: spec.color.to_string(),
            dashed: spec.dashed,
            symbol: symbols.then_some(spec.extras.symbol),
            line: spec.extras.line,
            unit: series.get(i).and_then(|series| series.unit.clone()),
            errors: series
                .get(i)
                .map(|series| series.errors.clone())
                .unwrap_or_default(),
            xs: Vec::new(),
            points: Vec::new(),
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
    match x {
        [] => {}
        [only] => sentences.push(fill(labels.range_one, &[("kind", &kind), ("first", only)])),
        [first, .., last] => sentences.push(fill(
            labels.range,
            &[("kind", &kind), ("first", first), ("last", last)],
        )),
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

/// Each series' mark lines with a value to sit at: the value, the line's name and the value
/// as written.
pub(super) fn mark_lines(
    labels: &ChartLabels,
    series: &[Series],
    write: impl Fn(&Series, f64) -> String,
) -> Vec<Vec<(f64, String, String)>> {
    series
        .iter()
        .map(|series| {
            series
                .mark_lines
                .iter()
                .filter_map(|line| {
                    let value = line.resolve(&series.values)?;
                    let name = line.label.clone().unwrap_or_else(|| {
                        match line.at {
                            MarkLineAt::Average => labels.average,
                            MarkLineAt::Min => labels.lowest,
                            MarkLineAt::Max => labels.highest,
                            MarkLineAt::Value(_) => labels.mark,
                        }
                        .to_string()
                    });
                    Some((value, name, write(series, value)))
                })
                .collect()
        })
        .collect()
}

/// Hands the specs their mark lines' values and texts, and returns the summary's sentences.
pub(super) fn mark_line_specs(
    labels: &ChartLabels,
    specs: &mut [SeriesSpec],
    series: &[Series],
    write: impl Fn(&Series, f64) -> String,
) -> Vec<String> {
    let lines = mark_lines(labels, series, write);
    let mut sentences = Vec::new();
    for ((spec, series), lines) in specs.iter_mut().zip(series).zip(lines) {
        for (value, label, written) in lines {
            let text = fill(labels.mark_line, &[("label", &label), ("value", &written)]);
            sentences.push(fill(
                labels.mark_line_summary,
                &[
                    ("name", &series.name),
                    ("label", &label),
                    ("value", &written),
                ],
            ));
            spec.extras.mark_lines.push((value, text));
        }
    }
    sentences
}

/// `specs` with the marks handed to `marked`, for a second palette of the same series.
pub(super) fn with_extras(specs: Vec<SeriesSpec>, marked: &[SeriesSpec]) -> Vec<SeriesSpec> {
    specs
        .into_iter()
        .zip(marked)
        .map(|(spec, marked)| SeriesSpec {
            extras: marked.extras.clone(),
            ..spec
        })
        .collect()
}

/// Hands the specs each marked point's place and value as written, and returns the
/// summary's sentences. The first of equal values is marked.
pub(super) fn mark_point_specs(
    labels: &ChartLabels,
    specs: &mut [SeriesSpec],
    series: &[Series],
    x: &[String],
    write: impl Fn(&Series, f64) -> String,
) -> Vec<String> {
    let mut sentences = Vec::new();
    for (spec, series) in specs.iter_mut().zip(series) {
        let values = || {
            series
                .values
                .iter()
                .enumerate()
                .filter_map(|(i, value)| Some((i, (*value)?)).filter(|(_, v)| v.is_finite()))
        };
        for point in &series.mark_points {
            let (label, found) = match point {
                MarkPoint::Min => (
                    labels.lowest,
                    values().reduce(|a, b| if b.1 < a.1 { b } else { a }),
                ),
                MarkPoint::Max => (
                    labels.highest,
                    values().reduce(|a, b| if b.1 > a.1 { b } else { a }),
                ),
            };
            let Some((index, value)) = found else {
                continue;
            };
            let written = write(series, value);
            let at = x.get(index).cloned().unwrap_or_default();
            sentences.push(fill(
                labels.mark_point,
                &[
                    ("name", &series.name),
                    ("x", &at),
                    ("value", &written),
                    ("label", &label),
                ],
            ));
            spec.extras.mark_points.push((index, written));
        }
    }
    sentences
}

/// A sentence per series with error ranges, as a line's band or a bar's error bars, with
/// their outer ends.
pub(super) fn band_summary(
    labels: &ChartLabels,
    series: &[Series],
    write: impl Fn(&Series, f64) -> String,
) -> Vec<String> {
    series
        .iter()
        .filter_map(|series| {
            let ranges = series
                .errors
                .iter()
                .zip(&series.values)
                .filter_map(|(range, value)| value.and(*range));
            let low = ranges.clone().map(|range| range.0).reduce(f64::min)?;
            let high = ranges.map(|range| range.1).reduce(f64::max)?;
            Some(fill(
                labels.band,
                &[
                    ("name", &series.name),
                    ("low", &write(series, low)),
                    ("high", &write(series, high)),
                ],
            ))
        })
        .collect()
}

/// The axes' titles, which the drawing hides with itself.
pub(super) fn axis_titles(
    labels: &ChartLabels,
    x: &Option<String>,
    y: &Option<String>,
    y2: &Option<String>,
) -> Vec<String> {
    [(labels.x_axis, x), (labels.y_axis, y), (labels.y2_axis, y2)]
        .into_iter()
        .filter_map(|(label, title)| Some(fill(label, &[("title", title.as_ref()?)])))
        .collect()
}

/// The plot's size, snapped, once it is measured: by its resizes, and at mount in case
/// the first resize comes late.
#[derive(Clone, Copy)]
pub(super) struct PlotSize {
    pub(super) element: ElementHandle,
    size: Signal<Option<(f32, f32)>>,
    /// Unsnapped, in CSS px: the overlay maps the pointer with it.
    pub(super) box_size: Signal<Option<(f64, f64)>>,
    /// The plot's `onresize`: the web fires it, Blitz gets it from the fallback.
    onresize: Callback<Event<ResizeData>>,
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
    let onresize = use_callback(move |event: Event<ResizeData>| {
        if let Ok(border) = event.get_border_box_size() {
            measured(border.width, border.height);
        }
    });
    use_resize_fallback(element, move |event| onresize.call(event));
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
        onresize,
    }
}
