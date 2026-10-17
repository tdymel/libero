//! A funnel chart's spec and drawing.

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FunnelSpec {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) stages: Vec<SeriesSpec>,
    pub(crate) labels: PieLabels,
    /// A percent label is a share of this, the first stage's value.
    pub(crate) first: f64,
    pub(crate) ink: ChartInk,
    pub(crate) format: ValueFormat,
    pub(crate) unit: Option<String>,
    pub(crate) formats: &'static Formats,
}

pub(crate) fn draw_funnel(spec: &FunnelSpec) -> Option<Drawn> {
    let value = |stage: &SeriesSpec| {
        stage
            .values
            .first()
            .copied()
            .flatten()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .max(0.0)
    };
    let series = spec
        .stages
        .iter()
        .map(|stage| charts_rs::Series::new(stage.name.clone(), vec![value(stage) as f32]))
        .collect();
    let mut chart = charts_rs::FunnelChart::new(series);
    chart.width = spec.width;
    chart.height = spec.height;
    chart.background_color = InkColor::transparent();
    chart.title.text = String::new();
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.animation = None;
    chart.margin = Default::default();
    chart.series.colors = spec.stages.iter().map(|s| ink(s.color)).collect();
    chart.series_label_position = Some("inside".to_string());
    chart.series.label.font.color = ON_BAR;
    chart.series.label.formatter = match spec.labels {
        PieLabels::Value => format!("{AXIS_MARK}{{c}}{}", unit_suffix(spec.unit.as_deref())),
        PieLabels::Percent => format!("{PERCENT_MARK}{{c}}"),
        PieLabels::Name => "{a}".to_string(),
        // Blank, as an empty one is the renderer's "{a}: {c}"; `ink_slice_labels` drops it.
        PieLabels::None => " ".to_string(),
    };
    let svg = centre_stage_labels(&chart.svg().ok()?);
    let (svg, triggers) = inert(
        &svg,
        |label| {
            spec.format
                .format(label, spec.unit.as_deref(), spec.formats)
        },
        "",
        false,
    );
    let svg = relabel(&svg, PERCENT_MARK, |value| match spec.first > 0.0 {
        true => ValueFormat::Percent.format(value / spec.first, None, spec.formats),
        false => String::new(),
    });
    // The renderer draws the widest stage first; a sort keeps ties in data order.
    let mut order: Vec<usize> = (0..spec.stages.len()).collect();
    order.sort_by(|a, b| {
        let (a, b) = (
            value(&spec.stages[*a]) as f32,
            value(&spec.stages[*b]) as f32,
        );
        b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
    });
    let expected: Vec<_> = order
        .iter()
        .map(|&i| (0, i, value(&spec.stages[i]), spec.stages[i].name.as_str()))
        .collect();
    let mut marks = match_marks(&expected, &triggers);
    let svg = ink_slice_labels(
        &svg,
        &marks,
        |stage| spec.ink.on(spec.stages[stage].color),
        spec.ink.label,
    );
    // The keys walk the author's order, as the summary and the legend read.
    marks.sort_by_key(|mark| mark.index);
    Some(Drawn {
        svg: spec.ink.paint(svg),
        geometry: Geometry {
            width: spec.width,
            height: spec.height,
            plot: Plot::whole(spec.width, spec.height),
            marks,
            hole: 0.0,
            share_of: Some(spec.first),
            cells: false,
        },
    })
}
