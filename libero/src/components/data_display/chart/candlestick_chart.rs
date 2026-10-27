use dioxus::prelude::*;

use super::{
    Candle, ChartColor, ChartRange,
    engine::{self, Axes, CandleSpec},
    export::ExportSource,
    frame::{Frame, Interaction, is_named, use_plot_size},
    legend::use_hidden_series,
    overlay::{Layout, OverlaySeries},
    props::chart_props,
    zoom::{cut, use_zoom},
};
use crate::{
    components::common::{Input, input_from_str, use_name_warning},
    hooks::{use_accessibility, use_formats, use_localization, use_root_id, use_theme},
    localization::{ChartLabels, Formats, fill},
    theme::{CandleStyle, ChartInk, Color, LegendPosition, Theme, chart_color},
    utils::warn,
};

input_from_str!(CandleStyle);

chart_props! {
    pub struct CandlestickChartProps {
        /// The periods' names, left to right, evenly spaced.
        x: Vec<String>,
        /// One per period of `x`. A candle with a price that is not finite leaves a gap.
        candles: Vec<Candle>,
        /// The prices' name in the tooltip and the announcement. Unset, "Price".
        #[props(default, into)]
        name: Option<String>,
        /// A rising candle's colour. Unset, `success`.
        #[props(default, into)]
        up_color: Option<ChartColor>,
        /// A falling candle's colour. Unset, `error`.
        #[props(default, into)]
        down_color: Option<ChartColor>,
        /// Candles, or OHLC bars. Unset, candles.
        #[props(default, into)]
        style: Input<CandleStyle>,
        /// The price axis' lowest value. Unset, a round number at or under the lowest low.
        #[props(default)]
        y_min: Option<f64>,
        /// The price axis' highest value. Unset, a round number at or over the highest high.
        #[props(default)]
        y_max: Option<f64>,
        /// About how many steps the price axis is cut into. Unset, the theme's.
        #[props(default)]
        y_ticks: Option<usize>,
        /// Under the periods: what they are.
        #[props(default, into)]
        x_title: Option<String>,
        /// Beside the price axis, read upward: what it measures.
        #[props(default, into)]
        y_title: Option<String>,
        /// Zoom and pan along the periods: Ctrl or Cmd with the wheel, a pinch, a drag
        /// (Shift or Alt with it once `onrange` is set), the plus, minus and 0 keys, and a
        /// reset button. Unset, the theme's.
        #[props(default)]
        zoom: Option<bool>,
        /// Set, a drag or Shift with an arrow key selects a band of periods; `None` when
        /// Escape clears it. With `zoom`, Shift or Alt with a drag pans.
        #[props(default)]
        onrange: Option<EventHandler<Option<ChartRange>>>,
    }
}

/// Prices over periods as candles: a body from the open to the close on a wick from the low
/// to the high.
///
/// Never colour alone: a rising candle is hollow, a falling one filled, and a key under the
/// plot says so; the tooltip, the keys and the summary read every price. Web and Android;
/// enable the `charts` feature.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Candle, CandlestickChart};
/// # fn app() -> Element {
/// rsx! {
///     CandlestickChart {
///         title: "Share price",
///         x: vec!["Mon".to_string(), "Tue".to_string(), "Wed".to_string()],
///         candles: vec![
///             Candle::new(20.0, 38.0, 10.0, 34.0),
///             Candle::new(34.0, 40.0, 28.0, 30.0),
///             Candle::new(30.0, 44.0, 29.0, 42.0),
///         ],
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/chart>
#[component]
pub fn CandlestickChart(props: CandlestickChartProps) -> Element {
    let theme = use_theme();
    let labels = use_localization().chart;
    let formats = use_formats();
    let id = use_root_id(&props.attributes);
    use_name_warning(
        is_named(&props.title, &props.attributes),
        "CandlestickChart: no `title` - name the chart: it is how a screen reader finds it.",
    );
    let inverted: Vec<String> = (props.candles.iter().zip(&props.x))
        .filter(|(candle, _)| {
            candle.prices().is_some_and(|[open, high, low, close]| {
                high < open.max(close) || low > open.min(close)
            })
        })
        .map(|(_, x)| x.clone())
        .collect();
    use_hook(move || {
        if !inverted.is_empty() {
            warn(&format!(
                "CandlestickChart: a high under its body or a low over it at {} - drawn as given.",
                inverted.join(", ")
            ));
        }
    });

    let ratio = props
        .aspect_ratio
        .unwrap_or(theme.chart.aspect_ratio)
        .max(0.1);
    let size = use_plot_size();
    let (width, height) = size.get(ratio);
    let zoom = use_zoom(
        props.x.len(),
        props.zoom.unwrap_or(theme.chart.zoom),
        props.handle,
    );
    let window = zoom.map_or((0, props.x.len()), |zoom| zoom.range());
    let forced = use_accessibility().forced_colors();
    let name = props
        .name
        .clone()
        .unwrap_or_else(|| labels.price.to_string());
    let candles: Vec<Option<[f64; 4]>> = (0..props.x.len())
        .map(|i| props.candles.get(i).and_then(Candle::prices))
        .collect();
    let empty = candles.iter().all(Option::is_none);
    let key = (Frame::legend(&props.legend, theme, 2) != LegendPosition::None)
        .then(|| (labels.up_key.to_string(), labels.down_key.to_string()));
    let spec = CandleSpec {
        width,
        height,
        x: props.x.clone(),
        name: name.clone(),
        candles,
        up: resolve(theme, forced, props.up_color, Color::Success),
        down: resolve(theme, forced, props.down_color, Color::Error),
        ohlc: props.style.copied_or(CandleStyle::Candle) == CandleStyle::Ohlc,
        key,
        axes: Axes {
            min: props.y_min,
            max: props.y_max,
            ticks: props.y_ticks.unwrap_or(theme.chart.y_ticks),
            x_title: props.x_title.clone(),
            y_title: props.y_title.clone(),
            y2_title: None,
            compact: labels.compact,
        },
        grid: props.grid.unwrap_or(theme.chart.grid),
        ink: ChartInk::of(theme, forced),
        format: props.value_format,
        unit: props.unit.clone(),
        formats,
    };
    let view = CandleSpec {
        x: cut(&spec.x, window),
        candles: cut(&spec.candles, window),
        ..spec.clone()
    };
    // The file keeps the normal palette under forced colours.
    let export = {
        let spec = CandleSpec {
            up: resolve(theme, false, props.up_color, Color::Success),
            down: resolve(theme, false, props.down_color, Color::Error),
            ink: ChartInk::of(theme, false),
            ..view.clone()
        };
        ExportSource::of(theme, props.title.clone(), move || {
            engine::draw_candles(&spec)
        })
    };
    let drawn = use_memo(use_reactive!(|view, window, empty| {
        (!empty)
            .then(|| engine::draw_candles(&view).map(|drawn| drawn.with_offset(window.0)))
            .flatten()
    }));
    let hidden = use_hidden_series(&None, &[], None, &[]);

    let write = |value: f64| {
        props
            .value_format
            .format(value, props.unit.as_deref(), formats)
    };
    let summary = match (props.loading, empty) {
        (true, _) => vec![labels.loading.to_string()],
        (false, true) => Vec::new(),
        (false, false) => summary(&labels, &spec, write, formats),
    };
    let points = spec
        .candles
        .iter()
        .map(|candle| candle.map_or_else(String::new, |c| say(&labels, c, write)))
        .collect();
    let series = vec![OverlaySeries {
        name,
        color: "transparent".to_string(),
        dashed: false,
        symbol: None,
        line: false,
        unit: None,
        errors: Vec::new(),
        xs: Vec::new(),
        points,
    }];

    Frame {
        id,
        labels,
        title: props.title.clone(),
        description: props.description.clone(),
        loading: props.loading,
        empty,
        empty_slot: props.empty.clone(),
        drawn,
        center: None,
        legend: LegendPosition::None,
        legend_label: labels.legend,
        series,
        hidden,
        x: props.x.clone(),
        summary,
        plot: size,
        aspect_ratio: Some(ratio),
        interaction: Interaction {
            format: props.value_format,
            unit: props.unit.clone(),
            formats,
            tooltip: props.tooltip.unwrap_or(theme.chart.tooltip),
            tooltip_content: props.tooltip_content,
            onpointclick: props.onpointclick,
            onactivechange: props.onactivechange,
            swap_axes: false,
            onrange: props.onrange,
            layout: Layout::Columns,
        },
        export,
        handle: props.handle,
        zoom,
    }
    .render(
        &props.class,
        &props.sx,
        &props.parts,
        &props.states,
        props.attributes,
    )
}

fn resolve(
    theme: &Theme,
    forced: bool,
    color: Option<ChartColor>,
    fallback: Color,
) -> crate::theme::HexColor {
    match color.unwrap_or(ChartColor::from(fallback)) {
        ChartColor::Hex(hex) => hex,
        ChartColor::Palette(color, shade) => chart_color(theme, color, shade, forced),
    }
}

fn direction(labels: &ChartLabels, [open, _, _, close]: [f64; 4]) -> &str {
    match close.total_cmp(&open) {
        std::cmp::Ordering::Greater => labels.up,
        std::cmp::Ordering::Less => labels.down,
        std::cmp::Ordering::Equal => labels.unchanged,
    }
}

/// A candle's prices and its direction, as the tooltip and the keys say them.
fn say(labels: &ChartLabels, candle: [f64; 4], write: impl Fn(f64) -> String) -> String {
    let [open, high, low, close] = candle;
    fill(
        labels.candle,
        &[
            ("open", &write(open)),
            ("high", &write(high)),
            ("low", &write(low)),
            ("close", &write(close)),
            ("direction", &direction(labels, candle).to_string()),
        ],
    )
}

/// The periods, the first open and the last close with the change, the highest high and the
/// lowest low with their periods, and how many candles rose, fell and stayed.
fn summary(
    labels: &ChartLabels,
    spec: &CandleSpec,
    write: impl Fn(f64) -> String,
    formats: &'static Formats,
) -> Vec<String> {
    let drawn: Vec<(&String, [f64; 4])> = (spec.x.iter().zip(&spec.candles))
        .filter_map(|(x, candle)| Some((x, (*candle)?)))
        .collect();
    let (Some(first), Some(last)) = (drawn.first(), drawn.last()) else {
        return Vec::new();
    };
    let (open, close) = (first.1[0], last.1[3]);
    let signed = |text: String, value: f64| match value > 0.0 {
        true => format!("+{text}"),
        false => text,
    };
    let change = close - open;
    let change = match open != 0.0 {
        true => format!(
            "{} ({})",
            signed(write(change), change),
            signed(
                super::ValueFormat::Percent.format(change / open.abs(), None, formats),
                change
            )
        ),
        false => signed(write(change), change),
    };
    let highest = drawn.iter().max_by(|a, b| a.1[1].total_cmp(&b.1[1]));
    let lowest = drawn.iter().min_by(|a, b| a.1[2].total_cmp(&b.1[2]));
    let count = |word: &str| {
        let n = drawn.iter().filter(|c| direction(labels, c.1) == word);
        n.count().to_string()
    };
    let mut sentences = vec![
        fill(
            labels.candle_range,
            &[
                ("kind", &labels.candlestick.to_string()),
                ("count", &drawn.len().to_string()),
                ("first", first.0),
                ("last", last.0),
            ],
        ),
        fill(
            labels.candle_change,
            &[
                ("open", &write(open)),
                ("close", &write(close)),
                ("change", &change),
            ],
        ),
    ];
    if let (Some(highest), Some(lowest)) = (highest, lowest) {
        sentences.push(fill(
            labels.candle_extremes,
            &[
                ("high", &write(highest.1[1])),
                ("high_x", highest.0),
                ("low", &write(lowest.1[2])),
                ("low_x", lowest.0),
            ],
        ));
    }
    sentences.push(fill(
        labels.candle_count,
        &[
            ("up", &count(labels.up)),
            ("down", &count(labels.down)),
            ("flat", &count(labels.unchanged)),
        ],
    ));
    sentences
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LiberoProvider;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn days() -> Vec<String> {
        ["Mon", "Tue", "Wed", "Thu"].map(String::from).to_vec()
    }

    /// The server's HTML has the candles, the key and a summary of every price.
    #[test]
    fn the_first_render_is_a_named_figure_with_candles_and_a_summary() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    CandlestickChart {
                        title: "Share price",
                        x: days(),
                        candles: vec![
                            Candle::new(20.0, 38.0, 10.0, 34.0),
                            Candle::new(34.0, 40.0, 28.0, 30.0),
                            Candle::new(f64::NAN, 1.0, 1.0, 1.0),
                            Candle::new(30.0, 36.0, 28.0, 30.0),
                        ],
                    }
                }
            }
        });
        assert!(html.contains("role=\"figure\""), "{html}");
        assert!(html.matches("<rect").count() >= 3, "{html}");
        assert!(
            html.contains(">Up</text>") && html.contains(">Down</text>"),
            "{html}"
        );
        assert!(
            html.contains("Candlestick chart of 3 periods from Mon to Thu."),
            "{html}"
        );
        assert!(
            html.contains("Opened at 20, closed at 30: +10 (+50%)."),
            "{html}"
        );
        assert!(
            html.contains("Highest: 40 on Tue. Lowest: 10 on Mon."),
            "{html}"
        );
        assert!(html.contains("1 up, 1 down, 1 unchanged."), "{html}");
        assert!(!html.contains("NaN"), "{html}");
    }

    #[test]
    fn a_candle_reads_its_prices_and_direction() {
        let write = |value: f64| value.to_string();
        let said = say(&ChartLabels::ENGLISH, [20.0, 38.0, 10.0, 34.0], write);
        assert_eq!(said, "open 20, high 38, low 10, close 34, up");
        let said = say(&ChartLabels::ENGLISH, [34.0, 40.0, 28.0, 30.0], write);
        assert_eq!(said, "open 34, high 40, low 28, close 30, down");
    }

    #[test]
    fn a_high_under_the_body_warns() {
        let warnings = crate::utils::warnings_of(|| {
            rsx! {
                LiberoProvider {
                    CandlestickChart {
                        title: "Share price",
                        x: days(),
                        candles: vec![Candle::new(20.0, 18.0, 10.0, 34.0)],
                    }
                }
            }
        });
        assert_eq!(
            warnings,
            ["CandlestickChart: a high under its body or a low over it at Mon - drawn as given."]
        );
    }

    #[test]
    fn a_legend_of_none_leaves_the_key_out() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    CandlestickChart {
                        title: "Share price",
                        x: days(),
                        candles: vec![Candle::new(20.0, 38.0, 10.0, 34.0)],
                        legend: "none",
                    }
                }
            }
        });
        assert!(!html.contains(">Up</text>"), "{html}");
    }
}
