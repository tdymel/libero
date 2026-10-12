//! `PieChart`: a pie with its shares and read-outs of the active slice, the clicks and
//! the hidden slices, and a donut with a centre.

use dioxus::prelude::*;
use libero::components::{ChartPoint, PieChart, Slice};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/pie-chart", || rsx! { Pie {} }),
    ("/pie-chart/donut", || rsx! { Donut {} }),
    ("/pie-chart/gauge", || rsx! { Gauge {} }),
];

fn slices() -> Vec<Slice> {
    vec![
        Slice::new("Chrome", 60.0),
        Slice::new("Safari", 25.0),
        Slice::new("Other", 15.0),
    ]
}

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.series_name, point.index, point.value)
}

#[component]
fn Pie() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    let mut hidden = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            PieChart {
                id: "chart",
                title: "Browsers",
                slices: slices(),
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

/// A half donut over 9 to 3 o'clock, its labels on the slices, a centre under the arc.
#[component]
fn Gauge() -> Element {
    rsx! {
        div { max_width: "480px",
            PieChart {
                id: "chart",
                title: "Browsers",
                slices: slices(),
                donut: true,
                start_angle: -90.0,
                end_angle: 90.0,
                label_position: "inside",
                center: rsx! { span { id: "center", "100" } },
            }
        }
    }
}
