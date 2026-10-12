//! The docs page's type switch: one `SegmentedControl` over a line, a bar and a pie chart.

use dioxus::prelude::*;
use libero::components::{
    BarChart, LineChart, OptionLabel, PieChart, SegmentedControl, Series, Slice,
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
                options: ["line", "bar", "pie"].map(String::from).to_vec(),
                option_label: |option: String| OptionLabel::from(match option.as_str() {
                    "line" => "Line",
                    "bar" => "Bar",
                    _ => "Pie",
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
