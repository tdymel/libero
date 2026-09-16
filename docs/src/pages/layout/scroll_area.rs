use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, DocSection, UNSET, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Button, Code, CodeBlock, Flex, Input, List, ListItem, ScrollArea, ScrollPositionEvent,
        Text, Virtualize, use_scroll_area,
    },
    sx::sx,
    use_theme,
};

const FOCUSABLE_EXAMPLE: &str = r#"ScrollArea {
    aria_label: "Release notes",
    Text { "..." }
}"#;

/// Fifty thousand rows, of which the preview renders about a dozen.
const VIRTUAL_ROWS: usize = 50_000;

/// What the `virtualize` switch puts in the area instead of `CONTENT`.
const VIRTUAL_CONTENT: &str = r#"List {
    Virtualize {
        count: 50_000,
        item: move |index: usize| rsx! {
            ListItem { sx: sx().padding("sm"), "Row {index}" }
        },
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
/// props are the demo's fixture rather than anything a control varies. `area`,
/// `position` and `edge` are declared in `PREAMBLE`. The plain content makes
/// the area a tab stop, so it needs a name.
const FIXED: [&str; 5] = [
    r#"aria_label: "Items""#,
    "handle: area",
    "onscroll: move |event: ScrollPositionEvent| position.set(event)",
    r#"ontopreached: move |_| edge.set("top")"#,
    r#"onbottomreached: move |_| edge.set("bottom")"#,
];

/// A raw `{event:?}` prints an unrounded `f64` and reflows the row on every
/// scroll tick, so the readout formats the two percentages itself.
/// **Kept in step with `PREAMBLE` by hand.**
fn readout(event: ScrollPositionEvent) -> String {
    let (kind, x, y) = match event {
        ScrollPositionEvent::Start(x, y) => ("Start", x, y),
        ScrollPositionEvent::Change(x, y) => ("Change", x, y),
        ScrollPositionEvent::End(x, y) => ("End", x, y),
    };
    format!("{kind} at x {x:.0}%, y {y:.0}%")
}

/// The helper, the handle and the two signals the preview's wiring uses,
/// printed above the rsx so the snippet compiles as it stands.
const PREAMBLE: &str = r#"fn readout(event: ScrollPositionEvent) -> String {
    let (kind, x, y) = match event {
        ScrollPositionEvent::Start(x, y) => ("Start", x, y),
        ScrollPositionEvent::Change(x, y) => ("Change", x, y),
        ScrollPositionEvent::End(x, y) => ("End", x, y),
    };
    format!("{kind} at x {x:.0}%, y {y:.0}%")
}

let area = use_scroll_area();
let mut position = use_signal(|| ScrollPositionEvent::Change(0.0, 0.0));
let mut edge = use_signal(|| "none");

"#;

/// A scroll area fills its parent, so the preview has to give it one - and the
/// readout and the three jump buttons are the other half of the wiring above,
/// so they belong in the preview too. The code block prints all of it.
fn wrap_frame(_: &DemoValues, code: &str) -> String {
    format!(
        "{PREAMBLE}rsx! {{\n    Flex {{\n        direction: \"column\",\n        gap: \"sm\",\n        sx: sx().width(\"100%\"),\n        Box {{\n            sx: sx().height(\"160px\").width(\"100%\").border(\"1px solid var(--lsx-muted-3)\"),\n{}        }}\n        Text {{ size: \"sm\", \"{{readout(position())}} - last edge: {{edge()}}\" }}\n        Flex {{\n            gap: \"sm\",\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to_percent(None, Some(0.0)), \"Scroll to top\" }}\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to_percent(None, Some(100.0)), \"Scroll to bottom\" }}\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to(0.0, 120.0), \"Scroll to 120px\" }}\n        }}\n    }}\n}}",
        indent(&indent(&indent(code)))
    )
}

#[component]
pub fn ScrollAreaPage() -> Element {
    let theme = use_theme();
    let mut position = use_signal(|| ScrollPositionEvent::Change(0.0, 0.0));
    let mut edge = use_signal(|| "none");
    let area = use_scroll_area();

    rsx! {
        DocPage {
            title: "ScrollArea",
            source: "libero/src/components/layout/scroll_area/scroll_area.rs",
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
                prop("handle", "ScrollAreaHandle")
                    .doc("From use_scroll_area(). Its scroll_to_percent(x, y) and px scroll_to(x, y) scroll the area from any handler, and every call scrolls - unlike the two props above, which only re-apply when their value changes."),
                prop("focusable", "bool")
                    .default("false")
                    .doc("Makes the viewport a `region` tab stop always. Unset, it is one only while it overflows and holds nothing focusable. A caller's `tabindex` or non-region `role` turns the automatic stop off."),
                prop("onscroll", "EventHandler<ScrollPositionEvent>")
                    .doc("Fires on every scroll tick with the position as a percent of each axis's scrollable range."),
                prop("onresize", "EventHandler<Event<ResizeData>>")
                    .doc("Fires after the area resized, once it has re-measured itself for a Virtualize child."),
                prop("ontopreached", "EventHandler<()>").doc("Fires once when the top edge is reached."),
                prop("onbottomreached", "EventHandler<()>").doc("Fires once when the bottom edge is reached."),
                prop("onleftreached", "EventHandler<()>").doc("Fires once when the left edge is reached."),
                prop("onrightreached", "EventHandler<()>").doc("Fires once when the right edge is reached."),
                prop("children", "Element").doc("The scrollable content."),
            ]), props("Virtualize", vec![
                prop("count", "usize")
                    .doc("Rows in the whole list, not just the rendered ones."),
                prop("item", "Callback<usize, Element>")
                    .doc("Renders one row. Called only for the rows in view."),
                prop("item_size", "f64")
                    .doc("Row pitch in px - a row's height plus the gap below it. Rows must be a uniform height: unset, it is measured from the first rows rendered and assumed for the rest."),
                prop("overscan", "usize")
                    .default("theme.scroll_area.overscan")
                    .doc("Rows kept beyond each edge, so a scroll has something to reveal before the next render lands."),
            ]).without_base_props()],
            lead: rsx! {
                Text {
                    "Scrolls its content, filling the parent by default. "
                    Code { source: "onscroll" }
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
                    " scroll it to a percent, re-applied only when the value changes. The buttons use "
                    Code { source: "let area = use_scroll_area();" }
                    " instead, a handle whose every call scrolls, in percent or in px. Each edge has its own event; the demo wires "
                    Code { source: "ontopreached" }
                    " and "
                    Code { source: "onbottomreached" }
                    ", and "
                    Code { source: "onleftreached" }
                    "/"
                    Code { source: "onrightreached" }
                    " work the same way."
                }
            },
            Demo {
                component: "ScrollArea",
                children_text: "",
                code_child: Child(|values: &DemoValues| match values.str("virtualize").as_str() {
                    "true" => VIRTUAL_CONTENT.to_string(),
                    _ => CONTENT.to_string(),
                }),
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    Control::toggle("scrollbars", ["vertical", "horizontal", "both", "none"])
                        .default(theme.scroll_area.scrollbars.as_str()),
                    Control::toggle("scrollbar_visibility", ["always", "hover", "hidden"])
                        .default(theme.scroll_area.visibility.as_str()),
                    Control::toggle("scrollbar_size", ["thin", "auto"])
                        .default(theme.scroll_area.size.as_str()),
                    // The unset thumb is grey-5, which a bare `grey` would
                    // *not* resolve to - so the first swatch is unset, and it
                    // is painted grey-5 rather than white: the swatch shows
                    // what the scrollbar actually draws without the prop.
                    Control::color("scrollbar_color").with_unset()
                    .unset_swatch("muted.5"),
                    // Not a prop: swaps the content for a `Virtualize` list,
                    // which renders only the rows in view.
                    Control::switch("virtualize").code(|_, _| vec![]),
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
                                .border("1px solid var(--lsx-muted-3)"),
                            ScrollArea {
                                scrollbars: values.str("scrollbars"),
                                scrollbar_visibility: values.str("scrollbar_visibility"),
                                scrollbar_size: values.str("scrollbar_size"),
                                scrollbar_color: match values.str("scrollbar_color").as_str() {
                                    UNSET => Input::None,
                                    color => Input::from(color),
                                },
                                aria_label: "Items",
                                handle: area,
                                onscroll: move |event: ScrollPositionEvent| {
                                    position.set(event)
                                },
                                ontopreached: move |_| edge.set("top"),
                                onbottomreached: move |_| edge.set("bottom"),
                                if values.str("virtualize") == "true" {
                                    List {
                                        Virtualize {
                                            count: VIRTUAL_ROWS,
                                            item: move |index: usize| rsx! {
                                                ListItem { sx: sx().padding("sm"), "Row {index}" }
                                            },
                                        }
                                    }
                                } else {
                                    Box {
                                        sx: sx().width("150%").padding("md"),
                                        for i in 0..20 {
                                            Text { key: "{i}", "Item {i}" }
                                        }
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
                                onclick: move |_| area.scroll_to_percent(None, Some(0.0)),
                                "Scroll to top"
                            }
                            Button {
                                size: "sm",
                                variant: "outlined",
                                onclick: move |_| area.scroll_to_percent(None, Some(100.0)),
                                "Scroll to bottom"
                            }
                            Button {
                                size: "sm",
                                variant: "outlined",
                                onclick: move |_| area.scroll_to(0.0, 120.0),
                                "Scroll to 120px"
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_frame),
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "A scroll area whose content has focusable elements is not a tab stop: "
                    "tabbing to those scrolls them into view."
                }
                Text {
                    "Content that has nothing to focus is the other case: a strip of images, "
                    "or a block of text. There the viewport is the only thing to focus, and "
                    "without it the content cannot be read with a keyboard at all. So while "
                    "such content overflows, the area makes itself a tab stop with "
                    Code { source: "role=\"region\"" }
                    ", re-checked on every resize and re-render, and the browser's own "
                    "arrow-key scrolling comes with it. "
                    Code { source: "focusable: true" }
                    " keeps the stop whatever the content."
                }
                CodeBlock { source: FOCUSABLE_EXAMPLE, language: "rust" }
                Box {
                    sx: sx()
                        .height("120px")
                        .width("100%")
                        .border("1px solid var(--lsx-muted-3)"),
                    ScrollArea {
                        id: "focusable-demo",
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
                    "Name it as well. A tab stop that reads as nothing is worse than none, so "
                    "APG's scrollable-region pattern wants an accessible name alongside the "
                    "stop: pass "
                    Code { source: "aria_label" }
                    " or "
                    Code { source: "aria_labelledby" }
                    ". A debug build warns once about a stop without one."
                }
            }

        }
    }
}
