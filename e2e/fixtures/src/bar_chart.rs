//! `BarChart`: grouped upright bars, and stacked lying ones with their values, with
//! read-outs of the active bar, the clicks and the hidden series.

use dioxus::prelude::*;
use libero::components::{BarChart, ChartPoint, ChartRange, Series};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/bar-chart", || rsx! { Grouped {} }),
    ("/bar-chart/stacked", || rsx! { Stacked {} }),
    ("/bar-chart/styled", || rsx! { Styled {} }),
    ("/bar-chart/zoom", || rsx! { Zoomable {} }),
];

fn series() -> Vec<Series> {
    vec![
        Series::new("2025", [120, 132, 101]),
        Series::new("2026", [Some(140), None, Some(150)]),
    ]
}

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.series_name, point.x, point.value)
}

#[component]
fn Grouped() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    let mut hidden = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            BarChart {
                id: "chart",
                title: "Sales",
                x: ["Q1", "Q2", "Q3"],
                series: series(),
                onactivechange: move |point: Option<ChartPoint>| {
                    active.set(point.as_ref().map(describe).unwrap_or_default())
                },
                onpointclick: move |point: ChartPoint| clicked.set(describe(&point)),
                onhiddenchange: move |names: Vec<String>| hidden.set(names.join(",")),
            }
        }
        p { id: "active", "{active}" }
        p { id: "clicked", "{clicked}" }
        p { id: "hidden", "{hidden}" }
    }
}

#[component]
fn Stacked() -> Element {
    let mut active = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            BarChart {
                id: "chart",
                title: "Sales",
                x: ["Q1", "Q2", "Q3"],
                series: series(),
                layout: "stacked",
                orientation: "horizontal",
                value_labels: true,
                onactivechange: move |point: Option<ChartPoint>| {
                    active.set(point.as_ref().map(describe).unwrap_or_default())
                },
            }
        }
        p { id: "active", "{active}" }
    }
}

/// One series: half-band bars, a loss in its own colour, error ranges, values inside.
#[component]
fn Styled() -> Element {
    rsx! {
        div { max_width: "480px",
            BarChart {
                id: "chart",
                title: "Profit",
                x: ["Q1", "Q2", "Q3"],
                series: vec![
                    Series::new("2026", [120, -40, 150])
                        .negative_color("#c92a2a")
                        .errors([(110, 130), (-50, -30), (135, 165)]),
                ],
                bar_width: 0.5,
                value_labels: true,
                value_label_position: "inside",
            }
        }
    }
}

/// Lying bars that zoom down their side, with the brushed range read out.
#[component]
fn Zoomable() -> Element {
    let mut range = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            BarChart {
                id: "chart",
                title: "Sales",
                x: ["Jan", "Feb", "Mar", "Apr", "May", "Jun"],
                series: vec![Series::new("2026", [3, 5, 4, 6, 8, 7])],
                orientation: "horizontal",
                zoom: true,
                onrange: move |selected: Option<ChartRange>| {
                    range.set(selected.map(|r| format!("{} {}", r.from, r.to)).unwrap_or_default())
                },
            }
        }
        p { id: "range", "{range}" }
    }
}
