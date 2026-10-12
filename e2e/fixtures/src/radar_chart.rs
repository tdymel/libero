//! `RadarChart`: two series round four axes, one value missing, with read-outs of the active
//! point, the clicks and the hidden series.

use dioxus::prelude::*;
use libero::components::{ChartPoint, RadarAxis, RadarChart, Series};

use crate::Routes;

pub const ROUTES: Routes = &[("/radar-chart", || rsx! { RadarChartPage {} })];

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.series_name, point.x, point.value)
}

#[component]
fn RadarChartPage() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    let mut hidden = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            RadarChart {
                id: "chart",
                title: "Cars",
                axes: vec![RadarAxis::from(("Speed", 10.0)), ("Power", 10.0).into(), "Range".into(), ("Cost", 10.0).into()],
                series: vec![
                    Series::new("X", [8.0, 6.0, 7.0, 5.0]),
                    Series::new("Y", [Some(6.0), None, Some(18.0), Some(4.0)]),
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
