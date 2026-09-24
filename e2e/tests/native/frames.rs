//! Native frame cost (1086): a wheel step's update and CPU raster, report only and opt-in:
//! `cargo test --release -p e2e --features native --test native frames:: -- --ignored --nocapture`.

use dioxus::prelude::*;
use e2e::frames;
use e2e::native::mount;
use libero::components::{ScrollArea, Virtualize};

const STEPS: usize = 60;

/// The control: the same wheel over a box that does not scroll.
fn still() -> Element {
    rsx! {
        div { id: "list-pane", height: "120px", "Still" }
    }
}

fn list() -> Element {
    rsx! {
        div { id: "list-pane", height: "120px",
            ScrollArea {
                Virtualize {
                    count: 1000,
                    item_size: Some(20.0),
                    item: move |i: usize| rsx! {
                        div { "data-row": i, height: "20px", "Row {i}" }
                    },
                }
            }
        }
    }
}

#[test]
#[ignore = "frame-time report, run on request"]
fn raster_time_of_a_scroll_area_under_the_wheel() {
    for (name, app, ready) in [
        ("native_still", still as fn() -> Element, "#list-pane"),
        ("native_scroll_area", list, "[data-row='2']"),
    ] {
        let mut page = mount(app);
        // The pane's measure is a timer: the rows come a few polls late.
        page.wait_for(|page| page.exists(ready));
        page.hover("#list-pane");
        let times = page.time_raster_steps("#list-pane", STEPS, 40.0);
        assert!(times.raster.count > 0, "no steps timed: {times:?}");
        assert!(
            !page.exists("[data-row='0']"),
            "the wheel did not scroll:\n{}",
            page.tree()
        );
        frames::report(&format!("{name}_update"), times.update).unwrap();
        frames::report(&format!("{name}_raster"), times.raster).unwrap();
    }
}
