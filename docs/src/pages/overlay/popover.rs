use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
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
        "left" => Side::Left,
        "right" => Side::Right,
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

/// `generate_code` prints rsx props, and what this page has to show is a
/// builder chain inside a hook call - so the snippet is rebuilt from the
/// control values by hand, and preview-equals-code is maintained here rather
/// than by the generator.
fn wrap_hook_call(values: &DemoValues, _generated: &str) -> String {
    let mut options = format!(
        "PopoverOptions::new({}.0, theme.popover.padding)\n    .side(Side::{})\n    .align(Align::{})",
        values.str("gap"),
        match values.str("side").as_str() {
            "top" => "Top",
            "left" => "Left",
            "right" => "Right",
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

/// The hook call lives here rather than in `Demo`'s render closure: a hook
/// called there would land in `Demo`'s own hook slots and shift every time a
/// control changed what the closure does.
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

    // A dialog takes focus once it opens, but only once placed: until then it
    // is `visibility: hidden` and `focus()` moves nothing. Once per opening, so
    // a re-measure does not pull focus back from a control on the page.
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
                        .doc("Where the box lines up along that side's cross axis."),
                    prop("gap", "f64")
                        .default("theme.popover.gap")
                        .doc("Pixels between the anchor's edge and the box."),
                    prop("padding", "f64")
                        .default("theme.popover.padding")
                        .doc("How close to a viewport edge the box may come before it flips or shifts. The box is also never wider than the viewport less this at both edges."),
                    prop("flip", "bool")
                        .default("true")
                        .doc("Move to the opposite side when the preferred one has no room."),
                    prop("shift", "bool")
                        .default("true")
                        .doc("Slide along the side to stay on screen, once flipping cannot help."),
                    prop("width", "PopoverWidth")
                        .default("Auto")
                        .doc("Whether the box follows its own content, the anchor's width exactly, or at least the anchor's width."),
                    prop("remeasure", "u64")
                        .default("0")
                        .doc("Not a placement input: changing it re-measures. For an anchor that resizes while the box is open - nothing else re-measures."),
                    prop("dismiss", "bool")
                        .default("false")
                        .doc("Escape anywhere and a press outside close the box, through `on_dismiss`. Spread `anchor_events()` on the trigger and `floating_events()` on the box."),
                ]).without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "A popover is a hook, not a component: a dropdown, a menu and a hover "
                    "card share when and where, never what the box looks like. "
                    Code { source: "use_popover" }
                    " portals the box to the document root, so it escapes an "
                    Code { source: "overflow: hidden" }
                    " ancestor, and places it in viewport coordinates, flipping and shifting "
                    "to stay on screen. It owns no open state - "
                    Code { source: "show(None)" }
                    " is how a closed popover stops rendering. Popovers nest with nothing "
                    "extra: anchor the inner one to a row inside the outer box, and the one "
                    "shown later paints over the earlier."
                }
            },
            // snippet: let mut opened = use_signal(|| false);
            // snippet: let theme = use_theme();
            Demo {
                component: "PopoverDemo",
                children_text: "",
                controls: vec![
                    Control::toggle("side", ["top", "right", "bottom", "left"]).default("bottom"),
                    Control::toggle("align", ["start", "center", "end"]).default("start"),
                    Control::toggle("width", ["auto", "match", "min"]).default("auto"),
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
                title: "Accessibility",
                Text {
                    "The hook contributes no role and no keyboard - a popover has no "
                    "semantics. The consumer supplies "
                    Code { source: "role" }
                    " on the box and the "
                    Code { source: "aria-haspopup" }
                    ", "
                    Code { source: "aria-expanded" }
                    " and "
                    Code { source: "aria-controls" }
                    " that name the trigger, as the demo above does. Drive "
                    Code { source: "aria-expanded" }
                    " from the same signal the hook is given: a trigger claiming to be closed "
                    "over an open box is announced as closed."
                }
                Text {
                    "Escape must close the box (WCAG 2.1 SC 1.4.13). "
                    Code { source: "dismiss(true)" }
                    " does it, plus a press outside: "
                    Kbd { "Esc" }
                    " anywhere closes the box and hands focus back to the trigger when it was "
                    "on the trigger or in the box; focus leaving both closes it and stays where "
                    "it went. Give the box "
                    Code { source: "tabindex=\"-1\"" }
                    ", or a click on its text moves focus out and closes it. Safari does not "
                    "focus a button on click, so there a press outside a pointer-opened box "
                    "does not close it. Off the web only the element holding focus hears "
                    Kbd { "Esc" }
                    ", which is why both event lists are spread. Focus is deliberately not "
                    "trapped - Tab closes the surface and moves on. If you animate the close, give the closing box "
                    Code { source: "visibility: hidden" }
                    " or "
                    Code { source: "inert" }
                    " for the duration: until it unmounts it is still tabbable and still "
                    "announced."
                }
            }

            DocSection {
                title: "What it cannot do",
                Text {
                    "Scroll tracking needs a document-level scroll notification, which only "
                    "the web answers today - natively an open popover drifts when the page "
                    "scrolls. Nothing tracks a resize on any backend; "
                    Code { source: "remeasure" }
                    " is the only answer there. "
                    Code { source: "dismiss" }
                    "'s press outside needs a renderer that can say where focus is: the web "
                    "and Blitz can, a WebView cannot, so there only "
                    Kbd { "Esc" }
                    " and your own handlers close the box."
                }
            }
        }
    }
}
