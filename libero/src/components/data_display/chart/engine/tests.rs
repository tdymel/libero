use super::*;
use crate::theme::{Color, ColorShade, Theme, chart_hex};

fn scatter(points: &[(f64, f64, Option<f64>)], symbol: PointSymbol) -> ScatterSpec {
    let series = |name: &str, symbol| ScatterSpecSeries {
        name: name.to_string(),
        points: points
            .iter()
            .enumerate()
            .map(|(i, &(x, y, size))| (i, x, y, size))
            .collect(),
        color: chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6),
        symbol,
    };
    ScatterSpec {
        width: 320.0,
        height: 180.0,
        series: vec![series("A", PointSymbol::Circle), series("B", symbol)],
        bubble: points.iter().any(|p| p.2.is_some()),
        trend: None,
        axes: axes(),
        x_min: None,
        x_max: None,
        grid: true,
        radius: 5.0,
        ink: ChartInk::of(&Theme::DEFAULT, false),
        format: ValueFormat::Auto,
        unit: None,
        formats: &Formats::GERMAN,
    }
}

/// Every point is a mark at its place, kept visible, its shape's centre; the axes fit the
/// data, not 0, and are written in the locale.
#[test]
fn scatter_points_are_marks_on_fitted_axes() {
    let points = [
        (150.0, 42.5, None),
        (160.0, 51.0, None),
        (175.5, 70.0, None),
    ];
    for symbol in [
        PointSymbol::Triangle,
        PointSymbol::Square,
        PointSymbol::Diamond,
    ] {
        let drawn = draw_scatter(&scatter(&points, symbol)).expect("drawn");
        let marks = &drawn.geometry.marks;
        assert_eq!(marks.len(), 6, "{symbol:?}: {}", drawn.svg);
        let at = |m: &Mark| match m.shape {
            MarkShape::Point { x, y } => (x, y),
            _ => panic!("{m:?}"),
        };
        // Both series lie on each other: B's centres are A's.
        for i in 0..3 {
            let (a, b) = (at(&marks[i]), at(&marks[i + 3]));
            assert!(
                (a.0 - b.0).abs() < 0.6 && (a.1 - b.1).abs() < 0.6,
                "{a:?} {b:?}"
            );
            assert_eq!((marks[i].series, marks[i].index), (0, i));
        }
        assert!(at(&marks[0]).0 < at(&marks[1]).0 && at(&marks[1]).1 < at(&marks[0]).1);
        assert_eq!(marks[2].value, 70.0);
        assert!(drawn.svg.contains("<circle"), "{}", drawn.svg);
        assert!(!drawn.svg.contains("ct-"), "{}", drawn.svg);
        // Not from 0: the x axis starts near the lowest x.
        assert!(!drawn.svg.contains(">\n0\n<"), "{}", drawn.svg);
        assert!(drawn.svg.contains("150"), "{}", drawn.svg);
    }
}

/// Bubbles are drawn largest first; the marks keep each point's place in its series.
#[test]
fn bubbles_keep_their_places() {
    let points = [
        (1.0, 1.0, Some(10.0)),
        (2.0, 2.0, Some(90.0)),
        (3.0, 3.0, Some(40.0)),
    ];
    let mut spec = scatter(&points, PointSymbol::Circle);
    spec.trend = Some(Trend::Linear);
    let drawn = draw_scatter(&spec).expect("drawn");
    let order: Vec<_> = drawn
        .geometry
        .marks
        .iter()
        .map(|m| (m.series, m.index))
        .collect();
    assert_eq!(order, [(0, 1), (0, 2), (0, 0), (1, 1), (1, 2), (1, 0)]);
    // The trend line is drawn, its formula not.
    assert!(!drawn.svg.contains("y ="), "{}", drawn.svg);
    // Both trend lines over every bubble, the bubbles translucent.
    let last_bubble = drawn.svg.rfind("<circle").expect("bubbles");
    assert_eq!(
        drawn.svg[last_bubble..].matches("<path").count(),
        2,
        "{}",
        drawn.svg
    );
    assert!(drawn.svg.contains("fill-opacity"), "{}", drawn.svg);
}

fn line(series: Vec<SeriesSpec>, x: &[&str]) -> LineSpec {
    LineSpec {
        width: 320.0,
        height: 180.0,
        x: x.iter().map(|x| x.to_string()).collect(),
        x_numbers: Vec::new(),
        series,
        errors: Vec::new(),
        curve: Curve::Linear,
        area: None,
        stacked: false,
        axes: axes(),
        grid: true,
        points: true,
        stroke_width: 2.0,
        ink: ChartInk::of(&Theme::DEFAULT, false),
        format: ValueFormat::Auto,
        unit: Some("kg".to_string()),
        y2_unit: None,
        formats: &Formats::GERMAN,
    }
}

fn axes() -> Axes {
    Axes {
        ticks: Theme::DEFAULT.chart.y_ticks,
        ..Axes::default()
    }
}

/// The renderer's own step count is the theme's, so an unset `y_ticks` changes nothing.
#[test]
fn the_theme_s_ticks_are_the_renderer_s() {
    let chart = charts_rs::LineChart::new(Vec::new(), Vec::new());
    assert_eq!(
        chart.y_axis_configs[0].split_number,
        Theme::DEFAULT.chart.y_ticks
    );
}

/// Fixed ends and a step count give exactly those labels; titles sit by their axes.
#[test]
fn a_fixed_axis_range_and_titles_are_drawn() {
    let mut spec = line(vec![spec("A", &[Some(12.0), Some(31.0)])], &["Q1", "Q2"]);
    spec.axes = Axes {
        min: Some(10.0),
        max: Some(50.0),
        ticks: 4,
        x_title: Some("Quarter".to_string()),
        y_title: Some("Weight".to_string()),
        ..Axes::default()
    };
    let svg = draw_line(&spec).expect("drawn").svg;
    for label in [
        "10 kg",
        "20 kg",
        "30 kg",
        "40 kg",
        "50 kg",
        "\nQuarter\n",
        "\nWeight\n",
    ] {
        assert!(svg.contains(label), "{label}: {svg}");
    }
    assert!(!svg.contains("60 kg") && !svg.contains("\n0 kg"), "{svg}");
    assert!(svg.contains("rotate(-90)"), "{svg}");
}

/// Unset ends land on a round step, as many as `ticks` asks or one more or fewer.
#[test]
fn an_unset_axis_end_lands_on_a_round_step() {
    let labels = |values: &[f64], axes: Axes| {
        let values: Vec<_> = values.iter().map(|v| Some(*v)).collect();
        let mut spec = line(vec![spec("A", &values)], &["Q1", "Q2"]);
        spec.axes = axes;
        let svg = draw_line(&spec).expect("drawn").svg;
        let ticks = texts(&svg).into_iter().map(|text| text.3);
        ticks
            .filter(|label| label.ends_with(" kg"))
            .collect::<Vec<_>>()
    };
    let kg = |values: &[i32]| values.iter().map(|v| format!("{v} kg")).collect::<Vec<_>>();
    assert_eq!(
        labels(&[12.0, 31.0], axes()),
        kg(&[35, 30, 25, 20, 15, 10, 5, 0])
    );
    let four = Axes { ticks: 4, ..axes() };
    assert_eq!(
        labels(&[3.0, 48.0], four.clone()),
        kg(&[50, 40, 30, 20, 10, 0])
    );
    assert_eq!(labels(&[-7.0, 18.0], four), kg(&[20, 10, 0, -10]));
    assert_eq!(labels(&[0.0, 0.0], axes())[0], "1,0 kg");
    let from = Axes {
        min: Some(10.0),
        ..axes()
    };
    assert_eq!(labels(&[12.0, 31.0], from), kg(&[35, 30, 25, 20, 15, 10]));
}

/// Ticks past seven digits are written in the locale's millions; the rest keep theirs.
#[test]
fn a_long_axis_writes_its_ticks_in_millions() {
    let compact = crate::localization::ChartLabels::GERMAN.compact;
    let mut short = line(vec![spec("A", &[Some(1.0), Some(4.0)])], &["Q1", "Q2"]);
    short.axes.compact = compact;
    let mut long = line(
        vec![spec("A", &[Some(12_345_678.0), Some(48_000_000.0)])],
        &["Q1", "Q2"],
    );
    long.axes.compact = compact;
    let svg = draw_line(&long).expect("drawn").svg;
    let ticks: Vec<String> = texts(&svg)
        .into_iter()
        .map(|text| text.3)
        .filter(|label| label.contains("Mio."))
        .collect();
    assert!(ticks.len() >= 3, "{svg}");
    assert!(ticks.contains(&"40\u{a0}Mio. kg".to_string()), "{ticks:?}");
    assert!(texts(&svg).iter().any(|text| text.3 == "0 kg"), "{svg}");
    let digits = |label: &str| label.chars().filter(char::is_ascii_digit).count();
    assert!(texts(&svg).iter().all(|text| digits(&text.3) <= 7), "{svg}");

    let svg = draw_line(&short).expect("drawn").svg;
    assert!(!svg.contains("Mio."), "{svg}");
}

/// Every tick of a tiny axis has the decimals its step needs, so none reads 0 but zero.
#[test]
fn a_tiny_axis_writes_every_tick_at_one_precision() {
    let mut spec = line(vec![spec("A", &[Some(0.001), Some(0.004)])], &["Q1", "Q2"]);
    spec.unit = None;
    let svg = draw_line(&spec).expect("drawn").svg;
    let ticks: Vec<String> = texts(&svg)
        .into_iter()
        .map(|text| text.3)
        .filter(|label| label.starts_with("0,"))
        .collect();
    assert!(ticks.len() >= 3, "{svg}");
    let decimals = |label: &String| label.len() - 2;
    assert!(
        ticks.iter().all(|t| decimals(t) == decimals(&ticks[0])),
        "{ticks:?}"
    );
    let mut distinct = ticks.clone();
    distinct.dedup();
    assert_eq!(distinct, ticks);
}

/// A series on the right axis is drawn by its own scale, labelled in its unit there.
#[test]
fn a_secondary_series_is_measured_on_the_right() {
    let mut rain = spec("Rain", &[Some(2.0), Some(4.0)]);
    rain.extras.secondary = true;
    let mut spec = line(
        vec![spec("A", &[Some(120.0), Some(310.0)]), rain],
        &["Q1", "Q2"],
    );
    spec.y2_unit = Some("mm".to_string());
    spec.axes.y2_title = Some("Rainfall".to_string());
    let drawn = draw_line(&spec).expect("drawn");
    let plot = &drawn.geometry.plot;
    assert!(drawn.svg.contains("\nRainfall\n"), "{}", drawn.svg);
    assert!(!drawn.svg.contains(AXIS2_MARK), "{}", drawn.svg);
    let mut rest = drawn.svg.as_str();
    let mut right = 0;
    while let Some(at) = rest.find(" mm\n") {
        let tag = &rest[rest[..at].rfind("<text").expect("a label")..at];
        let x = attribute(&attributes(tag), "x").and_then(|x| x.parse::<f32>().ok());
        assert!(x.is_some_and(|x| x >= plot.right), "{tag}");
        right += 1;
        rest = &rest[at + 1..];
    }
    assert!(right >= 2, "{}", drawn.svg);
    let high = drawn
        .geometry
        .marks
        .iter()
        .find(|m| m.series == 1 && m.index == 1);
    let Some(MarkShape::Point { y, .. }) = high.map(|mark| &mark.shape) else {
        panic!("no point");
    };
    assert!(*y < (plot.top + plot.bottom) / 2.0, "{y} in {plot:?}");
}

/// Error ranges fill a band around a line, only where it has points, on an axis that
/// spans them; the marks stay its points. A stack draws none.
#[test]
fn error_ranges_draw_a_band_around_the_line() {
    let series = spec("A", &[Some(2.0), None, Some(4.0), Some(3.0)]);
    let mut spec = line(vec![series], &["a", "b", "c", "d"]);
    spec.errors = vec![vec![
        Some((1.0, 3.0)),
        Some((1.0, 2.0)),
        Some((3.0, 9.0)),
        Some((2.0, 4.0)),
    ]];
    let drawn = draw_line(&spec).expect("drawn");
    assert_eq!(
        drawn.svg.matches("class=\"ct-band\"").count(),
        1,
        "{}",
        drawn.svg
    );
    assert!(
        drawn.svg.contains("9 kg") || drawn.svg.contains("10 kg"),
        "{}",
        drawn.svg
    );
    assert_eq!(drawn.geometry.marks.len(), 3);
    spec.stacked = true;
    let stacked = draw_line(&spec).expect("drawn");
    assert!(!stacked.svg.contains("ct-band"), "{}", stacked.svg);
}

/// Each shape draws its own marker at every point, in the series' colour; the points are
/// still harvested.
#[test]
fn a_series_draws_its_point_symbol() {
    let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
    for (symbol, tag) in [
        (PointSymbol::Square, "<rect"),
        (PointSymbol::Triangle, "<polygon"),
        (PointSymbol::Diamond, "<polygon"),
    ] {
        series.extras.symbol = symbol;
        let drawn = draw_line(&line(vec![series.clone()], &["a", "b", "c"])).expect("drawn");
        assert_eq!(
            drawn.svg.matches(tag).count(),
            3,
            "{symbol:?}: {}",
            drawn.svg
        );
        assert!(!drawn.svg.contains("<circle"), "{symbol:?}: {}", drawn.svg);
        assert_eq!(drawn.geometry.marks.len(), 3, "{symbol:?}");
    }
}

/// An area is filled see-through at the theme's opacity; stacked, each line sits on the
/// one before and its point keeps its own value.
#[test]
fn stacked_areas_fill_between_the_lines() {
    let series = vec![
        spec("A", &[Some(1.0), Some(2.0)]),
        spec("B", &[Some(3.0), Some(1.0)]),
    ];
    let mut spec = line(series, &["a", "b"]);
    spec.area = Some(Theme::DEFAULT.chart.area_opacity);
    let alone = draw_line(&spec).expect("drawn");
    spec.stacked = true;
    let stacked = draw_line(&spec).expect("drawn");
    assert!(
        stacked.svg.contains("fill-opacity=\"0.2\""),
        "{}",
        stacked.svg
    );
    let values: Vec<f64> = stacked.geometry.marks.iter().map(|m| m.value).collect();
    assert_eq!(values, [1.0, 2.0, 3.0, 1.0]);
    let y = |drawn: &Drawn, i: usize| match drawn.geometry.marks[i].shape {
        MarkShape::Point { y, .. } => y,
        _ => panic!(),
    };
    // At b, B's 1 is under A's 2 alone, stacked over it at 2 + 1.
    assert!(y(&alone, 3) > y(&alone, 1), "{}", alone.svg);
    assert!(y(&stacked, 3) < y(&stacked, 1), "{}", stacked.svg);
}

/// A mark line is drawn at its value in the series' colour and labelled in the label ink;
/// one past the axis' ends or in a stack is not drawn.
#[test]
fn mark_lines_are_drawn_and_labelled() {
    let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
    series.extras.mark_lines = vec![(2.0, "Average: 2".into()), (9.0, "Out".into())];
    let mut spec = line(vec![series.clone()], &["a", "b", "c"]);
    let drawn = draw_line(&spec).expect("drawn");
    assert_eq!(
        drawn.svg.matches("stroke-dasharray=\"4,2\"").count(),
        1,
        "{}",
        drawn.svg
    );
    let label = format!(
        "fill=\"{}\" stroke=\"{}\"",
        spec.ink.label, spec.ink.surface
    );
    assert!(drawn.svg.contains(&label), "{}", drawn.svg);
    assert!(drawn.svg.contains("\nAverage: 2\n"), "{}", drawn.svg);
    assert!(!drawn.svg.contains("Out"), "{}", drawn.svg);
    assert_eq!(drawn.geometry.marks.len(), 3);
    spec.stacked = true;
    assert!(!draw_line(&spec).expect("drawn").svg.contains("Average"));

    let mut bars = bars(BarLayout::Grouped, false);
    bars.series[0].extras.mark_lines = vec![(1.0, "Target: 1".into())];
    assert!(
        draw_bar(&bars)
            .expect("drawn")
            .svg
            .contains("\nTarget: 1\n")
    );
    bars.horizontal = true;
    assert!(!draw_bar(&bars).expect("drawn").svg.contains("Target"));
}

/// A marked point's value sits in a pill of its colour above it, or below it at the top,
/// in the ink that reads on that colour.
#[test]
fn a_marked_point_is_pinned_by_its_value() {
    let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
    series.extras.mark_points = vec![(0, "1 kg".into()), (1, "3 kg".into())];
    let spec = line(vec![series.clone()], &["a", "b", "c"]);
    let drawn = draw_line(&spec).expect("drawn");
    assert_eq!(
        drawn.svg.matches("class=\"ct-pin\"").count(),
        2,
        "{}",
        drawn.svg
    );
    let ink = format!("fill=\"{}\">3 kg</text>", spec.ink.on(series.color));
    assert!(drawn.svg.contains(&ink), "{}", drawn.svg);
    let tops: Vec<f32> = drawn
        .svg
        .split("class=\"ct-pin\"><rect")
        .skip(1)
        .filter_map(|pin| pin.split("y=\"").nth(1)?.split('"').next()?.parse().ok())
        .collect();
    let ys: Vec<f32> = drawn.geometry.marks[..2]
        .iter()
        .map(|m| match m.shape {
            MarkShape::Point { y, .. } => y,
            _ => panic!(),
        })
        .collect();
    assert!(
        tops[0] < ys[0],
        "the low point's pin is above it: {tops:?} {ys:?}"
    );
    assert_eq!(drawn.geometry.marks.len(), 3);
}

/// A lying bar chart's value axis is at the bottom: the y title goes there.
#[test]
fn a_horizontal_bar_chart_swaps_its_titles() {
    let mut spec = bars(BarLayout::Grouped, true);
    spec.axes.x_title = Some("Team".to_string());
    spec.axes.y_title = Some("Score".to_string());
    let svg = draw_bar(&spec).expect("drawn").svg;
    let rotated = svg
        .split("<text")
        .find(|tag| tag.contains("rotate(-90)"))
        .unwrap_or_else(|| panic!("{svg}"));
    assert!(rotated.contains("\nTeam\n"), "{rotated}");
    assert!(svg.contains("\nScore\n"), "{svg}");
}

fn spec(name: &str, values: &[Option<f64>]) -> SeriesSpec {
    SeriesSpec {
        name: name.to_string(),
        values: values.to_vec(),
        color: chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6),
        dashed: false,
        extras: SeriesExtras::default(),
    }
}

/// Contract with the pinned `charts-rs`: under forced colours the ink it draws is ours to
/// trade, so no stand-in hex is left and the series keep theirs.
#[test]
fn forced_ink_leaves_no_stand_in_in_the_svg() {
    let mut spec = line(vec![spec("A", &[Some(1.0), Some(3.0)])], &["Mon", "Tue"]);
    spec.ink = ChartInk::of(&Theme::DEFAULT, true);
    let svg = draw_line(&spec).expect("drawn").svg;
    assert!(!svg.contains("#020"), "{svg}");
    assert!(svg.contains("CanvasText"));
    assert!(svg.contains(&spec.series[0].color.to_string()));
}

/// The y labels, written again grouped and with their unit, end on one line left of
/// the plot: the axis grows to hold them.
#[test]
fn grouped_y_labels_end_left_of_the_plot() {
    let drawn = draw_line(&line(
        vec![spec("A", &[Some(12_400.0), Some(21_300.0)])],
        &["Q1", "Q2"],
    ))
    .expect("drawn");
    let ends: Vec<f32> = drawn
        .svg
        .split("<text x=\"")
        .skip(1)
        .filter(|tag| tag.contains("text-anchor=\"end\""))
        .filter_map(|tag| tag.split('"').next()?.parse().ok())
        .collect();
    assert!(ends.len() > 2, "{}", drawn.svg);
    assert!(ends.iter().all(|end| *end == ends[0]), "{ends:?}");
    assert!(drawn.svg.contains("25.000 kg"), "{}", drawn.svg);
    let first = xs(&drawn)[0];
    assert!(
        ends[0] < first,
        "labels end at {} past the plot at {first}",
        ends[0]
    );
}

/// Contract with the pinned `charts-rs`: a line's points come back in data order,
/// gaps skipped, and the SVG keeps none of its own tooltips.
#[test]
fn a_line_s_points_are_harvested_and_its_tooltips_dropped() {
    let drawn = draw_line(&line(
        vec![
            spec("A & B", &[Some(1.0), None, Some(3.0)]),
            spec("C", &[Some(2.0), Some(2.5), Some(1.0)]),
        ],
        &["Mon", "Tue", "Wed"],
    ))
    .expect("drawn");
    let marks = &drawn.geometry.marks;

    let places: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
    assert_eq!(places, [(0, 0), (0, 2), (1, 0), (1, 1), (1, 2)]);
    let ys: Vec<_> = marks
        .iter()
        .map(|m| match m.shape {
            MarkShape::Point { y, .. } => y,
            _ => panic!("{m:?}"),
        })
        .collect();
    // Higher values sit higher up.
    assert!(ys[1] < ys[0] && ys[3] < ys[2], "{ys:?}");
    for gone in [
        "ct-trigger",
        "ct-tip",
        "<title",
        "<style",
        "Roboto",
        "\u{2060}",
        "\u{2061}",
    ] {
        assert!(!drawn.svg.contains(gone), "{gone}: {}", drawn.svg);
    }
    assert!(
        drawn
            .svg
            .starts_with("<svg width=\"100%\" height=\"100%\" aria-hidden=\"true\"")
    );
    assert!(drawn.svg.contains("2,5 kg"), "{}", drawn.svg);
}

fn xs(drawn: &Drawn) -> Vec<f32> {
    drawn
        .geometry
        .marks
        .iter()
        .map(|m| match m.shape {
            MarkShape::Point { x, .. } => x,
            _ => panic!("{m:?}"),
        })
        .collect()
}

/// Numbers are placed by value, categories evenly, and both are harvested in data order.
#[test]
fn numeric_x_values_are_spaced_by_value() {
    let values = [Some(1.0), Some(2.0), Some(3.0)];
    let mut numeric = line(vec![spec("A", &values)], &["1", "2", "10"]);
    let even = xs(&draw_line(&numeric).expect("drawn"));
    assert!(
        ((even[1] - even[0]) - (even[2] - even[1])).abs() < 0.1,
        "{even:?}"
    );

    numeric.x_numbers = vec![1.0, 2.0, 10.0];
    let drawn = draw_line(&numeric).expect("drawn");
    let at = xs(&drawn);
    assert_eq!(at.len(), 3, "{drawn:?}");
    let ratio = (at[2] - at[1]) / (at[1] - at[0]);
    assert!((ratio - 8.0).abs() < 0.05, "{at:?}");
}

/// The renderer's `2.02k` on the axis reads as the number in the chart's format.
#[test]
fn numeric_x_labels_are_written_in_the_locale() {
    let mut numeric = line(vec![spec("A", &[Some(1.0), Some(2.0)])], &["a", "b"]);
    numeric.x_numbers = vec![2020.0, 2030.0];
    let years = draw_line(&numeric).expect("drawn").svg;
    assert!(years.contains("\n2025\n"), "{years}");
    numeric.x_numbers = vec![0.0, 1.0];
    let svg = draw_line(&numeric).expect("drawn").svg;
    assert!(svg.contains("\n0,5\n") || svg.contains("\n0,2\n"), "{svg}");
}

#[test]
fn a_non_finite_x_number_still_draws() {
    let mut numeric = line(
        vec![spec("A", &[Some(1.0), Some(2.0), Some(3.0)])],
        &["a", "b", "c"],
    );
    numeric.x_numbers = vec![1.0, f64::NAN, 3.0];
    let drawn = draw_line(&numeric).expect("drawn");
    let places: Vec<_> = drawn.geometry.marks.iter().map(|m| m.index).collect();
    assert!(places.is_empty() || places == [0, 2], "{places:?}");
    numeric.x_numbers = vec![f64::NAN; 3];
    assert!(draw_line(&numeric).is_some());
}

#[test]
fn a_suffixed_number_is_read_back() {
    assert_eq!(scaled("1.5k"), Some(1500.0));
    assert_eq!(scaled("-2M"), Some(-2e6));
    assert_eq!(scaled("42"), Some(42.0));
    assert_eq!(scaled("Jan"), None);
    assert_eq!(scaled(""), None);
}

/// The series' hex is the one drawn: no `var()` reaches the renderer.
#[test]
fn a_line_is_drawn_in_its_hex() {
    let drawn =
        draw_line(&line(vec![spec("A", &[Some(1.0), Some(2.0)])], &["a", "b"])).expect("drawn");
    let hex = chart_hex(&Theme::DEFAULT, Color::Primary, ColorShade::S6).to_string();
    assert!(
        drawn.svg.to_uppercase().contains(&hex.to_uppercase()),
        "{hex}: {}",
        drawn.svg
    );
}

#[test]
fn a_mismatch_harvests_no_marks() {
    let triggers = [Trigger {
        series: "B".to_string(),
        shape: MarkShape::Point { x: 0.0, y: 0.0 },
    }];
    assert!(match_marks(&[(0, 0, 1.0, "A")], &triggers).is_empty());
}

/// Contract: a bar is a trigger `rect`, kept as the mark, its hooks stripped.
#[test]
fn a_bar_is_harvested_as_its_rect() {
    let mut chart = charts_rs::BarChart::new(
        vec![charts_rs::Series::new("A".to_string(), vec![1.0, 2.0])],
        vec!["x".to_string(), "y".to_string()],
    );
    chart.tooltip.show = true;
    chart.background_color = InkColor::transparent();
    let (svg, triggers) = sanitize(&chart.svg().expect("svg"), |v| v.to_string());

    assert_eq!(triggers.len(), 2);
    assert!(
        triggers
            .iter()
            .all(|t| matches!(t.shape, MarkShape::Rect { height, .. } if height > 0.0))
    );
    assert_eq!(svg.matches("<rect").count(), 2, "{svg}");
    assert!(
        !svg.contains("ct-trigger") && !svg.contains("<title"),
        "{svg}"
    );
}

fn bars(layout: BarLayout, horizontal: bool) -> BarSpec {
    BarSpec {
        width: 320.0,
        height: 180.0,
        x: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        series: vec![
            spec("A", &[Some(1.0), Some(3.0), None]),
            spec("B", &[Some(3.0), Some(1.0), Some(2.0)]),
        ],
        layout,
        horizontal,
        radius: 4.0,
        value_labels: true,
        axes: axes(),
        labels_inside: layout != BarLayout::Grouped || horizontal,
        bar_colors: Vec::new(),
        errors: Vec::new(),
        bar_width: None,
        bar_gap: RENDERER_GAP,
        grid: true,
        ink: ChartInk::of(&Theme::DEFAULT, false),
        format: ValueFormat::Auto,
        unit: None,
        y2_unit: None,
        curve: Curve::Linear,
        points: true,
        stroke_width: 2.0,
        formats: &Formats::GERMAN,
    }
}

#[test]
fn a_lying_bar_chart_keeps_its_value_labels_apart() {
    let mut lying = bars(BarLayout::Grouped, true);
    lying.width = 320.0;
    lying.value_labels = false;
    lying.axes.min = Some(-50.0);
    lying.axes.max = Some(150.0);
    lying.axes.ticks = 6;
    let svg = draw_bar(&lying).expect("drawn").svg;
    let ticks: Vec<(f32, f32)> = svg
        .split("<text")
        .filter(|text| text.contains("text-anchor=\"middle\""))
        .filter_map(|text| {
            let open = format!("<text{}", text.split('>').next()?);
            let tag = attributes(&open);
            let x = attribute(&tag, "x")?.parse::<f32>().ok()?;
            let label = text.split('>').nth(1)?.split('<').next()?.trim();
            label
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-'))
                .then(|| (x, text_width(&tag, label)))
        })
        .collect();

    assert!(ticks.len() >= 2, "{svg}");
    for pair in ticks.windows(2) {
        assert!(
            pair[0].0 + pair[0].1 / 2.0 < pair[1].0 - pair[1].1 / 2.0,
            "{svg}"
        );
    }
}

/// A line among bars: its points are marks after the bars', centred in their bands,
/// and it takes no slot of the group.
#[test]
fn a_line_over_bars_is_harvested_after_them() {
    let mut spec = bars(BarLayout::Grouped, false);
    spec.series[1].extras.line = true;
    spec.bar_width = Some(0.6);
    let drawn = draw_bar(&spec).expect("drawn");
    let marks = &drawn.geometry.marks;

    assert_eq!(marks.len(), 5, "{marks:?}");
    assert!(
        marks[..2]
            .iter()
            .all(|m| m.series == 0 && matches!(m.shape, MarkShape::Rect { .. }))
    );
    assert!(
        marks[2..]
            .iter()
            .all(|m| m.series == 1 && matches!(m.shape, MarkShape::Point { .. }))
    );
    let plot = drawn.geometry.plot;
    let band = (plot.right - plot.left) / 3.0;
    for mark in &marks[2..] {
        let MarkShape::Point { x, .. } = mark.shape else {
            unreachable!()
        };
        let middle = plot.left + band * (mark.index as f32 + 0.5);
        assert!((x - middle).abs() < 1.0, "{x} vs {middle}");
    }
    // The one bar series fills the whole 60% of its band.
    let (_, _, width, _) = rect(&marks[0]);
    assert!(
        (width - band * 0.6).abs() < 1.0,
        "{width} vs {}",
        band * 0.6
    );
}

/// A line on the right axis is measured by its own scale and unit.
#[test]
fn a_line_over_bars_can_take_the_right_axis() {
    let mut chart = bars(BarLayout::Stacked, false);
    chart.series.push(SeriesSpec {
        extras: SeriesExtras {
            line: true,
            secondary: true,
            ..Default::default()
        },
        ..spec("Share", &[Some(0.2), Some(0.5), Some(0.4)])
    });
    chart.y2_unit = Some("%".to_string());
    chart.axes.y2_title = Some("Share".to_string());
    let drawn = draw_bar(&chart).expect("drawn");

    assert_eq!(drawn.geometry.marks.len(), 8);
    // The right axis writes its unit, the left one none; its title stands beside it.
    assert_eq!(drawn.svg.matches("%\n</text>").count(), 5, "{}", drawn.svg);
    assert!(
        drawn.svg.contains("rotate(90)\"") && drawn.svg.contains("Share"),
        "{}",
        drawn.svg
    );
}

/// A stacked bar on the right axis is a stack of its own beside the left one, here too
/// when the bars are set again at their own width.
#[test]
fn a_bar_on_the_right_axis_stacks_apart() {
    for bar_width in [None, Some(0.8)] {
        let mut chart = bars(BarLayout::Stacked, false);
        chart.series[1].extras.secondary = true;
        chart.bar_width = bar_width;
        let drawn = draw_bar(&chart).expect("drawn");
        let marks = &drawn.geometry.marks;
        let at = |series: usize| {
            let mark = marks
                .iter()
                .find(|m| m.series == series && m.index == 0)
                .expect("a mark");
            rect(mark)
        };
        let (left, right) = (at(0), at(1));
        assert!(left.0 + left.2 <= right.0 + 0.5, "{left:?} {right:?}");
    }
}

/// A line among bars pins its marked point by its value, as a line chart does.
#[test]
fn a_line_over_bars_pins_its_marked_point() {
    let mut chart = bars(BarLayout::Grouped, false);
    chart.series.push(SeriesSpec {
        extras: SeriesExtras {
            line: true,
            mark_points: vec![(1, "Peak 7".to_string())],
            ..Default::default()
        },
        ..spec("Target", &[Some(4.0), Some(7.0), Some(5.0)])
    });
    let drawn = draw_bar(&chart).expect("drawn");
    assert_eq!(
        drawn.svg.matches("class=\"ct-pin\"").count(),
        1,
        "{}",
        drawn.svg
    );
    assert!(drawn.svg.contains("Peak 7"), "{}", drawn.svg);
}

fn rect(mark: &Mark) -> (f32, f32, f32, f32) {
    match mark.shape {
        MarkShape::Rect {
            x,
            y,
            width,
            height,
        } => (x, y, width, height),
        _ => panic!("{mark:?}"),
    }
}

/// Which corners of each bar path are rounded, as `[top left, top right, bottom right,
/// bottom left]`, in drawing order.
fn corners(svg: &str) -> Vec<[bool; 4]> {
    svg.split("<path d=\"")
        .skip(1)
        .map(|path| {
            let d = path.split('"').next().unwrap_or_default();
            let arcs: Vec<bool> = d
                .split('A')
                .skip(1)
                .map(|arc| arc.split(',').next() != Some("0"))
                .collect();
            // The path walks top right, bottom right, bottom left, top left.
            [arcs[3], arcs[0], arcs[1], arcs[2]]
        })
        .collect()
}

const TOP: [bool; 4] = [true, true, false, false];
const BOTTOM: [bool; 4] = [false, false, true, true];
const RIGHT: [bool; 4] = [false, true, true, false];
const LEFT: [bool; 4] = [true, false, false, true];
const SQUARE: [bool; 4] = [false; 4];

/// Contract: grouped bars come back series by series, rounded, a gap without a bar.
#[test]
fn grouped_bars_are_harvested_in_data_order() {
    let drawn = draw_bar(&bars(BarLayout::Grouped, false)).expect("drawn");
    let marks = &drawn.geometry.marks;

    let places: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
    assert_eq!(places, [(0, 0), (0, 1), (1, 0), (1, 1), (1, 2)]);
    // A's 3 is taller than its 1, and A's bar sits left of B's in the same group.
    assert!(rect(&marks[1]).3 > rect(&marks[0]).3);
    assert!(rect(&marks[0]).0 < rect(&marks[2]).0);
    assert_eq!(corners(&drawn.svg), [TOP; 5], "{}", drawn.svg);
    assert!(!drawn.svg.contains("data-series"), "{}", drawn.svg);
}

/// Horizontal, the first category is the top row and the marks keep data indices.
#[test]
fn horizontal_bars_read_top_down() {
    let drawn = draw_bar(&bars(BarLayout::Grouped, true)).expect("drawn");
    let marks = &drawn.geometry.marks;

    let a = marks
        .iter()
        .find(|m| (m.series, m.index) == (1, 0))
        .expect("B at a");
    let c = marks
        .iter()
        .find(|m| (m.series, m.index) == (1, 2))
        .expect("B at c");
    assert!(rect(a).1 < rect(c).1, "{marks:?}");
    // Values grow to the right.
    assert!(rect(a).2 > rect(c).2, "{marks:?}");
    assert_eq!(a.value, 3.0);
}

/// A percent stack fills the plot's height, its labels are shares in the locale, and
/// its marks keep the values.
#[test]
fn percent_bars_fill_the_stack_and_keep_their_values() {
    let drawn = draw_bar(&bars(BarLayout::Percent, false)).expect("drawn");
    let marks = &drawn.geometry.marks;

    let (a, b) = (rect(&marks[0]), rect(&marks[2]));
    assert!((a.3 * 3.0 - b.3).abs() < 1.0, "{a:?} {b:?}");
    // Stacked: same x, B on top of A.
    assert_eq!(a.0, b.0);
    assert!((b.1 + b.3 - a.1).abs() < 1.0, "{a:?} {b:?}");
    assert_eq!(marks[0].value, 1.0);
    assert!(
        drawn.svg.contains("25%") && drawn.svg.contains("100%"),
        "{}",
        drawn.svg
    );
    assert!(!drawn.svg.contains('\u{2061}'), "{}", drawn.svg);
    // Labels on the bars take the ink that reads on them.
    assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
    let on = ChartInk::of(&Theme::DEFAULT, false).on(spec("A", &[]).color);
    assert!(
        drawn.svg.contains(&format!("fill=\"{on}\"")),
        "{on}: {}",
        drawn.svg
    );
}

/// Shares of the absolute sum, so a stack with losses runs from its deepest loss to 100%,
/// never past it (3470).
#[test]
fn a_percent_axis_with_losses_stays_within_100() {
    let mut spec = bars(BarLayout::Percent, false);
    spec.x.push("d".to_string());
    spec.height = 480.0;
    spec.series = vec![
        self::spec("A", &[Some(40.0), Some(55.0), Some(-25.0), Some(62.0)]),
        self::spec("B", &[Some(30.0), Some(-15.0), Some(20.0), Some(48.0)]),
    ];
    spec.axes.ticks = 8;
    let svg = draw_bar(&spec).expect("drawn").svg;
    assert!(svg.contains("100%") && svg.contains("60%"), "{svg}");
    assert!(!svg.contains("120%"), "{svg}");
    let axes = percent_axes(&spec.axes, &spec.series);
    assert_eq!(
        (axes.min, axes.max, axes.ticks),
        (Some(-60.0), Some(100.0), 8)
    );
}

/// The deepest loss's axis is cut into at most `y_ticks` round steps (3515).
#[test]
fn a_percent_axis_with_losses_honours_y_ticks() {
    let mut spec = bars(BarLayout::Percent, false);
    spec.series = vec![
        self::spec("A", &[Some(40.0), Some(-25.0), Some(10.0)]),
        self::spec("B", &[Some(30.0), Some(20.0), Some(5.0)]),
    ];
    for ticks in [2, 4, 5, 10] {
        spec.axes.ticks = ticks;
        let axes = percent_axes(&spec.axes, &spec.series);
        assert_eq!(axes.max, Some(100.0));
        assert!(axes.ticks <= ticks, "{ticks}: {axes:?}");
        assert!(axes.ticks * 2 >= ticks, "{ticks}: {axes:?}");
    }
}

/// A line on the right axis of a percent chart is cut into as many round steps as the
/// left axis (3526).
#[test]
fn a_percent_chart_rounds_its_right_axis() {
    let mut chart = bars(BarLayout::Percent, false);
    chart.axes.ticks = 5;
    chart.series.push(SeriesSpec {
        extras: SeriesExtras {
            line: true,
            secondary: true,
            ..Default::default()
        },
        ..spec("Margin", &[Some(12.3), Some(25.1), Some(37.8)])
    });
    let svg = draw_bar(&chart).expect("drawn").svg;
    for label in ["\n0\n", "\n10\n", "\n20\n", "\n30\n", "\n40\n", "\n50\n"] {
        assert!(svg.contains(label), "{label}: {svg}");
    }
}

/// A lying chart cuts a long category at a third of its width (3525).
#[test]
fn a_lying_chart_cuts_a_long_category() {
    let mut chart = bars(BarLayout::Grouped, true);
    chart.x[0] = "A category name that goes on and on and on".to_string();
    let drawn = draw_bar(&chart).expect("drawn");
    assert!(drawn.svg.contains("…"), "{}", drawn.svg);
    assert!(!drawn.svg.contains("goes on and on"), "{}", drawn.svg);
    assert!(
        drawn.geometry.plot.left < chart.width / 3.0 + 10.0,
        "{:?}",
        drawn.geometry.plot
    );
}

/// Of two value labels that overlap the smaller goes; one clear of both stays (3524).
#[test]
fn crowded_value_labels_keep_the_larger() {
    let label = |x: f32, value: &str| {
        format!("<text x=\"{x}\" y=\"20\" font-size=\"14\">\n{LABEL_MARK}{value}\n</text>")
    };
    let svg = format!(
        "<svg>{}{}{}</svg>",
        label(10.0, "5"),
        label(12.0, "9"),
        label(200.0, "1")
    );
    let kept = drop_crowded_labels(&svg, |v| v.to_string());
    assert!(
        kept.contains('9') && kept.contains('1') && !kept.contains('5'),
        "{kept}"
    );
}

/// Many series in a group leave fewer labels, none overlapping, in the real drawing (3524).
#[test]
fn a_crowded_group_drops_labels() {
    let mut chart = bars(BarLayout::Grouped, false);
    chart.series = (0..10)
        .map(|n| {
            spec(
                "S",
                &[Some(1000.0 + n as f64), Some(2000.0 + n as f64), None],
            )
        })
        .collect();
    chart.labels_inside = false;
    let texts = |chart: &BarSpec| draw_bar(chart).expect("drawn").svg.matches("<text").count();
    let with = texts(&chart);
    chart.value_labels = false;
    let shown = with - texts(&chart);
    assert!(0 < shown && shown < 20, "{shown}");
}

/// The `[x1, y1, x2, y2]` of every error-bar line the engine drew itself.
fn error_lines(svg: &str) -> Vec<[f32; 4]> {
    svg.split("<line stroke-width=\"1.5\"")
        .skip(1)
        .map(|tail| {
            let tag = format!("<line{}", &tail[..tail.find("/>").unwrap_or(tail.len())]);
            let attributes = attributes(&tag);
            ["x1", "y1", "x2", "y2"].map(|name| {
                attribute(&attributes, name)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(f32::NAN)
            })
        })
        .collect()
}

/// A range on a stack or a lying bar is read from the end of its own segment (3480).
#[test]
fn a_range_on_a_stacked_or_lying_bar_starts_at_its_segment() {
    for horizontal in [false, true] {
        let mut spec = bars(BarLayout::Stacked, horizontal);
        spec.series[1].values = vec![Some(3.0), Some(1.0), Some(2.0)];
        spec.errors = vec![vec![], vec![Some((2.0, 5.0)), None, None]];
        let drawn = draw_bar(&spec).expect("drawn");
        let b = drawn
            .geometry
            .marks
            .iter()
            .find(|m| m.series == 1 && m.index == 0)
            .expect("B at a");
        let MarkShape::Rect {
            x,
            y,
            width,
            height,
        } = &b.shape
        else {
            panic!("a bar")
        };
        let (x, y, width, height) = (*x, *y, *width, *height);
        // The stem runs from the value 2 to the value 5, over a bar of 3.
        let stem = error_lines(&drawn.svg)
            .into_iter()
            .find(|l| match horizontal {
                true => l[1] == l[3],
                false => l[0] == l[2],
            })
            .expect("a stem");
        let (end, k) = match horizontal {
            true => (x + width, width / 3.0),
            false => (y, height / 3.0),
        };
        let [from, to] = match horizontal {
            true => [stem[0], stem[2]],
            false => [stem[1], stem[3]],
        };
        let (a, b) = match horizontal {
            true => (end - k, end + 2.0 * k),
            false => (end + k, end - 2.0 * k),
        };
        assert!(
            (from - a).abs() < 0.2 && (to - b).abs() < 0.2,
            "{stem:?} vs {a} {b}"
        );
        // The axis reaches the range's far end.
        let plot = &drawn.geometry.plot;
        let (start, stop) = match horizontal {
            true => (plot.left, plot.right),
            false => (plot.top, plot.bottom),
        };
        assert!(
            from.min(to) >= start - 0.2 && from.max(to) <= stop + 0.2,
            "{stem:?} outside {start}..{stop}"
        );
    }
}

/// A bar rounds only its free end: A's bars sit under B's, B's are on top or alone.
#[test]
fn a_stack_rounds_only_its_top_segment() {
    for (layout, horizontal, free) in [
        (BarLayout::Stacked, false, TOP),
        (BarLayout::Stacked, true, RIGHT),
        (BarLayout::Percent, false, TOP),
        (BarLayout::Percent, true, RIGHT),
    ] {
        let drawn = draw_bar(&bars(layout, horizontal)).expect("drawn");
        assert!(!drawn.svg.contains("<rect"), "{}", drawn.svg);
        let want = [SQUARE, SQUARE, free, free, free];
        assert_eq!(corners(&drawn.svg), want, "{}", drawn.svg);
    }
}

/// Grouped, every bar rounds its free end.
#[test]
fn grouped_bars_round_only_their_free_end() {
    for (horizontal, free) in [(false, TOP), (true, RIGHT)] {
        let drawn = draw_bar(&bars(BarLayout::Grouped, horizontal)).expect("drawn");
        assert_eq!(corners(&drawn.svg), [free; 5], "{}", drawn.svg);
    }
}

/// A negative value rounds its far end below (left of) the axis; a mixed stack the
/// topmost positive and the lowest negative segment.
#[test]
fn a_negative_bar_rounds_the_end_away_from_the_axis() {
    for (horizontal, up, down) in [(false, TOP, BOTTOM), (true, RIGHT, LEFT)] {
        let spec = BarSpec {
            series: vec![
                spec("A", &[Some(2.0), Some(-1.0), Some(2.0)]),
                spec("B", &[Some(3.0), Some(-2.0), Some(-1.0)]),
            ],
            ..bars(BarLayout::Stacked, horizontal)
        };
        let drawn = draw_bar(&spec).expect("drawn");
        let marks = &drawn.geometry.marks;
        let mut found: Vec<_> = marks
            .iter()
            .zip(corners(&drawn.svg))
            .map(|(mark, corners)| ((mark.series, mark.index), corners))
            .collect();
        found.sort_by_key(|(place, _)| *place);
        let found: Vec<_> = found.into_iter().map(|(_, corners)| corners).collect();
        // A at a, b, c, then B at a, b, c.
        let want = [SQUARE, SQUARE, up, up, down, down];
        assert_eq!(found, want, "{}", drawn.svg);
    }
}

/// The `name` attribute of every `tag` in the SVG that has one.
fn numbers(svg: &str, tag: &str, name: &str) -> Vec<f32> {
    svg.split(tag)
        .skip(1)
        .filter_map(|tag| {
            let tag = &tag[..tag.find('>')?];
            tag.split(&format!(" {name}=\""))
                .nth(1)
                .or_else(|| tag.strip_prefix(&format!("{name}=\"")))?
                .split('"')
                .next()?
                .parse()
                .ok()
        })
        .collect()
}

fn rects(drawn: &Drawn) -> Vec<[f32; 4]> {
    drawn
        .geometry
        .marks
        .iter()
        .map(|mark| match mark.shape {
            MarkShape::Rect {
                x,
                y,
                width,
                height,
            } => [x, y, width, height],
            _ => panic!("{mark:?}"),
        })
        .collect()
}

/// Set again at the renderer's own width and gap, every bar stays where it was drawn.
#[test]
fn a_relayout_at_the_renderer_s_measures_moves_nothing() {
    for (layout, horizontal) in [
        (BarLayout::Grouped, false),
        (BarLayout::Grouped, true),
        (BarLayout::Stacked, false),
    ] {
        let spec = bars(layout, horizontal);
        let drawn = draw_bar(&spec).expect("drawn");
        // The marks are in the renderer's order; it lists a lying chart's x bottom up.
        let marks = &drawn.geometry.marks;
        let triggers: Vec<Trigger> = marks
            .iter()
            .map(|mark| Trigger {
                series: spec.series[mark.series].name.clone(),
                shape: mark.shape.clone(),
            })
            .collect();
        let expected: Vec<_> = marks
            .iter()
            .map(|mark| {
                let index = if horizontal {
                    2 - mark.index
                } else {
                    mark.index
                };
                (mark.series, index, mark.value, "")
            })
            .collect();
        let (_, moved) = relayout(&drawn.svg, triggers.clone(), &expected, &spec);
        for (before, after) in triggers.iter().zip(&moved) {
            let (
                MarkShape::Rect {
                    x: a,
                    y: b,
                    width: c,
                    height: d,
                },
                MarkShape::Rect {
                    x: e,
                    y: f,
                    width: g,
                    height: h,
                },
            ) = (&before.shape, &after.shape)
            else {
                panic!("{before:?}");
            };
            for (old, new) in [(a, e), (b, f), (c, g), (d, h)] {
                assert!(
                    (old - new).abs() < 0.2,
                    "{layout:?} {horizontal}: {before:?} to {after:?}"
                );
            }
        }
    }
}

/// Half its band, each bar of a pair is half that less the gap's half; its label moves
/// with its centre.
#[test]
fn bars_fill_their_share_of_the_band() {
    let mut spec = bars(BarLayout::Grouped, false);
    spec.bar_width = Some(0.5);
    spec.bar_gap = 10.0;
    let wide = draw_bar(&spec).expect("drawn");
    let plot = wide.geometry.plot;
    let unit = (plot.right - plot.left) / 3.0;
    let found = rects(&wide);
    let labels = numbers(&wide.svg, "<text ", "x");
    for [x, _, width, _] in &found {
        assert!((width - (unit * 0.5 - 10.0) / 2.0).abs() < 0.2, "{found:?}");
        let centre = x + width / 2.0;
        assert!(
            labels.iter().any(|label| (label - centre).abs() < 0.2),
            "no label at {centre}: {labels:?}"
        );
    }
    // A of b and B of b: one gap apart.
    let (a, b) = (found[1], found[3]);
    assert!((b[0] - (a[0] + a[2]) - 10.0).abs() < 0.2, "{found:?}");
}

/// A band narrower than the renderer's margins still holds its bars, with a gap set.
#[test]
fn bars_stay_inside_a_narrow_band() {
    let mut narrow = bars(BarLayout::Grouped, false);
    narrow.x = (0..40).map(|i| i.to_string()).collect();
    narrow.series = vec![spec("A", &[Some(1.0); 40]), spec("B", &[Some(2.0); 40])];
    narrow.bar_gap = 2.0;
    let drawn = draw_bar(&narrow).expect("drawn");
    let plot = drawn.geometry.plot;
    let unit = (plot.right - plot.left) / 40.0;
    assert!(
        unit < 2.0 * RENDERER_MARGIN,
        "the band is not narrow: {unit}"
    );
    let found = rects(&drawn);
    assert_eq!(found.len(), 80, "{found:?}");
    for [x, _, width, _] in &found {
        let band = ((x - plot.left) / unit).floor();
        let end = plot.left + unit * (band + 1.0);
        assert!(
            *width > 1.0 && x + width <= end + 0.1,
            "{x}+{width} past {end}"
        );
    }
}

/// A bar's own colour wins over its series'; a negative one's too; its label reads on it.
#[test]
fn a_bar_takes_its_own_colour() {
    let mut spec = bars(BarLayout::Stacked, false);
    let (red, dark) = (HexColor::new(0xE03131), HexColor::new(0x101010));
    spec.bar_colors = vec![vec![None, Some(red)], vec![Some(dark)]];
    let svg = draw_bar(&spec).expect("drawn").svg;
    assert!(svg.contains(&format!("fill=\"{red}\"")), "{svg}");
    assert!(svg.contains(&format!("fill=\"{dark}\"")), "{svg}");
    assert!(
        svg.contains("fill=\"#FFFFFF\""),
        "white on the dark bar: {svg}"
    );
}

/// Upright grouped bars carry their error ranges, the axis reaching the highest end; a
/// percent stack carries none.
#[test]
fn error_bars_stand_on_upright_grouped_bars() {
    let mut spec = bars(BarLayout::Grouped, false);
    spec.value_labels = false;
    spec.errors = vec![vec![Some((0.5, 1.5)), Some((2.0, 9.0))]];
    let drawn = draw_bar(&spec).expect("drawn");
    let [x, _, width, _] = rects(&drawn)[1];
    let centre = x + width / 2.0;
    let upright = numbers(&drawn.svg, "<line ", "x1")
        .into_iter()
        .zip(numbers(&drawn.svg, "<line ", "x2"))
        .any(|(x1, x2)| (x1 - centre).abs() < 0.2 && (x2 - centre).abs() < 0.2);
    assert!(upright, "no error bar at {centre}: {}", drawn.svg);
    assert!(drawn.svg.contains("\n10\n"), "{}", drawn.svg);
    assert!(!drawn.svg.contains("#010203"), "{}", drawn.svg);

    spec.layout = BarLayout::Percent;
    let percent = draw_bar(&spec).expect("drawn").svg;
    assert!(!percent.contains("<line stroke-width=\"1.5\""), "{percent}");
}

/// Outside, a lying bar's label is past its end in the page's ink.
#[test]
fn a_lying_bar_s_label_can_sit_past_its_end() {
    let mut spec = bars(BarLayout::Grouped, true);
    spec.labels_inside = false;
    let drawn = draw_bar(&spec).expect("drawn");
    assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
    assert_eq!(overflow(&drawn.svg, 320.0, 180.0), None, "{}", drawn.svg);
}

fn pie(labels: PieLabels, donut: Option<f32>) -> PieSpec {
    PieSpec {
        width: 320.0,
        height: 320.0,
        slices: vec![spec("A", &[Some(1.0)]), spec("B", &[Some(3.0)])],
        donut,
        labels,
        start_angle: 0.0,
        end_angle: None,
        rose: false,
        slice_gap: 1.0,
        slice_radius: 0.0,
        labels_inside: false,
        min_label_angle: 0.0,
        ink: ChartInk::of(&Theme::DEFAULT, false),
        format: ValueFormat::Auto,
        unit: Some("kg".to_string()),
        formats: &Formats::GERMAN,
    }
}

/// A slice's corners are circular fillets of the radius, tangent to the edges and the
/// arcs, in the drawing and in what the pointer hits (3479).
#[test]
fn a_slice_s_corners_are_rounded_by_the_radius() {
    for (donut, fillets) in [(None, 2), (Some(0.4), 4)] {
        let mut spec = pie(PieLabels::None, donut);
        spec.slice_radius = 8.0;
        let drawn = draw_pie(&spec).expect("drawn");
        assert_eq!(drawn.geometry.marks.len(), 2);
        for mark in &drawn.geometry.marks {
            let MarkShape::Sector { path } = &mark.shape else {
                panic!("a slice")
            };
            assert_eq!(path.matches("A8,8 ").count(), fillets, "{path}");
            assert!(!path.contains("A0 0"), "{path}");
            assert!(drawn.svg.contains(&format!("d=\"{path}\"")), "{path}");
        }
    }
}

/// A thin slice gives up corner radius rather than fold over itself.
#[test]
fn a_thin_slice_keeps_its_corners_inside_it() {
    let mut spec = pie(PieLabels::None, None);
    spec.slices = vec![
        self::spec("A", &[Some(1.0)]),
        self::spec("B", &[Some(99.0)]),
    ];
    spec.slice_radius = 40.0;
    let drawn = draw_pie(&spec).expect("drawn");
    let MarkShape::Sector { path } = &drawn.geometry.marks[0].shape else {
        panic!("a slice")
    };
    let radii: Vec<f32> = path
        .split(" A")
        .skip(1)
        .filter_map(|a| a.split(',').next()?.parse().ok())
        .collect();
    assert!(radii[0] < 4.0, "{path}");
}

/// The first slice starts at 12 o'clock and runs clockwise; the labels are in the locale.
#[test]
fn a_pie_starts_at_the_top_and_writes_its_labels() {
    let drawn = draw_pie(&pie(PieLabels::Percent, None)).expect("drawn");
    let marks = &drawn.geometry.marks;

    let places: Vec<_> = marks.iter().map(|m| (m.series, m.index)).collect();
    assert_eq!(places, [(0, 0), (0, 1)]);
    let MarkShape::Sector { path } = &marks[0].shape else {
        panic!("{marks:?}")
    };
    // A quarter from the top, clockwise: from the centre straight up, then to the right.
    let point = |token: &str| -> (f32, f32) {
        let (x, y) = token.split_once(',').expect(path);
        (x.parse().expect(path), y.parse().expect(path))
    };
    let tokens: Vec<_> = path.split(' ').collect();
    let (top, end) = (point(&tokens[1][1..]), point(tokens[tokens.len() - 3]));
    assert!((top.0 - 160.0).abs() < 0.5 && top.1 < 160.0, "{path}");
    assert!(end.0 > 160.0 && (end.1 - 160.0).abs() < 0.5, "{path}");
    assert!(
        drawn.svg.contains("25%") && drawn.svg.contains("75%"),
        "{}",
        drawn.svg
    );

    let values = draw_pie(&pie(PieLabels::Value, None)).expect("drawn");
    assert!(values.svg.contains("3 kg"), "{}", values.svg);
    let none = draw_pie(&pie(PieLabels::None, Some(0.4))).expect("drawn");
    assert!(!none.svg.contains("<text"), "{}", none.svg);
}

fn sector_points(mark: &Mark) -> Vec<(f32, f32)> {
    match &mark.shape {
        MarkShape::Sector { path } => super::super::hit::outline(path).concat(),
        _ => panic!("{mark:?}"),
    }
}

/// A half donut from 9 to 3 o'clock lies above its middle and takes a larger radius
/// than a full one; its `center` room is the upper half of the hole.
#[test]
fn a_half_donut_bends_over_its_middle() {
    let mut full = pie(PieLabels::None, Some(0.4));
    full.height = 180.0;
    let mut half = full.clone();
    let full = draw_pie(&full).expect("drawn");
    (half.start_angle, half.end_angle) = (-90.0, Some(90.0));
    let half = draw_pie(&half).expect("drawn");
    let points: Vec<(f32, f32)> = half.geometry.marks.iter().flat_map(sector_points).collect();
    let (low, high) = points.iter().fold((f32::MAX, f32::MIN), |(low, high), p| {
        (low.min(p.1), high.max(p.1))
    });
    let room = half.geometry.plot;
    let middle_y = room.bottom;
    assert!(high <= middle_y + 0.5, "{high} below {middle_y}");
    assert!(half.geometry.hole > full.geometry.hole, "{half:?}");
    assert!(high - low > full.geometry.hole, "{low}..{high}");
    assert!(
        (room.right - room.left - half.geometry.hole).abs() < 0.5,
        "{room:?}"
    );
}

/// An end before the start counts a turn on: 3 o'clock to 9 o'clock is the lower half.
#[test]
fn an_end_before_the_start_wraps_round() {
    let mut wrapped = pie(PieLabels::None, Some(0.4));
    (wrapped.start_angle, wrapped.end_angle) = (90.0, Some(-90.0));
    let mut lower = wrapped.clone();
    lower.end_angle = Some(270.0);
    assert_eq!(wrapped.span(), 180.0);
    let (wrapped, lower) = (draw_pie(&wrapped), draw_pie(&lower));
    assert_eq!(
        wrapped.map(|d| d.geometry.marks),
        lower.map(|d| d.geometry.marks)
    );
}

/// A rose gives every slice the same angle and the larger value the longer radius.
#[test]
fn a_rose_s_slices_reach_out_by_value() {
    let mut rose = pie(PieLabels::None, None);
    rose.rose = true;
    let drawn = draw_pie(&rose).expect("drawn");
    let reach = |mark: &Mark| {
        sector_points(mark)
            .iter()
            .map(|(x, y)| (x - 160.0).hypot(y - 160.0))
            .fold(0.0, f32::max)
    };
    let marks = &drawn.geometry.marks;
    assert!(reach(&marks[1]) > reach(&marks[0]) + 10.0, "{marks:?}");
    // A, the smaller value, still spans half the turn: it reaches 6 o'clock.
    assert!(
        sector_points(&marks[0])
            .iter()
            .any(|(x, y)| (x - 160.0).abs() < 1.0 && *y > 160.0),
        "{marks:?}"
    );
}

/// On its slice a label takes the ink that reads there; a slice narrower than the
/// least angle has none. The slices part by the gap.
#[test]
fn labels_inside_read_on_their_slice() {
    let mut inside = pie(PieLabels::Percent, None);
    inside.labels_inside = true;
    inside.slice_gap = 4.0;
    let drawn = draw_pie(&inside).expect("drawn");
    let on = inside.ink.on(inside.slices[1].color);
    assert!(
        drawn.svg.contains(&format!("fill=\"{on}\"")),
        "{}",
        drawn.svg
    );
    assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
    assert!(drawn.svg.contains("stroke-width=\"4\""), "{}", drawn.svg);

    inside.min_label_angle = 100.0;
    let svg = draw_pie(&inside).expect("drawn").svg;
    assert!(svg.contains("75%") && !svg.contains("25%"), "{svg}");
}

/// Of two outside labels that overlap, the smaller slice's leaves with its leader line;
/// the largest slice keeps its label and no two left overlap.
#[test]
fn colliding_outside_labels_keep_the_larger_slice_s() {
    let mut many = pie(PieLabels::Percent, None);
    (many.width, many.height) = (304.0, 168.0);
    let values: Vec<f64> = (1..=14).map(f64::from).collect();
    many.slices = values
        .iter()
        .map(|value| spec(&format!("S{value}"), &[Some(*value)]))
        .collect();
    let svg = draw_pie(&many).expect("drawn").svg;
    let labels = text_contents(&svg);
    assert!(labels.len() < 14, "{svg}");
    assert!(
        labels.iter().any(|label| label.starts_with("13,3")),
        "{svg}"
    );
    assert_eq!(
        svg.matches("fill=\"none\"").count(),
        labels.len(),
        "a leader line per label: {svg}"
    );
    assert_eq!(drop_colliding_labels(&svg, &values), svg);
}

/// A slice that takes the whole pie, beside empty ones, shows no seam up to 12 o'clock.
#[test]
fn a_lone_slice_has_no_seam() {
    let mut lone = pie(PieLabels::Percent, None);
    lone.slice_gap = 4.0;
    lone.slices = vec![spec("A", &[Some(0.0)]), spec("B", &[Some(3.0)])];
    let svg = draw_pie(&lone).expect("drawn").svg;
    assert!(!svg.contains("stroke-width=\"4\""), "{svg}");
    assert!(svg.contains("stroke-width=\"0\""), "{svg}");
}

/// A label inside sits on its own slice: not in a thin donut's hole, nor by a rose's
/// small slice's centre.
#[test]
fn labels_inside_stay_on_their_slice() {
    for rose in [false, true] {
        let mut inside = pie(PieLabels::Percent, Some(0.25));
        inside.slices = vec![
            spec("A", &[Some(60.0)]),
            spec("B", &[Some(25.0)]),
            spec("C", &[Some(15.0)]),
        ];
        inside.rose = rose;
        inside.labels_inside = true;
        let drawn = draw_pie(&inside).expect("drawn");
        let outlines: Vec<_> = drawn
            .geometry
            .marks
            .iter()
            .map(|mark| match &mark.shape {
                MarkShape::Sector { path } => super::super::hit::outline(path),
                _ => panic!("{mark:?}"),
            })
            .collect();
        let middles: Vec<(f32, f32)> = numbers(&drawn.svg, "<text ", "x")
            .into_iter()
            .zip(numbers(&drawn.svg, "<text ", "y"))
            .map(|(x, y)| (x, y - 14.0 * 0.35))
            .collect();
        // The rose's smallest slice is a ring thinner than its label: that one is dropped.
        assert_eq!(middles.len(), if rose { 2 } else { 3 }, "{}", drawn.svg);
        for (slice, (x, y)) in middles.iter().enumerate() {
            assert!(
                super::super::hit::contains(&outlines[slice], *x, *y),
                "rose {rose}: label {slice} at {x},{y} off its slice: {}",
                drawn.svg
            );
        }
    }
}

/// A label wider than its slice is dropped rather than spill onto the next one.
#[test]
fn a_label_wider_than_its_slice_is_dropped() {
    let mut inside = pie(PieLabels::Name, None);
    inside.labels_inside = true;
    inside.slices = [("Chrome", 85.0), ("Internet Explorer", 15.0)]
        .map(|(name, value)| spec(name, &[Some(value)]))
        .to_vec();
    let drawn = draw_pie(&inside).expect("drawn");
    assert!(drawn.svg.contains("Chrome"), "{}", drawn.svg);
    assert!(!drawn.svg.contains("Internet Explorer"), "{}", drawn.svg);
    assert_eq!(drawn.geometry.marks.len(), 2, "{:?}", drawn.geometry);
}

/// On a half pie, the 10% label fits its slice's width but not its height: it would
/// reach onto the next slice, so it is dropped.
#[test]
fn a_label_taller_than_its_slice_is_dropped() {
    let mut half = pie(PieLabels::Percent, None);
    (half.width, half.height) = (304.0, 168.0);
    (half.start_angle, half.end_angle) = (-90.0, Some(90.0));
    half.labels_inside = true;
    half.slices = [("A", 62.0), ("B", 20.0), ("C", 10.0), ("D", 8.0)]
        .map(|(name, value)| spec(name, &[Some(value)]))
        .to_vec();
    let svg = draw_pie(&half).expect("drawn").svg;
    let labels = text_contents(&svg);
    assert!(labels.iter().any(|label| label.starts_with("62")), "{svg}");
    assert!(!labels.iter().any(|label| label.starts_with("10")), "{svg}");
}

/// A donut's ring holds the labels of its wide slices: a box's corner just past the ring,
/// where no glyph reaches, keeps its label.
#[test]
fn a_ring_keeps_its_wide_slices_labels() {
    let mut ring = pie(PieLabels::Percent, Some(0.4));
    (ring.width, ring.height) = (304.0, 168.0);
    ring.labels_inside = true;
    ring.slices = [("A", 62.0), ("B", 20.0), ("C", 10.0), ("D", 8.0)]
        .map(|(name, value)| spec(name, &[Some(value)]))
        .to_vec();
    let svg = draw_pie(&ring).expect("drawn").svg;
    let labels = text_contents(&svg);
    assert!(
        ["62", "20"]
            .iter()
            .all(|share| labels.iter().any(|label| label.starts_with(share))),
        "{svg}"
    );
}

fn text_contents(svg: &str) -> Vec<&str> {
    svg.split("<text ")
        .skip(1)
        .filter_map(|text| Some(text.split_once('>')?.1.split('<').next()?.trim()))
        .collect()
}

/// Every label keeps inside the SVG: a pie's beside its slices on a phone, a bar's
/// above the tallest bar, the top y label. The pie stays in the middle.
#[test]
fn every_label_keeps_inside_the_svg() {
    let mut narrow = pie(PieLabels::Percent, None);
    (narrow.width, narrow.height) = (280.0, 180.0);
    narrow.slices = [
        ("Chrome", 62.0),
        ("Safari", 20.0),
        ("Firefox", 10.0),
        ("Other", 8.0),
    ]
    .map(|(name, value)| spec(name, &[Some(value)]))
    .to_vec();
    let lines = line(vec![spec("A", &[Some(12.0), Some(180.0)])], &["Mon", "Tue"]);
    for (drawn, width, height) in [
        (draw_pie(&narrow), 280.0, 180.0),
        (draw_bar(&bars(BarLayout::Grouped, false)), 320.0, 180.0),
        (draw_line(&lines), 320.0, 180.0),
    ] {
        let drawn = drawn.expect("drawn");
        assert_eq!(overflow(&drawn.svg, width, height), None, "{}", drawn.svg);
    }
    let drawn = draw_pie(&narrow).expect("drawn");
    let MarkShape::Sector { path } = &drawn.geometry.marks[0].shape else {
        panic!("{drawn:?}")
    };
    assert!(path.starts_with("M140,90 "), "{path}");
}

/// A name too long to fit beside a pie at its least radius ends in an ellipsis.
#[test]
fn a_long_name_beside_a_pie_is_cut_short() {
    let mut small = pie(PieLabels::Name, None);
    (small.width, small.height) = (288.0, 162.0);
    small.slices = [
        ("Chrome", 50.0),
        ("Internet Explorer", 30.0),
        ("Safari & Co", 20.0),
    ]
    .map(|(name, value)| spec(name, &[Some(value)]))
    .to_vec();
    let drawn = draw_pie(&small).expect("drawn");
    assert_eq!(overflow(&drawn.svg, 288.0, 162.0), None, "{}", drawn.svg);
    assert!(drawn.svg.contains('…'), "{}", drawn.svg);
    assert!(drawn.svg.contains(">\nChrome\n<"), "{}", drawn.svg);
}

/// The plot is the box inside the axes, grid or not: above the x labels, right of
/// the y labels, and every mark in it.
#[test]
fn the_plot_is_the_box_inside_the_axes() {
    let mut lines = line(vec![spec("A", &[Some(1.0), Some(3.0)])], &["Mon", "Tue"]);
    lines.grid = false;
    let mut lying = bars(BarLayout::Grouped, true);
    lying.grid = false;
    for geometry in [
        draw_line(&lines).expect("drawn").geometry,
        draw_bar(&bars(BarLayout::Stacked, false))
            .expect("drawn")
            .geometry,
        draw_bar(&lying).expect("drawn").geometry,
    ] {
        let plot = geometry.plot;
        assert!(plot.left > 10.0 && plot.top > 0.0, "{plot:?}");
        assert!(plot.bottom < geometry.height - 10.0, "{plot:?}");
        assert!(plot.right <= geometry.width, "{plot:?}");
        for mark in &geometry.marks {
            let (x, y) = hit_anchor(&mark.shape);
            assert!(
                (plot.left..=plot.right).contains(&x) && (plot.top..=plot.bottom).contains(&y),
                "{mark:?} {plot:?}"
            );
        }
    }
}

/// Lying, the value labels under the plot, written again grouped, stay centred on
/// their ticks.
#[test]
fn lying_value_labels_sit_on_their_ticks() {
    let mut lying = bars(BarLayout::Grouped, true);
    lying.series = vec![spec("A", &[Some(12_400.0), Some(31_000.0), Some(8_000.0)])];
    let svg = draw_bar(&lying).expect("drawn").svg;
    let ticks: Vec<f32> = svg
        .split("<line ")
        .skip(1)
        .filter_map(|tag| {
            let number = |name: &str| -> Option<f32> {
                tag.split(&format!("{name}=\""))
                    .nth(1)?
                    .split('"')
                    .next()?
                    .parse()
                    .ok()
            };
            let (x1, x2, y1, y2) = (number("x1")?, number("x2")?, number("y1")?, number("y2")?);
            (x1 == x2 && y2 - y1 == 5.0).then_some(x1)
        })
        .collect();
    let labels: Vec<f32> = svg
        .split("<text x=\"")
        .skip(1)
        .filter(|tag| tag.contains("text-anchor=\"middle\""))
        .filter_map(|tag| tag.split('"').next()?.parse().ok())
        .collect();
    assert!(labels.len() > 2 && svg.contains("25.000"), "{svg}");
    // The last one ends at the SVG's edge rather than past it.
    let (last, inner) = labels.split_last().expect("labels");
    for label in inner {
        assert!(
            ticks.iter().any(|tick| (tick - label).abs() < 1.0),
            "{label} off {ticks:?}"
        );
    }
    assert!(
        *last > inner[inner.len() - 1] && *last < 320.0 - 15.0,
        "{last}"
    );
}

fn hit_anchor(shape: &MarkShape) -> (f32, f32) {
    match *shape {
        MarkShape::Point { x, y } => (x, y),
        MarkShape::Rect {
            x,
            y,
            width,
            height,
        } => (x + width / 2.0, y + height / 2.0),
        MarkShape::Sector { .. } => panic!("{shape:?}"),
    }
}

/// Contract: a slice is a trigger `path`, one per series.
#[test]
fn a_pie_slice_is_harvested_as_its_path() {
    let mut chart = charts_rs::PieChart::new(vec![
        charts_rs::Series::new("A".to_string(), vec![1.0]),
        charts_rs::Series::new("B".to_string(), vec![3.0]),
    ]);
    chart.tooltip.show = true;
    let (_, triggers) = sanitize(&chart.svg().expect("svg"), |v| v.to_string());

    let names: Vec<_> = triggers.iter().map(|t| t.series.as_str()).collect();
    assert_eq!(names, ["A", "B"]);
    assert!(
        triggers
            .iter()
            .all(|t| matches!(&t.shape, MarkShape::Sector { path } if path.starts_with('M')))
    );
}

/// Each text's `(left, right, baseline, label)`, `dx` and `dy` applied.
fn texts(svg: &str) -> Vec<(f32, f32, f32, String)> {
    let mut found = Vec::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let close = open + rest[open..].find('>').expect("closed");
        let attributes = attributes(&rest[open..=close]);
        let label = &rest[close + 1..];
        let label = label[..label.find('<').unwrap_or(label.len())].trim();
        rest = &rest[close + 1..];
        let number = |name| attribute(&attributes, name)?.parse::<f32>().ok();
        let (Some(x), Some(y)) = (number("x"), number("y")) else {
            continue;
        };
        let (x, y) = (
            x + number("dx").unwrap_or(0.0),
            y + number("dy").unwrap_or(0.0),
        );
        let drawn = text_width(&attributes, label);
        let left = match attribute(&attributes, "text-anchor") {
            Some("middle") => x - drawn / 2.0,
            Some("end") => x - drawn,
            _ => x,
        };
        found.push((left, left + drawn, y, label.to_string()));
    }
    found
}

#[test]
fn the_last_numeric_x_label_keeps_inside_the_svg() {
    let mut numeric = line(
        vec![spec("A", &[Some(3.0), Some(5.0), Some(4.0), Some(8.0)])],
        &["1", "2", "10", "1000.5"],
    );
    numeric.x_numbers = vec![1.0, 2.0, 10.0, 1000.5];
    (numeric.width, numeric.height) = (480.0, 272.0);
    let drawn = draw_line(&numeric).expect("drawn");
    assert_eq!(
        overflow(&drawn.svg, 480.0, 272.0),
        None,
        "{:?}",
        texts(&drawn.svg)
    );
    assert!(
        texts(&drawn.svg).iter().any(|t| t.3 == "1000"),
        "{}",
        drawn.svg
    );
}

#[test]
fn a_numeric_x_axis_ends_past_the_last_x() {
    let mut numeric = line(
        vec![spec("A", &[Some(3.0), Some(5.0), Some(4.0), Some(8.0)])],
        &["1", "2", "10", "1000.5"],
    );
    numeric.x_numbers = vec![1.0, 2.0, 10.0, 1000.5];
    let drawn = draw_line(&numeric).expect("drawn");
    let Some(MarkShape::Point { x, .. }) = drawn.geometry.marks.last().map(|m| &m.shape) else {
        panic!("no point");
    };
    assert!(*x < drawn.geometry.plot.right - 1.0, "{x}");
}

#[test]
fn a_mark_line_label_keeps_off_the_value_labels() {
    let mut spec = bars(BarLayout::Grouped, false);
    spec.x = ["Q1", "Q2", "Q3", "Q4"].map(String::from).to_vec();
    let mut series = self::spec("A", &[Some(10.0), Some(50.0), Some(30.0), Some(33.0)]);
    series.extras.mark_lines = vec![(31.0, "Average: 31".to_string())];
    spec.series = vec![series];
    let found = texts(&draw_bar(&spec).expect("drawn").svg);
    let label = found.iter().find(|t| t.3 == "Average: 31").expect("label");
    for other in found.iter().filter(|t| t.3 != label.3) {
        let across = label.0 < other.1 && label.1 > other.0;
        assert!(
            !(across && (label.2 - other.2).abs() < 12.0),
            "{label:?} {other:?}"
        );
    }
}

/// Todo 3747 (F14-02): the label sat at the plot's end, over the rising series line.
#[test]
fn a_mark_line_label_keeps_off_the_series_lines() {
    let mut series = spec(
        "A",
        &[120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0].map(Some),
    );
    series.extras.mark_lines = vec![(145.29, "Average: 145,29".to_string())];
    let drawn = draw_line(&line(
        vec![series],
        &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    ))
    .expect("drawn");
    let found = texts(&drawn.svg);
    let label = found.iter().find(|t| t.3.starts_with("Average")).unwrap();
    let points: Vec<(f32, f32)> = drawn
        .geometry
        .marks
        .iter()
        .filter_map(|mark| match mark.shape {
            MarkShape::Point { x, y } => Some((x, y)),
            _ => None,
        })
        .collect();
    for pair in points.windows(2) {
        for step in 0..=20 {
            let t = step as f32 / 20.0;
            let x = pair[0].0 + (pair[1].0 - pair[0].0) * t;
            let y = pair[0].1 + (pair[1].1 - pair[0].1) * t;
            let inside = x > label.0 && x < label.1 && y > label.2 - 11.0 && y < label.2 + 3.0;
            assert!(!inside, "{label:?} crossed at ({x}, {y})");
        }
    }
}

#[test]
fn dense_x_labels_leave_room_for_a_wider_font() {
    let x: Vec<String> = (1..=40).map(|i| format!("Week {i}")).collect();
    let values: Vec<_> = (1..=40).map(|i| Some(f64::from(i))).collect();
    let spec = line(
        vec![spec("A", &values)],
        &x.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    let found = texts(&draw_line(&spec).expect("drawn").svg);
    let mut row: Vec<_> = found.iter().filter(|t| t.3.starts_with("Week")).collect();
    row.sort_by(|a, b| a.0.total_cmp(&b.0));
    for pair in row.windows(2) {
        let gap = pair[1].0 - pair[0].1;
        let widest = (pair[0].1 - pair[0].0).max(pair[1].1 - pair[1].0);
        assert!(gap >= widest * 0.15, "{pair:?}");
    }
}

#[test]
fn a_bar_s_label_writes_its_own_value_not_the_renderer_s() {
    let mut huge = bars(BarLayout::Grouped, false);
    huge.series = vec![spec("A", &[Some(1.2e9), Some(3.4e9), Some(9.9e9)])];
    let drawn = draw_bar(&huge).expect("drawn");
    let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
    assert!(labels.contains(&"9.900.000.000".to_string()), "{labels:?}");
}

/// A long category label is cut to its band; bands too narrow for any word show every
/// n-th label. Neither runs past the SVG nor over its neighbour.
#[test]
fn category_labels_keep_to_their_bands() {
    let row = |chart: &BarSpec| {
        let drawn = draw_bar(chart).expect("drawn");
        let plot = drawn.geometry.plot;
        let below = texts(&drawn.svg).into_iter();
        let row: Vec<_> = below.filter(|t| t.2 > plot.bottom + PLOT_TICK).collect();
        for pair in row.windows(2) {
            assert!(pair[0].1 <= pair[1].0, "{row:?}");
        }
        assert!(
            row.iter().all(|t| t.0 >= 0.0 && t.1 <= chart.width),
            "{row:?}"
        );
        (row, (plot.right - plot.left) / chart.x.len() as f32)
    };
    let mut long = bars(BarLayout::Grouped, false);
    long.x = ["A very long category label", "Another long one", "Short"]
        .map(String::from)
        .to_vec();
    let (labels, band) = row(&long);
    assert_eq!(labels.len(), 3, "{labels:?}");
    assert!(
        labels[0].3.ends_with('…') && labels[2].3 == "Short",
        "{labels:?}"
    );
    assert!(
        labels.iter().all(|t| t.1 - t.0 <= band),
        "{labels:?} {band}"
    );

    let mut many = bars(BarLayout::Grouped, false);
    many.x = (1..=40).map(|i| format!("Week {i}")).collect();
    many.series = vec![spec("A", &[Some(1.0); 40])];
    let (labels, _) = row(&many);
    assert!(labels.len() > 1 && labels.len() < 40, "{labels:?}");
    // Thinned, they have room to be written whole; the last may meet the SVG's side.
    let (_, inner) = labels.split_last().expect("labels");
    assert!(inner.iter().all(|t| t.3.starts_with("Week ")), "{labels:?}");
}

#[test]
fn a_falling_bar_s_label_sits_past_its_lower_end() {
    let mut falling = bars(BarLayout::Grouped, false);
    falling.series = vec![spec("A", &[Some(-10.0), Some(-200.0), Some(35.0)])];
    let drawn = draw_bar(&falling).expect("drawn");
    let left = drawn.geometry.plot.left;
    let found: Vec<_> = texts(&drawn.svg)
        .into_iter()
        .filter(|t| t.0 > left)
        .collect();
    let baseline = |value: &str| found.iter().find(|t| t.3 == value).expect(value).2;
    let ends: Vec<_> = (drawn.geometry.marks.iter())
        .map(|mark| match mark.shape {
            MarkShape::Rect { y, height, .. } => (y, y + height),
            _ => panic!("{mark:?}"),
        })
        .collect();
    assert!(baseline("-10") - 10.0 > ends[0].1, "{found:?} {ends:?}");
    // The lowest bar meets the x axis: its label goes inside, on the bar's ink.
    let lowest = baseline("-200");
    assert!(
        lowest > ends[1].0 && lowest < ends[1].1,
        "{found:?} {ends:?}"
    );
    assert!(baseline("35") < ends[2].0, "{found:?} {ends:?}");
    assert!(!drawn.svg.contains(ON_BAR_FILL), "{}", drawn.svg);
    // The two labels past their bars are haloed, the one inside is not.
    assert_eq!(
        drawn.svg.matches("paint-order=\"stroke\"").count(),
        2,
        "{}",
        drawn.svg
    );
}

#[test]
fn a_mark_line_label_wider_than_the_plot_is_left_out() {
    let mut series = spec("A", &[Some(1.0), Some(3.0), Some(2.0)]);
    series.extras.mark_lines = vec![(2.0, "Average over the whole week: 2".into())];
    let mut narrow = line(vec![series], &["a", "b", "c"]);
    let drawn = draw_line(&narrow).expect("drawn");
    assert!(drawn.svg.contains("Average over"), "{}", drawn.svg);
    (narrow.width, narrow.height) = (160.0, 120.0);
    let drawn = draw_line(&narrow).expect("drawn");
    assert!(!drawn.svg.contains("Average over"), "{}", drawn.svg);
    assert_eq!(drawn.svg.matches("stroke-dasharray=\"4,2\"").count(), 1);
}

#[test]
fn bars_keep_half_a_narrow_band_by_default() {
    let mut narrow = bars(BarLayout::Grouped, false);
    narrow.x = (0..40).map(|i| i.to_string()).collect();
    narrow.series = vec![spec("A", &[Some(1.0); 40])];
    (narrow.width, narrow.height) = (480.0, 272.0);
    let drawn = draw_bar(&narrow).expect("drawn");
    let plot = drawn.geometry.plot;
    let unit = (plot.right - plot.left) / 40.0;
    for [_, _, width, _] in rects(&drawn) {
        assert!(width >= unit / 2.0 - 0.1, "{width} of {unit}");
    }
}

#[test]
fn dense_x_labels_do_not_overlap() {
    let x: Vec<String> = (0..200).map(|i| i.to_string()).collect();
    let x: Vec<&str> = x.iter().map(String::as_str).collect();
    let mut dense = line(vec![spec("A", &[Some(1.0); 200])], &x);
    (dense.width, dense.height) = (480.0, 272.0);
    let drawn = draw_line(&dense).expect("drawn");
    let bottom = drawn.geometry.plot.bottom;
    let mut labels: Vec<_> = texts(&drawn.svg)
        .into_iter()
        .filter(|(.., y, _)| *y > bottom)
        .collect();
    labels.sort_by(|a, b| a.0.total_cmp(&b.0));
    for pair in labels.windows(2) {
        assert!(pair[0].1 <= pair[1].0, "{pair:?}");
    }
}

fn heatmap(values: Vec<Vec<Option<f64>>>) -> HeatmapSpec {
    HeatmapSpec {
        width: 480.0,
        height: 240.0,
        x: (0..values[0].len()).map(|c| format!("C{c}")).collect(),
        y: (0..values.len()).map(|r| format!("R{r}")).collect(),
        values,
        colors: vec![HexColor::new(0xEEEEFF), HexColor::new(0x2233AA)],
        range: (0.0, 10.0),
        steps: 0,
        labels: true,
        scale: false,
        no_value: None,
        ink: ChartInk::of(&Theme::DEFAULT, false),
        format: ValueFormat::Auto,
        unit: None,
        formats: &Formats::AMERICAN,
    }
}

/// Cells are harvested row by row from the top, an empty one left out and outlined.
#[test]
fn a_heatmap_harvests_its_cells_row_by_row_from_the_top() {
    let drawn = draw_heatmap(&heatmap(vec![
        vec![Some(1.0), Some(2.0), None],
        vec![Some(9.0), Some(5.5), Some(10.0)],
    ]))
    .expect("drawn");
    let cells: Vec<_> = drawn
        .geometry
        .marks
        .iter()
        .map(|m| (m.series, m.index, m.value))
        .collect();
    assert_eq!(
        cells,
        [
            (0, 0, 1.0),
            (0, 1, 2.0),
            (1, 0, 9.0),
            (1, 1, 5.5),
            (1, 2, 10.0)
        ]
    );
    let top = |mark: &Mark| match mark.shape {
        MarkShape::Rect { y, .. } => y,
        _ => panic!("{mark:?}"),
    };
    assert!(top(&drawn.geometry.marks[0]) < top(&drawn.geometry.marks[2]));
    assert!(drawn.geometry.cells);
    assert!(
        drawn.svg.contains("stroke-dasharray=\"4,3\""),
        "{}",
        drawn.svg
    );
    assert!(!drawn.svg.contains("#6E7079") && !drawn.svg.contains("data-"));
}

/// The scale sits under the plot, in classes with `steps`, its ends' values below it.
#[test]
fn a_heatmap_draws_its_scale_under_the_plot() {
    let mut spec = heatmap(vec![vec![Some(0.0), Some(10.0)]]);
    spec.scale = true;
    spec.steps = 5;
    let drawn = draw_heatmap(&spec).expect("drawn");
    assert!(
        drawn.svg.contains("viewBox=\"0 0 480 240\""),
        "{}",
        drawn.svg
    );
    let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
    for value in ["0", "2", "4", "6", "8", "10"] {
        assert!(labels.contains(&value.to_string()), "{labels:?}");
    }
    assert_eq!(
        drawn.svg.matches("height=\"10\"").count(),
        5,
        "{}",
        drawn.svg
    );
    let bottom = drawn.geometry.plot.bottom;
    assert!(bottom <= 240.0 - SCALE_ROOM + 1.0, "{bottom}");

    spec.values = vec![vec![Some(0.0), None]];
    spec.no_value = Some("No value".to_string());
    let drawn = draw_heatmap(&spec).expect("drawn");
    let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
    assert!(labels.contains(&"No value".to_string()), "{labels:?}");
    assert_eq!(drawn.svg.matches("stroke-dasharray").count(), 2);
}

/// Each cell writes its value at its middle; none without labels or in a narrow cell.
#[test]
fn a_heatmap_writes_values_that_fit_their_cells() {
    let mut spec = heatmap(vec![vec![Some(1.5), Some(10.0)]]);
    spec.unit = Some("°C".to_string());
    let labels = |spec: &HeatmapSpec| -> Vec<String> {
        let drawn = draw_heatmap(spec).expect("drawn");
        texts(&drawn.svg)
            .into_iter()
            .map(|t| t.3)
            .filter(|t| !t.starts_with('C') && !t.starts_with('R'))
            .collect()
    };
    assert_eq!(labels(&spec), ["1.5 °C", "10 °C"]);
    spec.labels = false;
    assert!(labels(&spec).is_empty());
    spec.labels = true;
    spec.width = 120.0;
    spec.unit = Some("kilowatt hours".to_string());
    assert!(labels(&spec).is_empty());
}

fn candles(ohlc: bool) -> CandleSpec {
    CandleSpec {
        width: 400.0,
        height: 300.0,
        x: ["Mon", "Tue", "Wed", "Thu"].map(String::from).to_vec(),
        name: "Price".to_string(),
        candles: vec![
            Some([20.0, 38.0, 10.0, 34.0]),
            Some([40.0, 50.0, 30.0, 35.0]),
            None,
            Some([30.0, 36.0, 28.0, 30.0]),
        ],
        up: HexColor::new(0x2E7D32),
        down: HexColor::new(0xC62828),
        ohlc,
        key: Some(("Up".to_string(), "Down".to_string())),
        axes: axes(),
        grid: true,
        ink: ChartInk::of(&Theme::DEFAULT, false),
        format: ValueFormat::Auto,
        unit: None,
        formats: &Formats::AMERICAN,
    }
}

/// A rising candle is an outline on the surface, a falling one filled, a flat one a line;
/// each mark spans its wick and a gap has none.
#[test]
fn a_rising_candle_is_hollow_and_a_falling_one_filled() {
    let drawn = draw_candles(&candles(false)).expect("drawn");
    let surface = ChartInk::of(&Theme::DEFAULT, false).surface;
    assert!(
        drawn
            .svg
            .contains(&format!("fill=\"{surface}\" stroke=\"#2E7D32\""))
    );
    assert!(drawn.svg.contains("stroke=\"#C62828\" fill=\"#C62828\""));
    assert!(drawn.svg.contains("height=\"1\" fill=\"#2E7D32\""));
    assert!(!drawn.svg.contains(UP_HEX));
    let marks = &drawn.geometry.marks;
    let found: Vec<_> = marks.iter().map(|m| (m.index, m.value)).collect();
    assert_eq!(found, [(0, 34.0), (1, 35.0), (3, 30.0)]);
    let MarkShape::Rect { y, height, .. } = marks[0].shape else {
        panic!("a rect")
    };
    // From the high, 38, to the low, 10, on an axis from 10 to 50.
    let plot = drawn.geometry.plot;
    let at = |price: f32| plot.bottom - (price - 10.0) / 40.0 * (plot.bottom - plot.top);
    assert!((y - at(38.0)).abs() < 0.5 && (y + height - at(10.0)).abs() < 0.5);
}

/// OHLC bars draw their ticks in the rising and falling colours, no stand-in left.
#[test]
fn ohlc_bars_tick_their_open_and_close() {
    let drawn = draw_candles(&candles(true)).expect("drawn");
    assert!(!drawn.svg.contains(UP_HEX));
    let up = drawn.svg.matches("stroke=\"#2E7D32\"").count();
    let down = drawn.svg.matches("stroke=\"#C62828\"").count();
    // A line and two ticks per bar, two rising (one flat) and one falling; a key each.
    assert_eq!((up, down), (7, 4), "{}", drawn.svg);
    assert_eq!(drawn.geometry.marks.len(), 3);
}

fn funnel(labels: PieLabels, values: &[f64]) -> FunnelSpec {
    FunnelSpec {
        width: 480.0,
        height: 320.0,
        stages: values
            .iter()
            .enumerate()
            .map(|(i, v)| spec(&format!("S{i}"), &[Some(*v)]))
            .collect(),
        labels,
        first: values.first().copied().unwrap_or(0.0),
        ink: ChartInk::of(&Theme::DEFAULT, false),
        format: ValueFormat::Auto,
        unit: None,
        formats: &Formats::AMERICAN,
    }
}

/// Stages are drawn widest first, each under its own anchor, and walked in data order.
#[test]
fn a_funnel_draws_its_stages_widest_first() {
    let drawn = draw_funnel(&funnel(PieLabels::Value, &[100.0, 40.0, 60.0])).expect("drawn");
    let order: Vec<_> = drawn.geometry.marks.iter().map(|m| m.index).collect();
    assert_eq!(order, [0, 1, 2], "{}", drawn.svg);
    let mut tops = Vec::new();
    for mark in &drawn.geometry.marks {
        let (x, y) = super::super::hit::anchor(&mark.shape);
        let outline = super::super::hit::outline(match &mark.shape {
            MarkShape::Sector { path } => path,
            _ => panic!("{mark:?}"),
        });
        assert!(super::super::hit::contains(&outline, x, y), "{mark:?}");
        tops.push(y);
    }
    assert!(tops[0] < tops[2] && tops[2] < tops[1], "{tops:?}");
    assert!(!drawn.svg.contains("ct-trigger") && !drawn.svg.contains("<title"));
}

/// A percent is a share of the first stage; a label too wide for its stage is dropped.
#[test]
fn a_funnel_writes_percent_of_the_first_stage() {
    let drawn = draw_funnel(&funnel(PieLabels::Percent, &[200.0, 50.0])).expect("drawn");
    let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
    assert!(labels.contains(&"100%".to_string()), "{labels:?}");
    assert!(labels.contains(&"25%".to_string()), "{labels:?}");

    let mut narrow = funnel(PieLabels::Name, &[1000.0, 1.0]);
    narrow.stages[1].name = "A very long stage name".to_string();
    let drawn = draw_funnel(&narrow).expect("drawn");
    let labels: Vec<_> = texts(&drawn.svg).into_iter().map(|t| t.3).collect();
    assert_eq!(labels, ["S0"], "{}", drawn.svg);
    assert!(!drawn.svg.contains("A very long"), "{}", drawn.svg);

    let drawn = draw_funnel(&funnel(PieLabels::None, &[2.0, 1.0])).expect("drawn");
    assert!(texts(&drawn.svg).is_empty(), "{}", drawn.svg);
}

fn radar(values: &[&[Option<f64>]]) -> RadarSpec {
    RadarSpec {
        width: 400.0,
        height: 300.0,
        axes: ["Speed", "Power", "Range", "Cost"]
            .iter()
            .map(|name| (name.to_string(), 40.0))
            .collect(),
        series: values
            .iter()
            .enumerate()
            .map(|(i, values)| RadarSpecSeries {
                name: format!("S{i}"),
                values: values.to_vec(),
                color: HexColor::new(0x336699 + i as u32),
                symbol: [PointSymbol::Circle, PointSymbol::Triangle][i % 2],
            })
            .collect(),
        filled: true,
        rings: 4,
        grid: true,
        radius: 4.0,
        ink: ChartInk::of(&Theme::DEFAULT, false),
    }
}

/// Each value is a point on its spoke, clockwise from the top, by its share of the rim;
/// a missing one has no point.
#[test]
fn radar_points_sit_on_their_spokes() {
    let drawn = draw_radar(&radar(&[
        &[Some(40.0), Some(20.0), Some(10.0), Some(0.0)],
        &[Some(20.0), None, Some(80.0), Some(40.0)],
    ]))
    .expect("drawn");
    let points: Vec<_> = drawn
        .geometry
        .marks
        .iter()
        .map(|mark| match mark.shape {
            MarkShape::Point { x, y } => (mark.series, mark.index, x.round(), y.round()),
            _ => panic!("{mark:?}"),
        })
        .collect();
    // The web's radius is 300 / 2 - 40 = 110, round (200, 150).
    assert_eq!(
        points,
        [
            (0, 0, 200.0, 40.0),
            (0, 1, 255.0, 150.0),
            (0, 2, 200.0, 178.0),
            (0, 3, 200.0, 150.0),
            (1, 0, 200.0, 95.0),
            (1, 2, 200.0, 260.0),
            (1, 3, 90.0, 150.0),
        ],
        "{}",
        drawn.svg
    );
    assert_eq!(drawn.svg.matches("<circle").count(), 4, "{}", drawn.svg);
    assert_eq!(drawn.svg.matches("<polygon").count(), 3, "{}", drawn.svg);
    assert!(!drawn.svg.contains("Roboto"), "{}", drawn.svg);
}

/// The names sit past their spokes' ends, read away from the web, cut to the SVG.
#[test]
fn radar_names_read_away_from_the_web() {
    let mut spec = radar(&[&[Some(1.0); 4]]);
    spec.axes[1].0 = "A very long axis name that cannot fit".to_string();
    let drawn = draw_radar(&spec).expect("drawn");
    let names = texts(&drawn.svg);
    assert_eq!(names.len(), 4, "{}", drawn.svg);
    assert!(
        drawn.svg.contains("text-anchor=\"middle\""),
        "{}",
        drawn.svg
    );
    assert!(drawn.svg.contains("text-anchor=\"start\""), "{}", drawn.svg);
    assert!(drawn.svg.contains("text-anchor=\"end\""), "{}", drawn.svg);
    assert!(names[1].3.ends_with('…'), "{names:?}");
    assert!(
        draw_radar(&RadarSpec {
            axes: spec.axes[..2].to_vec(),
            ..spec
        })
        .is_none()
    );
}
