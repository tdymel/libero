//! `CandlestickChart`: a rising, a falling and an unchanged candle, with read-outs of the
//! active and the clicked candle.

use dioxus::prelude::*;
use libero::components::{Button, Candle, CandlestickChart, ChartPoint, ChartRange, use_chart};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/candlestick-chart", || rsx! { Candles {} }),
    ("/candlestick-chart/zoom", || rsx! { Zoomable {} }),
];

fn describe(point: &ChartPoint) -> String {
    format!("{} {} {}", point.x, point.index, point.value)
}

#[component]
fn Candles() -> Element {
    let mut active = use_signal(String::new);
    let mut clicked = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            CandlestickChart {
                id: "chart",
                title: "Share price",
                x: ["Mon", "Tue", "Wed"].map(String::from).to_vec(),
                candles: vec![
                    Candle::new(20.0, 38.0, 10.0, 34.0),
                    Candle::new(34.0, 40.0, 28.0, 30.0),
                    Candle::new(30.0, 36.0, 28.0, 30.0),
                ],
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

/// Candles that zoom along their periods, with the active candle and the brushed range read out.
#[component]
fn Zoomable() -> Element {
    let chart = use_chart();
    let mut active = use_signal(String::new);
    let mut range = use_signal(String::new);
    rsx! {
        div { max_width: "480px",
            CandlestickChart {
                id: "chart",
                title: "Share price",
                x: ["Jan", "Feb", "Mar", "Apr", "May", "Jun"].map(String::from).to_vec(),
                candles: vec![
                    Candle::new(20.0, 38.0, 10.0, 34.0),
                    Candle::new(34.0, 40.0, 28.0, 30.0),
                    Candle::new(30.0, 46.0, 28.0, 41.0),
                    Candle::new(41.0, 44.0, 32.0, 36.0),
                    Candle::new(36.0, 48.0, 34.0, 44.0),
                    Candle::new(44.0, 46.0, 38.0, 40.0),
                ],
                zoom: true,
                handle: chart,
                onactivechange: move |point: Option<ChartPoint>| {
                    active.set(point.as_ref().map(describe).unwrap_or_default())
                },
                onrange: move |selected: Option<ChartRange>| {
                    range.set(selected.map(|r| format!("{} {}", r.from, r.to)).unwrap_or_default())
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
        p { id: "active", "{active}" }
        p { id: "range", "{range}" }
    }
}
