//! `BarChart`: grouped upright bars, and stacked lying ones with their values, with
//! read-outs of the active bar, the clicks and the hidden series.

use dioxus::prelude::*;
use libero::components::{BarChart, ChartPoint, Series};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/bar-chart", || rsx! { Grouped {} }),
    ("/bar-chart/stacked", || rsx! { Stacked {} }),
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
