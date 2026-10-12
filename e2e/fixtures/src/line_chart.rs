//! `LineChart`: two lines, with read-outs of the active point, the clicks and the hidden series.

use dioxus::prelude::*;
use libero::components::{Button, ChartPoint, ChartRange, LineChart, Series, use_chart};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/line-chart", || rsx! { LineChartPage {} }),
    ("/line-chart-zoom", || rsx! { ZoomPage {} }),
];

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

/// Zoom, the brush and the export, with the brushed range and the handle's files read out.
#[component]
fn ZoomPage() -> Element {
    let chart = use_chart();
    let mut active = use_signal(String::new);
    let mut range = use_signal(String::new);
    let mut svg = use_signal(String::new);
    let mut png = use_signal(String::new);
    let mut saved = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            LineChart {
                id: "chart",
                title: "Visitors",
                x: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"],
                series: vec![Series::new("Visits", [3, 5, 4, 6, 8, 7, 9, 12])],
                zoom: true,
                handle: chart,
                onactivechange: move |point: Option<ChartPoint>| {
                    active.set(point.as_ref().map(describe).unwrap_or_default())
                },
                onrange: move |selected: Option<ChartRange>| {
                    range.set(
                        selected
                            .map(|r| format!("{} {} {} {}", r.start, r.end, r.from, r.to))
                            .unwrap_or_default(),
                    )
                },
            }
        }
        Button {
            id: "reset",
            disabled: !chart.is_zoomed(),
            focusable_when_disabled: true,
            onclick: move |_| chart.reset_zoom(),
            "Reset zoom"
        }
        Button {
            id: "save-svg",
            onclick: move |_| async move {
                let outcome = chart.save_svg("visitors.svg").await;
                saved.set(format!("{outcome:?}"));
            },
            "Save as SVG"
        }
        Button {
            id: "read-svg",
            onclick: move |_| {
                let read = chart.svg().unwrap_or_default();
                let titled = read.contains("<title>Visitors</title>");
                svg.set(format!("{} {titled}", read.starts_with("<svg")));
            },
            "Read SVG"
        }
        Button {
            id: "read-png",
            onclick: move |_| async move {
                let bytes = chart.png().await.unwrap_or_default();
                png.set(format!("{}", bytes.starts_with(b"\x89PNG")));
            },
            "Read PNG"
        }
        p { id: "active", "{active}" }
        p { id: "range", "{range}" }
        p { id: "svg", "{svg}" }
        p { id: "png", "{png}" }
        p { id: "saved", "{saved}" }
        div { height: "150vh" }
    }
}
