use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, prop, props, side_of,
};
use dioxus::prelude::*;
use libero::{
    components::{
        Button, Code, Flex, MaskClick, Paper, Text, Title, TourOptions, TourPart, TourStep,
        TourView, use_tour,
    },
    hooks::{Side, use_element},
    sx::sx,
    use_theme,
};

fn mask_click_of(value: &str) -> MaskClick {
    match value {
        "close" => MaskClick::Close,
        "next" => MaskClick::Next,
        _ => MaskClick::None,
    }
}

/// The page's own source: the custom card prints from `TourDemo`.
const FILE: DemoFile = DemoFile(include_str!("tour.rs"));

/// The hook, its steps and the targets, rebuilt from the controls as `TourDemo` runs them.
fn wrap_hook(values: &DemoValues, _: &str) -> String {
    let side = match side_of(&values.str("side")) {
        Side::Bottom => String::new(),
        side => format!("\n            .side(Side::{side:?})"),
    };
    let custom = values.str("card") == "custom";
    let card = match custom {
        true => format!("{}\n", FILE.section("card")),
        false => String::new(),
    };
    let mut options = String::new();
    let mask_click = mask_click_of(&values.str("mask_click"));
    if mask_click != MaskClick::None {
        options.push_str(&format!("    mask_click: MaskClick::{mask_click:?},\n"));
    }
    if values.str("keyboard") != "true" {
        options.push_str("    keyboard: false,\n");
    }
    if custom {
        options.push_str("    card: Some(card),\n");
    }
    format!(
        "let search = use_element();\n\
         let create = use_element();\n\
         let mut seen = use_signal(|| false);\n\
         let finished = use_callback(move |()| seen.set(true));\n\
         let skipped = use_callback(move |_: usize| seen.set(true));\n\
         {card}\
         let tour = use_tour(TourOptions {{\n    \
             steps: vec![\n        \
                 TourStep::new(\"welcome\")\n            \
                     .title(\"Welcome\")\n            \
                     .description(\"Two stops, under a minute.\"),\n        \
                 TourStep::new(\"search\")\n            \
                     .target(search)\n            \
                     .title(\"Search\")\n            \
                     .description(\"Finds any page by its name.\"){side},\n        \
                 TourStep::new(\"create\")\n            \
                     .target(create)\n            \
                     .title(\"New project\")\n            \
                     .description(\"Starts an empty project.\"){side},\n    \
             ],\n\
         {options}    \
             onfinish: Some(finished),\n    \
             onclose: Some(skipped),\n    \
             ..Default::default()\n\
         }});\n\n\
         rsx! {{\n    \
             Flex {{\n        \
                 direction: \"row\",\n        \
                 gap: \"sm\",\n        \
                 Button {{\n            \
                     variant: \"outlined\",\n            \
                     onmounted: search.mount(),\n            \
                     attributes: search.attributes(),\n            \
                     \"Search\"\n        \
                 }}\n        \
                 Button {{\n            \
                     variant: \"outlined\",\n            \
                     onmounted: create.mount(),\n            \
                     attributes: create.attributes(),\n            \
                     \"New project\"\n        \
                 }}\n        \
                 Button {{ onclick: move |_| tour.start(), \"Take the tour\" }}\n        \
                 Text {{ size: \"sm\", role: \"status\", if seen() {{ \"Seen\" }} else {{ \"Not seen yet\" }} }}\n    \
             }}\n\
         }}"
    )
}

/// The hook needs a scope of its own: `Demo` calls its `render` closure from its own.
#[component]
fn TourDemo(side: String, mask_click: String, keyboard: bool, custom: bool) -> Element {
    let side = side_of(&side);
    let search = use_element();
    let create = use_element();
    let mut seen = use_signal(|| false);
    let finished = use_callback(move |()| seen.set(true));
    let skipped = use_callback(move |_: usize| seen.set(true));
    // demo-code: card start
    let card = use_callback(|view: TourView| {
        let title = view.step.title.clone().unwrap_or_default();
        let description = view.step.description.clone().unwrap_or_default();
        let (skip, next) = (view.clone(), view.clone());
        rsx! {
            Paper {
                shadow: "md",
                sx: sx().padding("md"),
                Text { size: "xs", "{view.progress}" }
                Title { size: "sm", component: "h2", "{title}" }
                Text { "{description}" }
                Flex {
                    direction: "row",
                    gap: "xs",
                    justify: "flex-end",
                    Button { variant: "text", size: "sm", onclick: move |_| skip.close(), "Not now" }
                    Button {
                        size: "sm",
                        onclick: move |_| next.next(),
                        if view.is_last() { "Got it" } else { "Next" }
                    }
                }
            }
        }
    });
    // demo-code: card end
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("welcome")
                .title("Welcome")
                .description("Two stops, under a minute."),
            TourStep::new("search")
                .target(search)
                .title("Search")
                .description("Finds any page by its name.")
                .side(side),
            TourStep::new("create")
                .target(create)
                .title("New project")
                .description("Starts an empty project.")
                .side(side),
        ],
        mask_click: mask_click_of(&mask_click),
        keyboard,
        card: custom.then_some(card),
        onfinish: Some(finished),
        onclose: Some(skipped),
        ..Default::default()
    });

    rsx! {
        Flex {
            direction: "row",
            gap: "sm",
            Button {
                variant: "outlined",
                onmounted: search.mount(),
                attributes: search.attributes(),
                "Search"
            }
            Button {
                variant: "outlined",
                onmounted: create.mount(),
                attributes: create.attributes(),
                "New project"
            }
            Button { onclick: move |_| tour.start(), "Take the tour" }
            Text { size: "sm", role: "status", if seen() { "Seen" } else { "Not seen yet" } }
        }
    }
}

#[component]
pub fn TourPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Tour",
            source: "libero/src/components/overlay/tour",
            markdown: "/md/tour.md",
            properties: vec![
                props("TourOptions", vec![
                    prop("steps", "Vec<TourStep>")
                        .default("required")
                        .doc("The stops, in order. A step without a `target` shows its card in the middle of the screen."),
                    prop("current", "Option<usize>")
                        .doc("Makes the step shown controlled: the buttons, keys and handle calls only call `onchange`."),
                    prop("onchange", "Callback<usize>")
                        .doc("Called with the step a button, key or handle call asks for."),
                    prop("onfinish", "Callback<()>")
                        .doc("Called once when Next is pressed on the last step."),
                    prop("onclose", "Callback<usize>")
                        .doc("Called with the step shown when the tour ends early: Escape, Back, Skip or the close button. Steps going empty while open end it too, with the last step shown."),
                    prop("mask_click", "MaskClick")
                        .default("None")
                        .doc("What a press on the dimmed page does: `None`, `Close` or `Next`. `None`, so a stray tap does not lose the tour."),
                    prop("keyboard", "bool")
                        .default("true")
                        .doc("← and → go to the previous and next step. They follow the text direction."),
                    prop("aria_label", "Option<String>")
                        .doc("Names every step's card, over the step titles. Without either, the localization's `tour.label`, with a warning in a debug build."),
                    prop("card", "Callback<TourView, Element>")
                        .doc("Draws the card's inside in place of the default. The tour still places, names and focuses it; draw your own surface, a `Paper`."),
                    prop("sx", "Input<Sx>")
                        .doc("Styles the card."),
                    prop("parts", "Parts<TourPart>")
                        .doc("Styles for the mask, the highlight and the card's parts."),
                ])
                .without_base_props()
                .parts("TourPart", vec![
                    (TourPart::Mask, "The transparent layer over the page that takes every press."),
                    (TourPart::Highlight, "The hole around the target. Its outer shadow is the dimming."),
                    (TourPart::Positioner, "Places the card beside the target, or in the middle."),
                    (TourPart::Card, "The `dialog`: the default card, or the box around `card`'s."),
                    (TourPart::Header, "The row holding the title and the close button."),
                    (TourPart::Title, "The step's title."),
                    (TourPart::Close, "The close button."),
                    (TourPart::Body, "The step's description or content."),
                    (TourPart::Footer, "The progress text and the buttons."),
                    (TourPart::Progress, "\"2 of 3\"."),
                    (TourPart::Skip, "Ends the tour early. Not on the last step."),
                    (TourPart::Previous, "Not on the first step."),
                    (TourPart::Next, "Reads Done on the last step."),
                ]),
                props("TourStep", vec![
                    prop("new(key)", "String")
                        .doc("Tells the steps apart. The card is drawn afresh when it changes."),
                    prop("target", "ElementHandle")
                        .doc("The element the hole goes around, from `use_element()`, mounted with `onmounted: handle.mount()`."),
                    prop("title", "String")
                        .doc("The card's heading and, unless the tour has an `aria_label`, its name."),
                    prop("description", "String")
                        .doc("The card's text."),
                    prop("content", "Element")
                        .doc("Shown in place of `description`, for rich content."),
                    prop("side", "Side")
                        .default("Bottom")
                        .doc("The card's side of the target. It flips when that side has no room."),
                    prop("align", "Align")
                        .default("Center")
                        .doc("Where the card lines up along that side."),
                    prop("padding", "f64")
                        .default(theme.tour.padding.to_string())
                        .doc("Room between the target and the edge of the hole, in pixels."),
                    prop("radius", "f64")
                        .default(theme.tour.radius.to_string())
                        .doc("The hole's corner radius, in pixels."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Right"], "Goes to the next step, or finishes on the last. Under `dir=\"rtl\"`, ← does. A held key steps once.")
                .key(["Left"], "Goes to the previous step.")
                .key(["Escape"], "Ends the tour early and returns focus to what started it.")
                .key(["Tab", "Shift+Tab"], "Moves the focus within the card. It does not leave while the tour shows.")
                .handles([
                    "Each step's card is a `dialog` with `aria-modal`, named by the step title and described by its text. Focus moves to it on every step.",
                    "While `keyboard` is on, the card names its arrow keys in `aria-keyshortcuts`; few screen readers announce it, so say the keys in the first step's text too.",
                    "The hole has a 2px ring of its own, and an outline in forced colours, so the highlighted element stands out on a dark page too.",
                    "A card taller than the room it has scrolls, so its buttons stay reachable at 400% zoom or on a phone held sideways.",
                    "The highlighted element cannot be pressed, and a press on the dimmed page does nothing unless `mask_click` says so.",
                    "Each step scrolls its target into view; smoothly, unless the user reduces motion. The hole glides to a new step's target, without animation then too, and follows a scroll at once.",
                    "Android's Back button ends the tour, as Escape does, rather than the app.",
                    "A target that never mounts, or renders nothing (`display: none`), shows its step's card in the middle, with a warning in a debug build.",
                    "Steps that go empty while the tour shows end it, as closing does.",
                ])
                .must([
                    "Call `tour.start()` from a button's handler, so focus returns there. Never start a tour on mount.",
                    "Give each step a `title`, or the tour an `aria_label`, so each card has a name of its own.",
                ])
                .example("A first-run tour over Search and New project: \"Take the tour\" moves focus into the Welcome card, → or Next moves the hole to Search, and Escape ends the tour and returns focus to \"Take the tour\".")
                .limits([
                    "The hole follows the target on scroll and window resize, not when the target alone changes size.",
                    "In a native window, the hole drifts under a page scroll the app causes itself; libero hears its own scrolls and the wheel.",
                ]),
            lead: rsx! {
                Text {
                    "A guided tour. "
                    Code { source: "use_tour" }
                    " dims the page around one element per step and explains it in a card "
                    "beside it, with Back, Next and Skip. The steps are data; a step's "
                    Code { source: "target" }
                    " is an element handle from "
                    Code { source: "use_element" }
                    ". The tour is fixed to the viewport, over everything but notifications. "
                    "It stores nothing: "
                    Code { source: "onfinish" }
                    " and "
                    Code { source: "onclose" }
                    " say it was seen, so keep that where your app keeps its settings and offer "
                    "the tour only while it is unseen. The demo keeps it in a signal, forgotten on reload."
                }
            },
            Demo {
                component: "TourOptions",
                children_text: "",
                controls: vec![
                    Control::side(["top", "end", "bottom", "start"]),
                    Control::toggle("mask_click", ["none", "close", "next"])
                        .labels(["None", "Close", "Next"]),
                    Control::toggle("card", ["default", "custom"]).labels(["Default", "Custom"]),
                    Control::switch("keyboard").default("true"),
                ],
                render: move |values: DemoValues| rsx! {
                    TourDemo {
                        side: values.str("side"),
                        mask_click: values.str("mask_click"),
                        keyboard: values.str("keyboard") == "true",
                        custom: values.str("card") == "custom",
                    }
                },
                wrap: Wrap(wrap_hook),
            }
        }
    }
}
