//! The docs page's type switch: one `SegmentedControl` over a line, a bar, a pie, a
//! funnel chart, a heatmap and a candlestick chart.

use dioxus::prelude::*;
use libero::components::{
    BarChart, Candle, CandlestickChart, FunnelChart, HeatmapChart, LineChart, OptionLabel, PieChart, SegmentedControl, Series, Slice,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/chart", || rsx! { Switch {} })];

#[component]
fn Switch() -> Element {
    let mut kind = use_signal(|| "line".to_string());
    rsx! {
        div { max_width: "480px",
            SegmentedControl {
                id: "type",
                variant: "outlined",
                size: "sm",
                full_width: true,
                "aria-label": "Type",
                value: kind(),
                options: ["line", "bar", "pie", "funnel", "heatmap", "candlestick"].map(String::from).to_vec(),
                option_label: |option: String| OptionLabel::from(match option.as_str() {
                    "line" => "Line",
                    "bar" => "Bar",
                    "pie" => "Pie",
                    "funnel" => "Funnel",
                    "heatmap" => "Heatmap",
                    _ => "Candlestick",
                }),
                onchange: move |next: String| kind.set(next),
            }
            match kind().as_str() {
                "bar" => rsx! {
                    BarChart {
                        id: "chart",
                        title: "Profit",
                        x: ["Q1", "Q2", "Q3"],
                        series: vec![
                            Series::new("2025", [40, 55, -25]),
                            Series::new("2026", [30, -15, 20]),
                        ],
                    }
                },
                "pie" => rsx! {
                    PieChart {
                        id: "chart",
                        title: "Browsers",
                        slices: vec![Slice::new("Chrome", 62.0), Slice::new("Safari", 20.0), Slice::new("Other", 18.0)],
                    }
                },
                "funnel" => rsx! {
                    FunnelChart {
                        id: "chart",
                        title: "Checkout",
                        stages: vec![Slice::new("Visits", 1200.0), Slice::new("Carts", 480.0), Slice::new("Orders", 210.0)],
                    }
                },
                "heatmap" => rsx! {
                    HeatmapChart {
                        id: "chart",
                        title: "Visits",
                        x: ["9 am", "12 pm"].map(String::from).to_vec(),
                        y: ["Mon", "Tue"].map(String::from).to_vec(),
                        values: vec![vec![12.0, 30.0], vec![8.0, 26.0]],
                    }
                },
                "candlestick" => rsx! {
                    CandlestickChart {
                        id: "chart",
                        title: "Share price",
                        x: ["Mon", "Tue"].map(String::from).to_vec(),
                        candles: vec![Candle::new(20.0, 38.0, 10.0, 34.0), Candle::new(34.0, 40.0, 28.0, 30.0)],
                    }
                },
                _ => rsx! {
                    LineChart {
                        id: "chart",
                        title: "Visitors",
                        x: ["Mon", "Tue", "Wed"],
                        series: vec![
                            Series::new("This week", [120, 132, 101]),
                            Series::new("Last week", [98, 110, 115]),
                        ],
                    }
                },
            }
        }
    }
}
