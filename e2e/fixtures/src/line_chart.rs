//! `LineChart`: two lines, with read-outs of the active point, the clicks and the hidden series.

use dioxus::prelude::*;
use libero::components::{ChartPoint, LineChart, Series};

use crate::Routes;

pub const ROUTES: Routes = &[("/line-chart", || rsx! { LineChartPage {} })];

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.series_name, point.x, point.value)
}

#[component]
fn LineChartPage() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    let mut hidden = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            LineChart {
                id: "chart",
                title: "Visitors",
                x: ["Mon", "Tue", "Wed", "Thu", "Fri"],
                series: vec![
                    Series::new("This week", [120, 132, 101, 134, 90]),
                    Series::new("Last week", [Some(20), Some(30), None, Some(25), Some(40)]).dashed(),
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
        // Room below, so a swipe over the chart has a page to scroll.
        div { height: "150vh" }
    }
}
