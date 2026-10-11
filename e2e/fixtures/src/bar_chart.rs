//! `BarChart`: grouped upright bars, and stacked lying ones with their values.

use dioxus::prelude::*;
use libero::components::{BarChart, Series};

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

#[component]
fn Grouped() -> Element {
    rsx! {
        div { max_width: "480px",
            BarChart { id: "chart", title: "Sales", x: ["Q1", "Q2", "Q3"], series: series() }
        }
    }
}

#[component]
fn Stacked() -> Element {
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
            }
        }
    }
}
