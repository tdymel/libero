//! Responsive counts and layouts (todos 5688-5692): each switches at `md` (62rem).

use dioxus::prelude::*;
use libero::components::{
    BarChart, Carousel, Cascader, CascaderLayout, CascaderOption, Pagination, Series,
};
use libero::theme::{BarLayout, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/responsive-counts", || rsx! { CountsPage {} }),
];

#[component]
fn CountsPage() -> Element {
    let mut page = use_signal(|| 10);
    let mut aisle = use_signal(|| None::<String>);
    let slides: Vec<Element> = (1..=6)
        .map(|n| rsx! { div { height: "80px", background: "#ddd", "Slide {n}" } })
        .collect();
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 24px; padding: 16px",
            Pagination {
                id: "rsc-pages",
                total: 20,
                page: page(),
                onchange: move |next| page.set(next),
                aria_label: "Pages",
                siblings: responsive(0).md(2),
            }
            Carousel {
                id: "rsc-carousel",
                aria_label: "Offers",
                align: "start",
                per_view: responsive(1.0).md(3.0),
                slides_to_scroll: responsive(1).md(3),
                slides,
            }
            Cascader {
                label: "Aisle",
                value: aisle(),
                onchange: move |next| aisle.set(next),
                layout: responsive(CascaderLayout::Paths).md(CascaderLayout::Columns),
                data: vec![
                    CascaderOption::<String>::new("food", "Food")
                        .children(vec![CascaderOption::new("leek", "Leek")]),
                ],
            }
            div { id: "rsc-bars", width: "480px",
                BarChart {
                    title: "Sales",
                    layout: responsive(BarLayout::Stacked).md(BarLayout::Grouped),
                    x: ["a", "b"],
                    series: vec![Series::new("A", [1.0, 3.0]), Series::new("B", [3.0, 1.0])],
                }
            }
        }
    }
}
