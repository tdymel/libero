use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, UNSET, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Button, Code, CodeBlock, Flex, Input, List, ListItem, ScrollArea, ScrollPositionEvent,
        Text, Virtualize,
    },
    sx::sx,
    use_theme,
};

const FOCUSABLE_EXAMPLE: &str = r#"ScrollArea {
    focusable: true,
    role: "region",
    aria_label: "Release notes",
    Text { "..." }
}"#;

/// Fifty thousand rows, of which the section below renders about a dozen.
const VIRTUAL_ROWS: usize = 50_000;

const VIRTUAL_EXAMPLE: &str = r#"let rows = use_signal(|| (0..50_000).map(|i| format!("Row {i}")).collect::<Vec<_>>());

rsx! {
    ScrollArea {
        List {
            Virtualize {
                count: rows.read().len(),
                item: move |index| rsx! {
                    ListItem { "{rows.read()[index]}" }
                },
            }
        }
    }
}"#;

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
                prop("focusable", "bool")
                    .default("false")
                    .doc("Makes the viewport itself a tab stop, so content with no focusable elements of its own can still be reached and arrow-keyed."),
                prop("on_scroll", "EventHandler<ScrollPositionEvent>")
                    .doc("Fires on every scroll tick with the position as a percent of each axis's scrollable range."),
                prop("on_top_reached", "EventHandler<()>").doc("Fires once when the top edge is reached."),
                prop("on_bottom_reached", "EventHandler<()>").doc("Fires once when the bottom edge is reached."),
                prop("on_left_reached", "EventHandler<()>").doc("Fires once when the left edge is reached."),
                prop("on_right_reached", "EventHandler<()>").doc("Fires once when the right edge is reached."),
                prop("children", "Element").doc("The scrollable content."),
            ]), props("Virtualize", vec![
                prop("count", "usize")
                    .doc("Rows in the whole list, not just the rendered ones."),
                prop("item", "Callback<usize, Element>")
                    .doc("Renders one row. Called only for the rows in view."),
                prop("item_size", "f64")
                    .doc("Row pitch in px - a row's height plus the gap below it. Measured from the first rows rendered when unset."),
                prop("overscan", "usize")
                    .default("theme.scroll_area.overscan")
                    .doc("Rows kept beyond each edge, so a scroll has something to reveal before the next render lands."),
            ]).without_base_props()],
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

            DocSection {
                title: "Keyboard access",
                Text {
                    "A scroll area is not a tab stop. Its content usually carries its own "
                    "focusable elements, and tabbing to those scrolls them into view, so a "
                    "stop on the viewport would be one more press on the way to anything "
                    "useful - Chromium adds one to every overflowing region, and "
                    Code { source: "ScrollArea" }
                    " opts out of it."
                }
                Text {
                    "Content that has nothing to focus is the other case: a strip of images, "
                    "or a block of text. There the viewport is the only thing to focus, and "
                    "without it the content cannot be read with a keyboard at all. "
                    Code { source: "focusable: true" }
                    " gives it the tab stop, and the browser's own arrow-key scrolling comes "
                    "with it."
                }
                CodeBlock { source: FOCUSABLE_EXAMPLE, language: "rust" }
                Box {
                    sx: sx()
                        .height("120px")
                        .width("100%")
                        .border("1px solid var(--lsx-grey-3)"),
                    ScrollArea {
                        id: "focusable-demo",
                        focusable: true,
                        role: "region",
                        aria_label: "Release notes",
                        Box {
                            sx: sx().padding("md"),
                            for i in 0..20 {
                                Text { key: "{i}", "Line {i} of text nothing can focus." }
                            }
                        }
                    }
                }
                Text {
                    "Name it as well as focus it. A tab stop that reads as nothing is worse "
                    "than none, so APG's scrollable-region pattern wants a "
                    Code { source: "role" }
                    " and an accessible name alongside the stop. "
                    Code { source: "ScrollArea" }
                    " sets no role of its own, so both go through the usual attribute spread."
                }
            }

            DocSection {
                title: "Virtualization",
                Text {
                    Code { source: "Virtualize" }
                    " renders only the rows the area can show. It draws no element of its "
                    "own, so it goes wherever the rows go - inside a "
                    Code { source: "List" }
                    ", a table body, a plain stack - and it needs a "
                    Code { source: "ScrollArea" }
                    " above it, which is what tells it where the viewport is and pads itself "
                    "with the rows that were skipped. The list below is "
                    "{VIRTUAL_ROWS} rows long; about a dozen exist at a time."
                }
                Box {
                    sx: sx()
                        .height("200px")
                        .width("100%")
                        .border("1px solid var(--lsx-grey-3)"),
                    ScrollArea {
                        List {
                            Virtualize {
                                count: VIRTUAL_ROWS,
                                item: move |index: usize| rsx! {
                                    ListItem {
                                        sx: sx().padding("sm"),
                                        "Row {index}"
                                    }
                                },
                            }
                        }
                    }
                }
                CodeBlock { source: VIRTUAL_EXAMPLE, language: "rust" }
                Text {
                    "Rows have to be a uniform height: it measures the first ones rendered "
                    "and assumes the rest match. Pass "
                    Code { source: "item_size" }
                    " to skip that measurement when the height is already known."
                }
            }
        }
    }
}
