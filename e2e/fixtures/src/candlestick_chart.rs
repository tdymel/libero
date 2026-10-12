//! `CandlestickChart`: a rising, a falling and an unchanged candle, with read-outs of the
//! active and the clicked candle.

use dioxus::prelude::*;
use libero::components::{Candle, CandlestickChart, ChartPoint};

use crate::Routes;

pub const ROUTES: Routes = &[("/candlestick-chart", || rsx! { Candles {} })];

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
