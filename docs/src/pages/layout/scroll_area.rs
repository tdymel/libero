use crate::components::{
    Child, Control, Demo, DemoFile, DemoValues, DocPage, DocSection, UNSET, Wrap, a11y, indent,
    prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Button, Code, Flex, Input, List, ListItem, ScrollArea, ScrollAreaPart,
        ScrollPositionEvent, Text, Virtualize, use_scroll_area,
    },
    sx::sx,
    use_theme,
};

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("scroll_area.rs"));

// demo-code: readout start
/// Rounded percentages: a raw `{event:?}` reflows the row every tick.
fn readout(event: ScrollPositionEvent) -> String {
    let (kind, x, y) = match event {
        ScrollPositionEvent::Start(x, y) => ("Start", x, y),
        ScrollPositionEvent::Change(x, y) => ("Change", x, y),
        ScrollPositionEvent::End(x, y) => ("End", x, y),
    };
    format!("{kind} at x {x:.0}%, y {y:.0}%")
}
// demo-code: readout end

/// Prints the sized parent a scroll area fills, and the readout and jump buttons it wires to.
fn wrap_frame(_: &DemoValues, code: &str) -> String {
    format!(
        "{}\n\n{}\n\nrsx! {{\n    Flex {{\n        direction: \"column\",\n        gap: \"sm\",\n        sx: sx().width(\"100%\"),\n        Box {{\n            sx: sx().height(\"160px\").width(\"100%\").border(\"1px solid var(--lsx-muted-3)\"),\n{}        }}\n        Text {{ size: \"sm\", \"{{readout(position())}}\" }}\n        // A reached edge is a result, so a screen reader hears it.\n        Text {{ size: \"sm\", role: \"status\", \"Last edge: {{edge()}}\" }}\n        Flex {{\n            gap: \"sm\",\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to_percent(None, Some(0.0)), \"Scroll to top\" }}\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to_percent(None, Some(100.0)), \"Scroll to bottom\" }}\n            Button {{ size: \"sm\", variant: \"outlined\", onclick: move |_| area.scroll_to(0.0, 120.0), \"Scroll to 120px\" }}\n        }}\n    }}\n}}",
        FILE.section("readout"),
        FILE.section("state"),
        indent(&indent(&indent(code)))
    )
}

#[component]
pub fn ScrollAreaPage() -> Element {
    let theme = use_theme();
    // demo-code: state start
    let area = use_scroll_area();
    let mut position = use_signal(|| ScrollPositionEvent::Change(0.0, 0.0));
    let mut edge = use_signal(|| "none");
    // demo-code: state end

    rsx! {
        DocPage {
            title: "ScrollArea",
            source: "libero/src/components/layout/scroll_area/scroll_area.rs",
            markdown: "/md/scroll_area.md",
            properties: vec![props("ScrollArea", vec![
                prop("scrollbars", "ScrollAxis")
                    .default(theme.scroll_area.scrollbars.as_str())
                    .doc("Which axes scroll and show a scrollbar. `none` clips the overflow."),
                prop("scrollbar_visibility", "ScrollbarVisibility")
                    .default(theme.scroll_area.visibility.as_str())
                    .doc("When the scrollbar shows, `always`, `hover` or `hidden`. `scroll` acts like `hover` for now. In a browser or WebView, `always` draws its own track and thumb, so the bar stays where the system overlays and fades its scrollbars; drag the thumb or press the track. The other values keep the native bar."),
                prop("scrollbar_size", "ScrollbarSize")
                    .default(theme.scroll_area.size.as_str())
                    .doc("`thin` or `auto`: the CSS `scrollbar-width`, or 8px and 12px for the bar `always` draws."),
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
                prop("onbottomreached", "EventHandler<()>").doc("Fires once when the bottom edge is reached, and again when content added at the bottom is scrolled to its new end, say a list loading more."),
                prop("onleftreached", "EventHandler<()>").doc("Fires once when the left edge is reached."),
                prop("onrightreached", "EventHandler<()>").doc("Fires once when the right edge is reached."),
                prop("children", "Element").doc("The scrollable content."),
                prop("parts", "Parts<ScrollAreaPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("ScrollAreaPart", vec![
                (ScrollAreaPart::Scrollbar, "A track `always` draws in a browser. `data-orientation` is `vertical` or `horizontal`."),
                (ScrollAreaPart::Thumb, "The thumb inside a track."),
            ]), props("Virtualize", vec![
                prop("count", "usize").default("required")
                    .doc("Rows in the whole list, not only the rendered ones."),
                prop("item", "Callback<usize, Element>").default("required")
                    .doc("Renders one row. Called only for the rows in view."),
                prop("item_size", "f64")
                    .doc("A row's height plus the gap below it, in px. Unset, it is measured from the first rows, and again when the area's width changes. Every row must have the same height."),
                prop("overscan", "usize")
                    .default(theme.scroll_area.overscan.to_string())
                    .doc("Rows rendered beyond each edge, so a fast scroll has something to show."),
                prop("keep_rendered", "usize")
                    .doc("An index rendered even out of view, e.g. the row holding focus, so scrolling it away keeps the focus."),
                prop("item_key", "Callback<usize, String>")
                    .default("the index")
                    .doc("A row's identity, such as its data's id. A row's state (focus, typed text, open details) follows its key, so set it when rows can be sorted, inserted or removed."),
            ]).without_base_props()],
            accessibility: a11y()
                .handles([
                    "Tab reaches focusable content inside the area as usual.",
                    "When the content has nothing to focus, like a block of text, the area itself becomes a tab stop while it overflows, so the arrow keys can scroll it. The Usage preview is one: Tab to it and press the arrow keys. `focusable: true` keeps the stop always.",
                    "A debug build warns about a tab stop without a name.",
                    "On Blitz, which scrolls nothing on a key, the area scrolls itself on the arrows, Page Up and Down, Home, End and Space, as a browser does.",
                    "The bar `always` draws is hidden from screen readers and takes no focus: the area itself scrolls by keyboard, wheel and touch. Forced colours paint its thumb in the system text colour.",
                ])
                .must([
                    "Name the area with `aria_label` or `aria_labelledby`.",
                    "Use `scrollbars: \"none\"` only where something else scrolls: it puts the clipped content out of reach.",
                    "Give each `Virtualize` row `aria_setsize: count` and `aria_posinset: index + 1`, as the virtualize preview does: only the rows in view exist, so a screen reader cannot count the rest. For a table row, `aria-rowcount` and `aria-rowindex`.",
                    "Pass `focusable: true` when the content holds only controls hidden by CSS (`visibility: hidden`, `display: none`): the area counts them as focusable and makes no tab stop.",
                    "Pad the content by 6 px or more where a focusable child sits flush with the area's edge: the area clips the outset focus ring there.",
                ])
                .example("A terms text in `ScrollArea { aria_label: \"Terms of service\", .. }`: Tab stops on the area, a screen reader reads its name, and the arrow keys scroll it."),
            lead: rsx! {
                Text {
                    "Scrolls its content and fills its parent, so give the parent a size. "
                    Code { source: "onscroll" }
                    " reports the position as a percent of each axis, and each edge has "
                    "its own event. The buttons scroll through a handle from "
                    Code { source: "use_scroll_area()" }
                    "."
                }
            },
            Demo {
                component: "ScrollArea",
                children_text: "",
                code_child: Child(|values: &DemoValues| match values.str("virtualize").as_str() {
                    "true" => FILE.section("virtual").to_string(),
                    _ => FILE.section("content").to_string(),
                }),
                fixed: FILE
                    .section("wiring")
                    .lines()
                    .map(|line| line.trim_end_matches(',').to_string())
                    .collect(),
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
                    // Unset draws grey-6, which bare `grey` is not: the unset swatch is painted grey-6.
                    Control::color("scrollbar_color").with_unset()
                    .unset_swatch("muted.6"),
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
                                // The plain content makes the area a tab stop, so it needs a name.
                                // demo-code: wiring start
                                aria_label: "Items",
                                handle: area,
                                onscroll: move |event: ScrollPositionEvent| position.set(event),
                                ontopreached: move |_| edge.set("top"),
                                onbottomreached: move |_| edge.set("bottom"),
                                // demo-code: wiring end
                                if values.str("virtualize") == "true" {
                                    // demo-code: virtual start
                                    List {
                                        Virtualize {
                                            count: 50_000,
                                            item: move |index: usize| rsx! {
                                                ListItem {
                                                    sx: sx().padding("sm"),
                                                    aria_setsize: 50_000,
                                                    aria_posinset: index + 1,
                                                    "Row {index}"
                                                }
                                            },
                                        }
                                    }
                                    // demo-code: virtual end
                                } else {
                                    // demo-code: content start
                                    Box {
                                        sx: sx().width("150%").padding("md"),
                                        for i in 0..20 {
                                            Text { key: "{i}", "Item {i}" }
                                        }
                                    }
                                    // demo-code: content end
                                }
                            }
                        }
                        // The position changes per scroll and stays silent; a reached edge is a result.
                        Text { size: "sm", "{readout(position())}" }
                        Text { size: "sm", role: "status", "Last edge: {edge()}" }
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
                title: "Virtualize",
                Text {
                    "With virtualize on, "
                    Code { source: "Virtualize" }
                    " renders only the rows in view out of 50,000. It draws no element of "
                    "its own and needs a "
                    Code { source: "ScrollArea" }
                    " above it, one per area. Without one, or as the second, it warns and "
                    "renders every row."
                }
            }
        }
    }
}
