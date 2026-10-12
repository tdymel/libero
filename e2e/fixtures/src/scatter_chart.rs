//! `ScatterChart`: two series with a trend line, and bubbles, with read-outs of the active
//! point, the clicks and the hidden series.

use dioxus::prelude::*;
use libero::components::{ChartPoint, ScatterChart, ScatterSeries, Trend};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/scatter-chart", || rsx! { ScatterChartPage {} }),
    ("/scatter-chart/bubbles", || rsx! { BubblesPage {} }),
];

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.series_name, point.x, point.value)
}

#[component]
fn ScatterChartPage() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    let mut hidden = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            ScatterChart {
                id: "chart",
                title: "Spend and sales",
                trend: Trend::Linear,
                series: vec![
                    ScatterSeries::new("A", [(10, 30), (20, 45), (30, 40)]),
                    ScatterSeries::new("B", [(13, 20), (28, 25)]),
                ],
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
fn BubblesPage() -> Element {
    rsx! {
        div { max_width: "480px",
            ScatterChart {
                id: "chart",
                title: "Cities",
                series: vec![ScatterSeries::new("Cities", [(1, 20, 300), (2, 40, 1200), (3, 30, 600)])],
            }
        }
    }
}
