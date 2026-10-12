//! `FunnelChart`: a funnel with read-outs of the active stage, the clicks and the hidden
//! stages.

use dioxus::prelude::*;
use libero::components::{ChartPoint, FunnelChart, Slice};

use crate::Routes;

pub const ROUTES: Routes = &[("/funnel-chart", || rsx! { Funnel {} })];

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.series_name, point.index, point.value)
}

#[component]
fn Funnel() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    let mut hidden = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            FunnelChart {
                id: "chart",
                title: "Checkout",
                stages: vec![
                    Slice::new("Visits", 1000.0),
                    Slice::new("Carts", 400.0),
                    Slice::new("Orders", 100.0),
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
