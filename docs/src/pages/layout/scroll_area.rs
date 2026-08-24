use crate::components::{Control, Demo, DemoValues, DocPage, UNSET, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, Flex, Input, ScrollArea, ScrollPositionEvent, Text},
    sx::sx,
    use_theme,
};

/// Wider and taller than the frame, so both axes have something to scroll -
/// relative, so it stays wider however wide the frame gets.
const CONTENT: &str = r#"Box {
    sx: sx().width("150%").padding("md"),
    for i in 0..20 {
        Text { key: "{i}", "Item {i}" }
    }
}"#;

/// The scroll position is read *and* written from the page, so all four wiring
/// props are the demo's fixture rather than anything a control varies.
const FIXED: [&str; 4] = [
    "scroll_position_y: jump()",
    "on_scroll: move |event: ScrollPositionEvent| position.set(event)",
    r#"on_top_reached: move |_| { edge.set("top"); jump.set(None) }"#,
    r#"on_bottom_reached: move |_| { edge.set("bottom"); jump.set(None) }"#,
];

/// A raw `{event:?}` prints an unrounded `f64` and reflows the row on every
/// scroll tick, so the readout formats the two percentages itself.
fn readout(event: ScrollPositionEvent) -> String {
    let (kind, x, y) = match event {
        ScrollPositionEvent::Start(x, y) => ("Start", x, y),
        ScrollPositionEvent::Change(x, y) => ("Change", x, y),
        ScrollPositionEvent::End(x, y) => ("End", x, y),
    };
    format!("{kind} at x {x:.0}%, y {y:.0}%")
}

/// A scroll area fills its parent, so the preview has to give it one - and the
/// readout and the two jump buttons are the other half of the wiring above,
/// so they belong in the preview too. The code block prints all of it.
fn wrap_frame(_: &DemoValues, code: &str) -> String {
    format!(
        "Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\"),\n    Box {{\n        sx: sx().height(\"160px\").width(\"100%\").border(\"1px solid var(--lsx-grey-3)\"),\n{}    }}\n    Text {{ size: \"sm\", \"{{readout(position())}} - last edge: {{edge()}}\" }}\n    Flex {{\n        gap: \"sm\",\n        Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| jump.set(Some(0.0)), \"Scroll to top\" }}\n        Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| jump.set(Some(100.0)), \"Scroll to bottom\" }}\n    }}\n}}",
        indent(&indent(code))
    )
}

#[component]
pub fn ScrollAreaPage() -> Element {
    let theme = use_theme();
    let mut position = use_signal(|| ScrollPositionEvent::Change(0.0, 0.0));
    let mut edge = use_signal(|| "none");
    let mut jump = use_signal(|| None::<f64>);

    rsx! {
        DocPage {
            title: "ScrollArea",
            source: "libero/src/components/layout/scroll_area.rs",
            markdown: "/md/scroll_area.md",
            properties: vec![props("ScrollArea", vec![
                prop("scrollbars", "ScrollAxis")
                    .default("vertical")
                    .doc("Which axes show a scrollbar and allow overflow: vertical, horizontal, both or none."),
                prop("scrollbar_visibility", "ScrollbarVisibility")
                    .default("always")
                    .doc("always, hover, hidden, or scroll (currently identical to hover)."),
                prop("scrollbar_size", "ScrollbarSize")
                    .default("thin")
                    .doc("CSS `scrollbar-width`: thin or auto."),
                prop("scrollbar_color", "ThemeAwareValue")
                    .doc("Scrollbar thumb color - the track stays transparent."),
                prop("scroll_position_x", "f64")
                    .doc("Percent (0-100) to scroll to horizontally. Bound to a signal it re-applies on every change; a literal applies once, at mount."),
                prop("scroll_position_y", "f64")
                    .doc("Percent (0-100) along the vertical axis - see `scroll_position_x`."),
                prop("on_scroll", "EventHandler<ScrollPositionEvent>")
                    .doc("Fires on every scroll tick with the position as a percent of each axis's scrollable range."),
                prop("on_top_reached", "EventHandler<()>").doc("Fires once when the top edge is reached."),
                prop("on_bottom_reached", "EventHandler<()>").doc("Fires once when the bottom edge is reached."),
                prop("on_left_reached", "EventHandler<()>").doc("Fires once when the left edge is reached."),
                prop("on_right_reached", "EventHandler<()>").doc("Fires once when the right edge is reached."),
                prop("children", "Element").doc("The scrollable content."),
            ])],
            lead: rsx! {
                Text {
                    "Scrolls its content, filling the parent by default. "
                    Code { source: "on_scroll" }
                    " reports the position as a percent of each axis - "
                    Code { source: "Start" }
                    "/"
                    Code { source: "End" }
                    " bracket one scroll, "
                    Code { source: "Change" }
                    " carries the rest - and "
                    Code { source: "scroll_position_x" }
                    "/"
                    Code { source: "scroll_position_y" }
                    " scroll it to a percent. Each edge has its own event; the demo wires "
                    Code { source: "on_top_reached" }
                    " and "
                    Code { source: "on_bottom_reached" }
                    ", and "
                    Code { source: "on_left_reached" }
                    "/"
                    Code { source: "on_right_reached" }
                    " work the same way."
                }
            },
            Demo {
                component: "ScrollArea",
                children_text: "",
                children_code: CONTENT.to_string(),
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    Control::toggle("scrollbars", ["vertical", "horizontal", "both", "none"])
                        .default(theme.scroll_area.scrollbars.as_str()),
                    Control::toggle("scrollbar_visibility", ["always", "hover", "hidden"])
                        .default(theme.scroll_area.visibility.as_str()),
                    Control::toggle("scrollbar_size", ["thin", "auto"])
                        .default(theme.scroll_area.size.as_str()),
                    // The unset thumb is grey-5, which a bare `grey` would
                    // *not* resolve to - so the first swatch is unset.
                    Control::color(
                        "scrollbar_color",
                        [UNSET, "primary", "secondary", "success", "error", "warning"],
                    ),
                ],
                render: move |values: DemoValues| rsx! {
                    Flex {
                        direction: "column",
                        gap: "sm",
                        // Pinned, so a longer `Change at ...` readout
                        // cannot widen the column mid-scroll.
                        sx: sx().width("100%"),
                        Box {
                            sx: sx()
                                .height("160px")
                                .width("100%")
                                .border("1px solid var(--lsx-grey-3)"),
                            ScrollArea {
                                scrollbars: values.str("scrollbars"),
                                scrollbar_visibility: values.str("scrollbar_visibility"),
                                scrollbar_size: values.str("scrollbar_size"),
                                scrollbar_color: match values.str("scrollbar_color").as_str() {
                                    UNSET => Input::None,
                                    color => Input::from(color),
                                },
                                scroll_position_y: jump(),
                                on_scroll: move |event: ScrollPositionEvent| {
                                    position.set(event)
                                },
                                on_top_reached: move |_| {
                                    edge.set("top");
                                    jump.set(None)
                                },
                                on_bottom_reached: move |_| {
                                    edge.set("bottom");
                                    jump.set(None)
                                },
                                Box {
                                    sx: sx().width("150%").padding("md"),
                                    for i in 0..20 {
                                        Text { key: "{i}", "Item {i}" }
                                    }
                                }
                            }
                        }
                        Text { size: "sm", "{readout(position())} - last edge: {edge()}" }
                        Flex {
                            gap: "sm",
                            Button {
                                size: "sm",
                                variant: "outlined",
                                onclick: move |_| jump.set(Some(0.0)),
                                "Scroll to top"
                            }
                            Button {
                                size: "sm",
                                variant: "outlined",
                                onclick: move |_| jump.set(Some(100.0)),
                                "Scroll to bottom"
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_frame),
            }
        }
    }
}
