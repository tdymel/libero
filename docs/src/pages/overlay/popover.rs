use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, CodeBlock, Text},
    hooks::{Align, PopoverOptions, PopoverWidth, Side, use_element, use_popover},
    sx::sx,
    use_theme,
};

const GAPS: [&str; 4] = ["0", "4", "8", "16"];

const CALL_ORDER: &str = r#"// use_popover first: the style it returns is what the box renders with.
let anchor = use_element();
let popover = use_popover(anchor, opened(), PopoverOptions::new(gap, padding));

// Then the box, taking that style. The other way round, the first render
// styles the box with last render's placement.
let dropdown = use_box()
    .framework_sx(&DROPDOWN_SX)
    .style(popover.style())
    .prepare();

popover.show(opened().then(|| dropdown
    .element(popover.floating())
    .render(HtmlTag::Div, vec![], rsx! { .. })));"#;

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

const DISMISS: &str = r#"let dismiss = use_dismiss(
    anchor,
    *popover.floating(),
    opened(),
    popover.placed(),
    Some(close),
    DismissOptions {
        initial_focus: Some(first_item),
        ..Default::default()
    },
);

// On the trigger, synchronously - that is where the active element
// still is the one the user acted on. For a trigger the application
// may delete while the box is open, name where focus should land
// instead:
onclick: move |_| {
    dismiss.focus_return().remember_active();
    dismiss.focus_return().fallback(list);
    opened.toggle();
}

// On the floating box: Escape and the focus-leaves check.
popover.show(opened().then(|| dropdown
    .element(popover.floating())
    .render(HtmlTag::Div, dismiss.floating_events(), rsx! { .. })));"#;

const CONTEXT_ACROSS_THE_PORTAL: &str = r#"// The box renders at the portal outlet, which sits at the document
// root - so it inherits none of the context around the call site.
// Re-provide what the content needs, inside the content itself.
popover.show(opened().then(|| rsx! {
    MenuProvider { context: menu, {items} }
}));

// A signal a callback outside every scope writes has to outlive the
// scope that made it, and be dropped by hand.
let tick = use_hook(|| Signal::new_in_scope(0u64, ScopeId::ROOT));
use_drop(move || tick.manually_drop());"#;

const NESTED: &str = r#"// A submenu is its own popover, anchored to the row that opened it.
// Geometry needs nothing special: both boxes are `position: fixed` in
// viewport coordinates, and the one that enters the portal later paints
// over the earlier one at the same z-index.
//
// What does need saying: the submenu is not a descendant of the menu in
// the DOM, so focus moving into it reads as focus *leaving* the menu.
// Register it, and keep the guard for as long as the submenu is open.
let _inside = use_hook(move || parent.contain(submenu_box));"#;

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

    format!(
        "let anchor = use_element();\nlet popover = use_popover(anchor, opened(), {options});\n\n\
         popover.show(opened().then(|| rsx! {{\n\
         {}\
         }}));",
        indent(
            "Box {\n    style: popover.style(),\n    onmounted: popover.floating().mount(),\n    \"Popover content\"\n}"
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

    let options = PopoverOptions::new(gap, theme.popover.padding)
        .side(side_of(&side))
        .align(align_of(&align))
        .width(width_of(&width))
        .flip(flip)
        .shift(shift);
    let popover = use_popover(anchor, opened(), options);
    let floating = *popover.floating();

    popover.show(opened().then(|| {
        rsx! {
            Box {
                style: popover.style(),
                onmounted: floating.mount(),
                sx: sx()
                    .background("white")
                    .border("1px solid var(--lsx-grey-3)")
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
                variant: "outlined",
                if opened() { "Close" } else { "Open popover" }
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
                        .doc("How close to a viewport edge the box may come before it flips or shifts."),
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
                ]).without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "There is no "
                    Code { source: "Popover" }
                    " component. A popover is a hook - "
                    Code { source: "use_popover" }
                    " - because every consumer themes its own box: a dropdown, a menu and a "
                    "hover card share when and where, never what it looks like. The hook "
                    "portals the box out to the document root, so it escapes an "
                    Code { source: "overflow: hidden" }
                    " ancestor, and places it in viewport coordinates, flipping and shifting "
                    "to stay on screen."
                }
                Text {
                    "It owns no open state. "
                    Code { source: "show(None)" }
                    " is how a closed popover stops rendering, and deciding when that happens "
                    "is the caller's."
                }
            },
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
                title: "Call order",
                Text {
                    "Call "
                    Code { source: "use_popover" }
                    " before the "
                    Code { source: "use_box()" }
                    " whose "
                    Code { source: "style()" }
                    " takes its placement. Hooks are positional, so the order is not a style "
                    "preference: the box reads the placement the hook produced on this render, "
                    "and the other way round it styles itself with the previous one."
                }
                CodeBlock { source: CALL_ORDER, language: "rust" }
            }

            DocSection {
                title: "Focus after placed, never after mount",
                Text {
                    "A box is mounted one render before it is measured, and until then it is "
                    Code { source: "visibility: hidden" }
                    " - laid out, so it can be measured, but not shown. "
                    Code { source: "focus()" }
                    " on a hidden element returns "
                    Code { source: "Ok(())" }
                    " and moves nothing, with no error to catch. That is why this fails "
                    "silently rather than loudly: wait for "
                    Code { source: "placed()" }
                    "."
                }
                CodeBlock { source: FOCUS_AFTER_PLACED, language: "rust" }
            }

            DocSection {
                title: "Dismissal",
                Text {
                    "Closing is not the placement hook's business, so it lives beside it in "
                    Code { source: "use_dismiss" }
                    ": Escape, focus leaving the box, and handing focus back to whatever "
                    "opened it. It renders nothing - it hands back two attributes to spread "
                    "on the box you drew yourself."
                }
                Text {
                    Code { source: "use_dismiss" }
                    " is internal while "
                    Code { source: "Menu" }
                    " is still finding its contract; the shape below is what it will be when "
                    "it goes public. Until then a downstream dropdown writes the two handlers "
                    "itself."
                }
                CodeBlock { source: DISMISS, language: "rust" }
                Text {
                    "Escape is arbitrated rather than claimed. Every open dismissible layer - "
                    "a popover, and a "
                    Code { source: "Modal" }
                    " too - is on one stack ordered by open time, and a handler acts only if "
                    "its own layer is on top. So a popover open inside a modal closes on "
                    "Escape and the modal under it stays up, without either of them stopping "
                    "the event: stopping propagation on a document-level listener would kill "
                    "every other handler for that press in the whole document."
                }
                Text {
                    "A layer joins that stack only where it can hear Escape from outside its "
                    "own subtree, which today means only where the platform can report a "
                    "document-level key press. A pointer-opened box leaves focus where it was, "
                    "so its own "
                    Code { source: "onkeydown" }
                    " never fires; if it took the top of the stack anyway, the modal that did "
                    "hear the press would decline and Escape would do nothing at all. Where "
                    "there is no document-level listener the box keeps its own handler, the "
                    "modal stays top, and Escape behaves exactly as it did before."
                }
                Text {
                    "A held Escape is one intent. Dismissal ignores an auto-repeat, or a "
                    "press-and-hold walks down the stack closing the menu and then the modal "
                    "behind it. Focus returns to the trigger on a deliberate close - Escape, "
                    "or an item being chosen - and not when focus simply left the box, since a "
                    "click has already put it somewhere the user meant."
                }
            }

            DocSection {
                title: "Context across the portal",
                Text {
                    "The box does not render where it is written. It renders at the portal "
                    "outlet, at the document root, so it inherits no context from around the "
                    "call site - a "
                    Code { source: "use_context" }
                    " inside the content finds the outlet's ancestors, not yours. Re-provide "
                    "what the content needs, inside the content."
                }
                CodeBlock { source: CONTEXT_ACROSS_THE_PORTAL, language: "rust" }
            }

            DocSection {
                title: "Nested popovers",
                Text {
                    "A popover inside a popover needs nothing from the geometry: both boxes "
                    "are placed in viewport coordinates, and the one that enters the portal "
                    "later paints over the earlier one. What it does need is the containment "
                    "check - the inner box is not a descendant of the outer one, so focus "
                    "moving into it looks exactly like focus leaving."
                }
                CodeBlock { source: NESTED, language: "rust" }
            }

            DocSection {
                title: "What it cannot do",
                Text {
                    "Scroll tracking needs a document-level scroll notification, which only "
                    "the web answers today - natively an open popover drifts when the page "
                    "scrolls. Nothing tracks a resize on any backend; "
                    Code { source: "remeasure" }
                    " is the only answer there, on every platform. The focus-leaves check "
                    "needs to wait for the platform's next task to see where focus landed, "
                    "and that wait is a real one only on the web."
                }
            }
        }
    }
}
