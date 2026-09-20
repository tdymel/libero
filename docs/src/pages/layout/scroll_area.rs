use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, DocSection, UNSET, Wrap, a11y, indent, prop, props,
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
        "{PREAMBLE}rsx! {{\n    Flex {{\n        direction: \"column\",\n        gap: \"sm\",\n        sx: sx().width(\"100%\"),\n        Box {{\n            sx: sx().height(\"160px\").width(\"100%\").border(\"1px solid var(--lsx-muted-3)\"),\n{}        }}\n        Text {{ size: \"sm\", \"{{readout(position())}}, last edge: {{edge()}}\" }}\n        Flex {{\n            gap: \"sm\",\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to_percent(None, Some(0.0)), \"Scroll to top\" }}\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to_percent(None, Some(100.0)), \"Scroll to bottom\" }}\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to(0.0, 120.0), \"Scroll to 120px\" }}\n        }}\n    }}\n}}",
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
                    .doc("Which axes scroll and show a scrollbar. `none` clips the overflow."),
                prop("scrollbar_visibility", "ScrollbarVisibility")
                    .default("always")
                    .doc("When the scrollbar shows, `always`, `hover` or `hidden`. `scroll` acts like `hover` for now."),
                prop("scrollbar_size", "ScrollbarSize")
                    .default("thin")
                    .doc("The CSS `scrollbar-width`, `thin` or `auto`."),
                prop("scrollbar_color", "ThemeAwareValue")
                    .doc("Thumb color. The track stays transparent."),
                prop("scroll_position_x", "f64")
                    .doc("Scrolls to this percent (0-100) horizontally. A signal re-applies it on every change, a literal once at mount."),
                prop("scroll_position_y", "f64")
                    .doc("The same as `scroll_position_x`, vertically."),
                prop("handle", "ScrollAreaHandle")
                    .doc("From `use_scroll_area()`. Its `scroll_to_percent(x, y)` and `scroll_to(x, y)` in px scroll the area from any handler, on every call. A `None` percent keeps that axis; a call before the area mounts does nothing."),
                prop("focusable", "bool")
                    .default("false")
                    .doc("Makes the area a tab stop always. Without it, the area is one only while it overflows and holds nothing focusable. Your own `tabindex`, or a `role` other than `region`, turns that off."),
                prop("onscroll", "EventHandler<ScrollPositionEvent>")
                    .doc("Fires on every scroll with the position as a percent of each axis: `Start` when a scroll begins, `Change` while it runs, `End` when it stops."),
                prop("onresize", "EventHandler<Event<ResizeData>>")
                    .doc("Fires after the area resized."),
                prop("ontopreached", "EventHandler<()>").doc("Fires once when the top edge is reached."),
                prop("onbottomreached", "EventHandler<()>").doc("Fires once when the bottom edge is reached."),
                prop("onleftreached", "EventHandler<()>").doc("Fires once when the left edge is reached."),
                prop("onrightreached", "EventHandler<()>").doc("Fires once when the right edge is reached."),
                prop("children", "Element").doc("The scrollable content."),
            ]), props("Virtualize", vec![
                prop("count", "usize")
                    .doc("Rows in the whole list, not only the rendered ones."),
                prop("item", "Callback<usize, Element>")
                    .doc("Renders one row. Called only for the rows in view."),
                prop("item_size", "f64")
                    .doc("A row's height plus the gap below it, in px. Unset, it is measured from the first rows. Every row must have the same height."),
                prop("overscan", "usize")
                    .default("4")
                    .doc("Rows rendered beyond each edge, so a fast scroll has something to show."),
            ]).without_base_props()],
            accessibility: a11y()
                .handles([
                    "Tab reaches focusable content inside the area as usual.",
                    "When the content has nothing to focus, like a block of text, the area itself becomes a tab stop while it overflows, so the arrow keys can scroll it. `focusable: true` keeps the stop always.",
                    "A debug build warns about a tab stop without a name.",
                ])
                .must([
                    "Name the area with `aria_label` or `aria_labelledby`.",
                    "Use `scrollbars: \"none\"` only where something else scrolls: it puts the clipped content out of reach.",
                ]),
            lead: rsx! {
                Text {
                    "Scrolls its content and fills its parent, so give the parent a size. "
                    Code { source: "onscroll" }
                    " reports the position as a percent of each axis, and each edge has "
                    "its own event. The buttons scroll through a handle from "
                    Code { source: "use_scroll_area()" }
                    ". With virtualize on, "
                    Code { source: "Virtualize" }
                    " renders only the rows in view out of 50,000. It draws no element of "
                    "its own and needs a "
                    Code { source: "ScrollArea" }
                    " above it, one per area. Without one, or as the second, it warns and "
                    "renders every row."
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
                        .labels(["Vertical", "Horizontal", "Both", "None"])
                        .default(theme.scroll_area.scrollbars.as_str()),
                    Control::toggle("scrollbar_visibility", ["always", "hover", "hidden"])
                        .labels(["Always", "Hover", "Hidden"])
                        .default(theme.scroll_area.visibility.as_str()),
                    Control::toggle("scrollbar_size", ["thin", "auto"])
                        .labels(["Thin", "Auto"])
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
                        Text { size: "sm", "{readout(position())}, last edge: {edge()}" }
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
                title: "Text with nothing to focus",
                Text {
                    "The area itself becomes a tab stop while it overflows, so the arrow keys "
                    "can scroll it. Tab to the box below and press the arrow keys."
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
            }

        }
    }
}
