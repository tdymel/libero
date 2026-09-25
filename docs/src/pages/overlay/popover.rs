use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, CodeBlock, Kbd, Text},
    hooks::{Align, PopoverOptions, PopoverWidth, Side, use_element, use_id, use_popover},
    platform::ElementApi,
    sx::sx,
    use_theme,
};

const GAPS: [&str; 4] = ["0", "4", "8", "16"];

// snippet: ignore - pieces of a component built on `use_popover`
const FOCUS_AFTER_PLACED: &str = r#"// Wrong: the box is mounted but not measured, so it is still
// `visibility: hidden`. `focus()` answers Ok(()) and nothing moves.
use_effect(move || {
    if opened() {
        let _ = first_item.focus();
    }
});

// Right: wait for the measurement.
use_effect(move || {
    if !opened() || !popover.placed() {
        return;
    }
    let _ = first_item.focus();
});"#;

// snippet: ignore - pieces of a component built on `use_popover`
const CONTEXT_ACROSS_THE_PORTAL: &str = r#"// The box renders at the portal outlet, at the document root, so it
// inherits none of the context around the call site. Re-provide what
// the content needs, inside the content itself.
popover.show(opened().then(|| rsx! {
    MenuProvider { context: menu, {items} }
}));

// A signal a callback outside every scope writes has to outlive the
// scope that made it, and be dropped by hand.
let tick = use_hook(|| Signal::new_in_scope(0u64, ScopeId::ROOT));
use_drop(move || tick.manually_drop());"#;

fn side_of(value: &str) -> Side {
    match value {
        "top" => Side::Top,
        "start" => Side::Start,
        "end" => Side::End,
        _ => Side::Bottom,
    }
}

fn align_of(value: &str) -> Align {
    match value {
        "center" => Align::Center,
        "end" => Align::End,
        _ => Align::Start,
    }
}

fn width_of(value: &str) -> PopoverWidth {
    match value {
        "match" => PopoverWidth::Match,
        "min" => PopoverWidth::Min,
        _ => PopoverWidth::Auto,
    }
}

/// `generate_code` prints rsx props, but this page shows a builder chain in a
/// hook call. So the snippet is rebuilt from the control values by hand.
fn wrap_hook_call(values: &DemoValues, _generated: &str) -> String {
    let mut options = format!(
        "PopoverOptions::new({}.0, theme.popover.padding)\n    .side(Side::{})\n    .align(Align::{})",
        values.str("gap"),
        match values.str("side").as_str() {
            "top" => "Top",
            "start" => "Start",
            "end" => "End",
            _ => "Bottom",
        },
        match values.str("align").as_str() {
            "center" => "Center",
            "end" => "End",
            _ => "Start",
        },
    );
    if values.str("width") != "auto" {
        options.push_str(&format!(
            "\n    .width(PopoverWidth::{})",
            match values.str("width").as_str() {
                "match" => "Match",
                _ => "Min",
            }
        ));
    }
    if values.str("flip") == "false" {
        options.push_str("\n    .flip(false)");
    }
    if values.str("shift") == "false" {
        options.push_str("\n    .shift(false)");
    }
    options.push_str("\n    .dismiss(true)");

    format!(
        "let anchor = use_element();\nlet popover = use_popover(anchor, opened(), {options});\n\
         popover.on_dismiss(move || opened.set(false));\n\n\
         popover.show(opened().then(|| rsx! {{\n\
         {}\
         }}));\n\n\
         rsx! {{\n\
         {}\
         }}",
        indent(
            "Box {\n    tabindex: \"-1\",\n    attributes: popover.floating_events(),\n    style: popover.style(),\n    onmounted: popover.floating().mount(),\n    \"Popover content\"\n}"
        ),
        indent(
            "Button {\n    onmounted: anchor.mount(),\n    onclick: move |_| opened.toggle(),\n    attributes: popover.anchor_events(),\n    \"Popover\"\n}"
        )
    )
}

/// Its own component: hooks in `Demo`'s render closure would land in `Demo`'s hook slots.
#[component]
fn PopoverDemo(
    side: String,
    align: String,
    width: String,
    flip: bool,
    shift: bool,
    gap: f64,
) -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    // One id, owned here and pointed at from the trigger.
    let box_id = use_id();

    let options = PopoverOptions::new(gap, theme.popover.padding)
        .side(side_of(&side))
        .align(align_of(&align))
        .width(width_of(&width))
        .flip(flip)
        .shift(shift)
        .dismiss(true);
    let popover = use_popover(anchor, opened(), options);
    // Escape anywhere and a press outside.
    popover.on_dismiss(move || opened.set(false));
    let floating = *popover.floating();

    // Focus once placed (before, `visibility: hidden` ignores `focus()`), once per opening,
    // so a re-measure doesn't pull focus back.
    let mut entered = use_signal(|| false);
    use_effect(move || match (opened(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            let _ = floating.focus();
        }
        (false, _) => entered.set(false),
        _ => {}
    });
    // Tab closes it and hands focus back to the trigger; forwards, the
    // browser's own Tab then moves on from there.
    let mut close = move |event: &KeyboardEvent| {
        opened.set(false);
        let _ = anchor.focus();
        if event.modifiers().shift() {
            event.prevent_default();
        }
    };

    popover.show(opened().then(|| {
        rsx! {
            Box {
                id: "{box_id}",
                // Plain text, so a dialog and not a listbox: a listbox has to
                // hold options.
                role: "dialog",
                aria_label: "Example popover",
                // Focusable, so a click on its text stays inside.
                tabindex: "-1",
                attributes: popover.floating_events(),
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Tab {
                        close(&event);
                    }
                },
                style: popover.style(),
                onmounted: floating.mount(),
                sx: sx()
                    .background("surface")
                    .border("1px solid var(--lsx-muted-3)")
                    .border_radius("6px")
                    .box_shadow("md")
                    .padding("var(--lsx-popover-padding)")
                    .z_index("var(--lsx-z-index-popover)"),
                "Popover content"
            }
        }
    }));

    rsx! {
        Box {
            sx: sx().padding("60px").display("flex").justify_content("center"),
            Button {
                onmounted: anchor.mount(),
                onclick: move |_| opened.toggle(),
                attributes: popover.anchor_events(),
                // Focus is back here after a click that closed the box, or
                // before the box is placed.
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Tab && opened() {
                        opened.set(false);
                    }
                },
                aria_haspopup: "dialog",
                // The same signal the hook is given, never a second copy -
                // nothing else stops the two disagreeing.
                aria_expanded: "{opened()}",
                aria_controls: "{box_id}",
                variant: "outlined",
                // One name; `aria-expanded` says whether it is open.
                "Popover"
            }
        }
    }
}

#[component]
pub fn PopoverPage() -> Element {
    rsx! {
        DocPage {
            title: "Popover",
            source: "libero/src/hooks/popover/mod.rs",
            markdown: "/md/popover.md",
            properties: vec![
                props("PopoverOptions", vec![
                    prop("side", "Side")
                        .default("Bottom")
                        .doc("The preferred side of the anchor. Flipping may override it."),
                    prop("align", "Align")
                        .default("Start")
                        .doc("Where the box lines up along that side."),
                    prop("gap", "f64")
                        .default("theme.popover.gap")
                        .doc("Pixels between the anchor and the box."),
                    prop("padding", "f64")
                        .default("theme.popover.padding")
                        .doc("How close to a viewport edge the box may come before it flips or shifts. The box is never wider than the viewport less this on both sides."),
                    prop("flip", "bool")
                        .default("true")
                        .doc("Moves to the opposite side when the preferred one has no room."),
                    prop("shift", "bool")
                        .default("true")
                        .doc("Slides along the side to stay on screen when flipping does not help."),
                    prop("width", "PopoverWidth")
                        .default("Auto")
                        .doc("`Auto` follows the content, `Match` takes the anchor's width, and `Min` is at least the anchor's width."),
                    prop("remeasure", "u64")
                        .default("0")
                        .doc("Changing it measures the box again. Use it for an anchor that resizes while the box is open."),
                    prop("dismiss", "bool")
                        .default("false")
                        .doc("Escape and a press outside close the box, through `on_dismiss`. Spread `anchor_events()` on the trigger and `floating_events()` on the box."),
                ]).without_base_props(),
                props("PopoverHandle", vec![
                    prop("floating()", "&ElementHandle")
                        .doc("Mount it on the box. Nothing is placed until it is attached."),
                    prop("placed()", "bool")
                        .doc("Whether the box has been measured. `false` on the render that opens it."),
                    prop("placement()", "Placement")
                        .doc("The side and align the box landed on, after flipping."),
                    prop("style()", "Option<String>")
                        .doc("The box's `style`, with its position and width."),
                    prop("show(content)", "Option<Element>")
                        .doc("Renders the box. `None` removes it."),
                    prop("on_dismiss(f)", "impl FnMut()")
                        .doc("What Escape and a press outside call, with `dismiss` on. Call it on every render."),
                    prop("anchor_events()", "Vec<Attribute>")
                        .doc("Spread on the trigger: `dismiss`, and on a WebView the link `Hotkey::within` follows into the box."),
                    prop("floating_events()", "Vec<Attribute>")
                        .doc("Spread on the box, as `anchor_events()` on the trigger; the WebView link needs both."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Escape"], "With `dismiss(true)`: closes the box and returns focus to the trigger.")
                .key(["Tab"], "Closes the box and moves on. Focus is not trapped.")
                .handles([
                    "The hook adds no role and no keys of its own. `dismiss(true)` adds the Escape key and closing on focus leaving.",
                    "With `dismiss(true)`, focus leaving the trigger and the box closes it.",
                ])
                .must([
                    "Put a `role` on the box, and `aria-haspopup`, `aria-expanded` and `aria-controls` on the trigger, as the example does.",
                    "Drive `aria-expanded` from the same signal the hook gets, or a screen reader hears the wrong state.",
                    "Make Escape close the box (WCAG 1.4.13): turn on `dismiss(true)`, or handle it yourself.",
                    "Give the box `tabindex=\"-1\"`, or a click on its text moves focus out and closes it.",
                    "If you animate the close, give the closing box `visibility: hidden` or `inert`. Until it unmounts, it is still tabbable and still announced.",
                ])
                .limits([
                    "Safari does not focus a button on click, so there a press outside a box opened by pointer does not close it.",
                ]),
            lead: rsx! {
                Text {
                    "A popover is a hook, not a component. A dropdown, a menu and a hover "
                    "card share where the box goes, not how it looks. "
                    Code { source: "use_popover" }
                    " portals the box, so no "
                    Code { source: "overflow: hidden" }
                    " ancestor clips it, and flips and shifts it to stay on screen."
                }
                Text {
                    "It owns no open state. Pass "
                    Code { source: "show(None)" }
                    " to take a closed box away. Popovers nest. Anchor the inner one to a row "
                    "in the outer box, and the one shown later paints on top."
                }
            },
            // snippet: let mut opened = use_signal(|| false);
            // snippet: let theme = use_theme();
            Demo {
                component: "PopoverDemo",
                children_text: "",
                controls: vec![
                    Control::toggle("side", ["top", "end", "bottom", "start"])
                        .labels(["Top", "End", "Bottom", "Start"])
                        .default("bottom"),
                    Control::toggle("align", ["start", "center", "end"])
                        .labels(["Start", "Center", "End"])
                        .default("start"),
                    Control::toggle("width", ["auto", "match", "min"])
                        .labels(["Auto", "Match", "Min"])
                        .default("auto"),
                    Control::slider("gap", GAPS).default("4"),
                    Control::switch("flip").default("true"),
                    Control::switch("shift").default("true"),
                ],
                render: move |values: DemoValues| rsx! {
                    PopoverDemo {
                        side: values.str("side"),
                        align: values.str("align"),
                        width: values.str("width"),
                        flip: values.str("flip") == "true",
                        shift: values.str("shift") == "true",
                        gap: values.str("gap").parse::<f64>().unwrap_or(4.0),
                    }
                },
                wrap: Wrap(wrap_hook_call),
            }

            DocSection {
                title: "Focus after placed, never after mount",
                CodeBlock { source: FOCUS_AFTER_PLACED, language: "rust" }
            }

            DocSection {
                title: "Context across the portal",
                CodeBlock { source: CONTEXT_ACROSS_THE_PORTAL, language: "rust" }
            }

            DocSection {
                title: "What it cannot do",
                Text {
                    "Only the web tells the box when the page scrolls. Elsewhere an open "
                    "popover drifts. No backend tracks a resize, so use "
                    Code { source: "remeasure" }
                    ". A press outside needs to know where focus is. The web and Blitz can "
                    "tell, a WebView cannot, so there only "
                    Kbd { "Esc" }
                    " and your own handlers close the box."
                }
            }
        }
    }
}
