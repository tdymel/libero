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
    if custom {
        options.push_str("    card: Some(card),\n");
    }
    format!(
        "let search = use_element();\n\
         let mut searches = use_signal(|| 0);\n\
         {card}\
         let tour = use_tour(TourOptions {{\n    \
             steps: vec![\n        \
                 TourStep::new(\"welcome\")\n            \
                     .title(\"Welcome\")\n            \
                     .description(\"Two stops, under a minute.\"),\n        \
                 TourStep::new(\"search\")\n            \
                     .target(search)\n            \
                     .interactive(true)\n            \
                     .title(\"Search\")\n            \
                     .description(\"Finds any page by its name. Try it: press it, or Tab to it.\"){side},\n        \
                 TourStep::new(\"create\")\n            \
                     .target_selector(\"#tour-demo-create\")\n            \
                     .title(\"New project\")\n            \
                     .description(\"Starts an empty project.\"){side},\n    \
             ],\n\
         {options}    \
             storage_key: Some(\"docs-tour-seen\".into()),\n    \
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
                     onclick: move |_| searches += 1,\n            \
                     \"Search\"\n        \
                 }}\n        \
                 // Found by its id: no handle to pass down.\n        \
                 Button {{ id: \"tour-demo-create\", variant: \"outlined\", \"New project\" }}\n        \
                 Button {{ onclick: move |_| tour.start(), \"Take the tour\" }}\n        \
                 Button {{ variant: \"text\", onclick: move |_| tour.forget(), \"Forget seen\" }}\n        \
                 Text {{\n            \
                     size: \"sm\",\n            \
                     role: \"status\",\n            \
                     if tour.seen() {{ \"Seen\" }} else {{ \"Not seen yet\" }}\n            \
                     \", searches: {{searches}}\"\n        \
                 }}\n    \
             }}\n\
         }}"
    )
}

/// The hook needs a scope of its own: `Demo` calls its `render` closure from its own.
#[component]
fn TourDemo(side: String, mask_click: String, custom: bool) -> Element {
    let side = side_of(&side);
    let search = use_element();
    let mut searches = use_signal(|| 0);
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
                .interactive(true)
                .title("Search")
                .description("Finds any page by its name. Try it: press it, or Tab to it.")
                .side(side),
            TourStep::new("create")
                .target_selector("#tour-demo-create")
                .title("New project")
                .description("Starts an empty project.")
                .side(side),
        ],
        mask_click: mask_click_of(&mask_click),
        card: custom.then_some(card),
        storage_key: Some("docs-tour-seen".into()),
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
                onclick: move |_| searches += 1,
                "Search"
            }
            // Found by its id: no handle to pass down.
            Button { id: "tour-demo-create", variant: "outlined", "New project" }
            Button { onclick: move |_| tour.start(), "Take the tour" }
            Button { variant: "text", onclick: move |_| tour.forget(), "Forget seen" }
            Text {
                size: "sm",
                role: "status",
                if tour.seen() { "Seen" } else { "Not seen yet" }
                ", searches: {searches}"
            }
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
                    prop("aria_label", "Option<String>")
                        .doc("Names every step's card, over the step titles. Without either, the localization's `tour.label`, with a warning in a debug build."),
                    prop("card", "Callback<TourView, Element>")
                        .doc("Draws the card's inside in place of the default. The tour still places, names and focuses it; draw your own surface, a `Paper`."),
                    prop("sx", "Input<Sx>")
                        .doc("Styles the card."),
                    prop("parts", "Parts<TourPart>")
                        .doc("Styles for the mask, the highlight and the card's parts."),
                    prop("storage_key", "Option<String>")
                        .doc("Remembers in local storage that the tour was finished or closed early, for `tour.seen()`; `tour.forget()` drops it. `start()` still starts: offer the tour while `!tour.seen()`. Read at mount."),
                ])
                .without_base_props()
                .parts("TourPart", vec![
                    (TourPart::Mask, "The transparent layer over the page that takes every press. Four strips around the hole on an `interactive` step."),
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
                    prop("target_selector", "String")
                        .doc("A CSS selector for the target, as `\"#search\"`, looked up in the document when the step shows: for a target in another component. `target` wins."),
                    prop("interactive", "bool")
                        .default("false")
                        .doc("Lets presses through the hole to the target, and puts the target in the Tab order beside the card."),
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
                .key(["Right"], "Goes to the next step. On the last step it does nothing: only Done finishes. Under `dir=\"rtl\"`, ← does. A held key steps once.")
                .key(["Left"], "Goes to the previous step.")
                .key(["Escape"], "Ends the tour early and returns focus to what started it.")
                .key(["Tab", "Shift+Tab"], "Moves the focus within the card. It does not leave while the tour shows, but for an `interactive` step's target: Tab past the card's last control reaches it, and Tab from it returns to the card. On the target the arrows are its own; Escape still ends the tour.")
                .handles([
                    "Each step's card is a `dialog` with `aria-modal`, named by the step title and described by its text and its progress (\"2 of 3\"), also with a custom `card`, which the tour describes by a hidden text of its own. Focus moves to it on every step.",
                    "The card names its arrow keys in `aria-keyshortcuts`; few screen readers announce it, so say the keys in the first step's text too.",
                    "The hole has a 2px ring of its own, and an outline in forced colours, so the highlighted element stands out on a dark page too.",
                    "A card taller than the room it has scrolls, so its buttons stay reachable at 400% zoom or on a phone held sideways.",
                    "The highlighted element cannot be pressed unless its step is `interactive`, and a press on the dimmed page does nothing unless `mask_click` says so.",
                    "An `interactive` step's card has `aria-modal=\"false\"`: the target outside it is reachable.",
                    "Each step scrolls its target into view; smoothly, unless the user reduces motion. The hole glides to a new step's target, without animation then too, and follows a scroll at once.",
                    "Android's Back button ends the tour, as Escape does, rather than the app.",
                    "A `target_selector` that matches nothing when its step shows centres the card, with a warning in a debug build.",
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
                    ", or a selector for one in another component. The tour is fixed to the viewport, "
                    "over everything but notifications. With a "
                    Code { source: "storage_key" }
                    " it remembers being finished or skipped across reloads: offer it while "
                    Code { source: "tour.seen()" }
                    " is false. The demo's Search step is interactive: press it while the tour shows."
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
                ],
                render: move |values: DemoValues| rsx! {
                    TourDemo {
                        side: values.str("side"),
                        mask_click: values.str("mask_click"),
                        custom: values.str("card") == "custom",
                    }
                },
                wrap: Wrap(wrap_hook),
            }
        }
    }
}
