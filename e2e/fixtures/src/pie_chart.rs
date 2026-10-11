//! `PieChart`: a pie with its shares, and a donut with a centre.

use dioxus::prelude::*;
use libero::components::{PieChart, Slice};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/pie-chart", || rsx! { Pie {} }),
    ("/pie-chart/donut", || rsx! { Donut {} }),
];

fn slices() -> Vec<Slice> {
    vec![
        Slice::new("Chrome", 60.0),
        Slice::new("Safari", 25.0),
        Slice::new("Other", 15.0),
    ]
}

#[component]
fn Pie() -> Element {
    rsx! {
        div { max_width: "480px",
            PieChart { id: "chart", title: "Browsers", slices: slices() }
        }
    }
}

#[component]
fn Donut() -> Element {
    rsx! {
        div { max_width: "480px",
            PieChart {
                id: "chart",
                title: "Browsers",
                slices: slices(),
                donut: true,
                labels: "none",
                center: rsx! { span { id: "center", "100 users" } },
            }
        }
    }
}
