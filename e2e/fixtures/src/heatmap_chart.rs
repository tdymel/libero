//! `HeatmapChart`: a grid with an empty cell, with read-outs of the active and the
//! clicked cell.

use dioxus::prelude::*;
use libero::components::{ChartPoint, HeatmapChart};

use crate::Routes;

pub const ROUTES: Routes = &[("/heatmap-chart", || rsx! { Heatmap {} })];

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.series_name, point.index, point.value)
}

#[component]
fn Heatmap() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            HeatmapChart {
                id: "chart",
                title: "Visits",
                x: ["9 am", "12 pm", "3 pm"].map(String::from).to_vec(),
                y: ["Mon", "Tue"].map(String::from).to_vec(),
                values: vec![vec![10.0, 40.0, 25.0], vec![5.0, f64::NAN, 50.0]],
                onactivechange: move |point: Option<ChartPoint>| {
                    active.set(point.as_ref().map(describe).unwrap_or_default())
                },
                onpointclick: move |point: ChartPoint| clicked.set(describe(&point)),
            }
        }
        p { id: "active", "{active}" }
        p { id: "clicked", "{clicked}" }
    }
}
