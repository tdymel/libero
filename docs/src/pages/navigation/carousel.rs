use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Carousel, Code, CodeBlock, Text},
    sx::sx,
    use_theme,
};

/// The slides are fixed markup, not a control: a carousel has nothing to show
/// without them, and there is nothing here for a reader to vary.
const FIXED: [&str; 2] = [
    r#"aria_label: "Product photos""#,
    r#"slides: (1..=6)
    .map(|n| rsx! {
        Box {
            sx: sx()
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .height("160px")
                .background(format!("primary.{n}"))
                .color(format!("primary-contrast.{n}")),
            Text { "Slide {n}" }
        }
    })
    .collect()"#,
];

const CONTROLLED: &str = r#"let mut slide = use_signal(|| 0usize);

rsx! {
    Carousel {
        aria_label: "Product photos",
        index: slide(),
        onindexchange: move |index| slide.set(index),
        slides: photos.iter().map(|photo| rsx! {
            Image { src: "{photo.url}", alt: "{photo.alt}", fit: "cover" }
        }).collect(),
    }
    Text { "Showing {slide() + 1} of {photos.len()}" }
}"#;

fn demo_slides() -> Vec<Element> {
    (1..=6)
        .map(|n| {
            rsx! {
                Box {
                    sx: sx()
                        .display("flex")
                        .align_items("center")
                        .justify_content("center")
                        .height("160px")
                        .background(format!("primary.{n}"))
                        .color(format!("primary-contrast.{n}")),
                    Text { "Slide {n}" }
                }
            }
        })
        .collect()
}

#[component]
pub fn CarouselPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Carousel",
            source: "libero/src/components/navigation/carousel.rs",
            markdown: "/md/carousel.md",
            properties: vec![
                props("Carousel", vec![
                    prop("slides", "Vec<Element>").doc("The slides, in order."),
                    prop("slide_label", "Callback<usize, String>")
                        .default("{n} of {m}")
                        .doc("Each slide group's accessible name."),
                    prop("index", "usize")
                        .default("uncontrolled")
                        .doc("The current slide. Set it and the carousel follows."),
                    prop("onindexchange", "EventHandler<usize>")
                        .doc("Fired once a scroll settles, and on every control, key, indicator and autoplay tick."),
                    prop("per_view", "f64")
                        .default("1")
                        .doc("Slides visible at once. Fractional peeks the next one."),
                    prop("gap", "Size").default(theme.carousel.gap.as_str()).doc("Between slides."),
                    prop("align", "CarouselAlign")
                        .default(theme.carousel.align.as_str())
                        .doc("Where a snapped slide comes to rest - start, center or end. Above per_view 1 it also moves which indices are reachable, because the browser clamps the scroll at both ends."),
                    prop("orientation", "Orientation")
                        .default("horizontal")
                        .doc("Scroll axis."),
                    prop("height", "ThemeAwareValue")
                        .default("auto")
                        .doc("Required for a vertical carousel, which has nothing to take its height from."),
                    prop("controls", "bool")
                        .default(&theme.carousel.controls.to_string())
                        .doc("Prev/next buttons."),
                    prop("indicators", "bool")
                        .default(&theme.carousel.indicators.to_string())
                        .doc("The dot strip - one per scroll position, which is fewer than the slides when per_view is above 1."),
                    prop("aria_label", "String").doc("Names the region. Leaving it unset falls back to the theme label and warns."),
                    prop("draggable", "bool")
                        .default("false")
                        .doc("Mouse drag-to-scroll. Touch already swipes natively."),
                    prop("autoplay", "bool")
                        .default("false")
                        .doc("Advances on a timer, with a pause control and pause on hover and focus."),
                    prop("r#loop", "bool")
                        .default("false")
                        .doc("Wraps at both ends by cloning slides onto each end and jumping back across the seam once the scroll settles. The clones are aria-hidden."),
                    prop("autoplay_delay", "u32")
                        .default(&theme.carousel.autoplay_delay.to_string())
                        .doc("Milliseconds between advances."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A scroll-snap strip that knows which slide it is on. The scrolling is the "
                    "browser's - so touch, momentum and rubber-banding are the platform's, not "
                    "ours - and the index is derived from the scroll position, which is what "
                    "makes a swipe, an arrow key, a control click and an autoplay tick all end "
                    "up in the same place. "
                    Code { source: "per_view" }
                    " sets how many slides are visible at once, fractionally if you want the "
                    "next one peeking."
                }
            },
            Demo {
                component: "Carousel",
                children_text: "",
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    Control::slider("per_view", ["1", "2", "3", "4"]),
                    Control::slider("gap", ["xs", "sm", "md", "lg", "xl"])
                        .default(theme.carousel.gap.as_str()),
                    Control::toggle("align", ["start", "center", "end"])
                        .default(theme.carousel.align.as_str()),
                    // A vertical strip has no height of its own, so the demo
                    // sets one - and prints it, because the component needs it.
                    Control::toggle("orientation", ["horizontal", "vertical"]).code(
                        |_, values| match values.str("orientation").as_str() {
                            "vertical" => vec![
                                r#"orientation: "vertical""#.to_string(),
                                r#"height: "300px""#.to_string(),
                            ],
                            _ => vec![],
                        },
                    ),
                    Control::switch("controls").default(&theme.carousel.controls.to_string()),
                    Control::switch("indicators").default(&theme.carousel.indicators.to_string()),
                    Control::switch("draggable"),
                    Control::switch("autoplay"),
                    Control::switch("loop").code(|_, values| {
                        match values.str("loop").as_str() {
                            "true" => vec!["r#loop: true".to_string()],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| {
                    let vertical = values.str("orientation") == "vertical";

                    rsx! {
                        Carousel {
                            aria_label: "Product photos",
                            per_view: values.str("per_view").parse::<f64>().unwrap_or(1.0),
                            gap: values.str("gap"),
                            align: values.str("align"),
                            orientation: values.str("orientation"),
                            height: if vertical { "300px" } else { "" },
                            controls: values.str("controls") == "true",
                            indicators: values.str("indicators") == "true",
                            draggable: values.str("draggable") == "true",
                            autoplay: values.str("autoplay") == "true",
                            r#loop: values.str("loop") == "true",
                            slides: demo_slides(),
                        }
                    }
                },
            }

            DocSection {
                title: "Controlled slide",
                Text {
                    "Pass "
                    Code { source: "index" }
                    " and the carousel follows it; read "
                    Code { source: "onindexchange" }
                    " to follow the carousel. It fires once a scroll settles rather than on "
                    "every frame, so it is safe to write straight back into the signal that "
                    "drives it."
                }
                Text {
                    "While you are driving index, it can also fire with an index you did not "
                    "ask for. If the one you passed is outside the reachable window - slide 0 "
                    "on a centred three-up strip, or an index left over after the slides got "
                    "shorter - the carousel clamps it and tells you, rather than the two of you "
                    "disagreeing forever with your code pushing the same unreachable value back "
                    "on every render. An uncontrolled carousel stays quiet."
                }
                CodeBlock { source: CONTROLLED, language: "rust" }
            }

            DocSection {
                title: "Looping",
                Text {
                    Code { source: "r#loop" }
                    " wraps at both ends. A scroll container has no wrap of its own, so it is "
                    "done the way it has always been done: enough slides are cloned onto each "
                    "end for the strip to scroll a full viewport past either edge, and once the "
                    "scroll settles on a clone the carousel jumps to the real slide showing the "
                    "same thing, with smooth scrolling switched off for that one jump so nothing "
                    "visibly rewinds."
                }
                Text {
                    "The clones carry "
                    Code { source: "aria-hidden" }
                    " and no name, so the content is not announced twice, and the controls stop "
                    "disabling because there is no longer an end to be at. The indicator strip "
                    "goes back to one dot per slide."
                }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    Code { source: "aria_label" }
                    " is not optional: the root is a "
                    Code { source: "role=\"region\"" }
                    " with "
                    Code { source: "aria-roledescription=\"carousel\"" }
                    ", and an unnamed region is a landmark a screen reader user cannot tell "
                    "apart from any other. Leaving it unset warns in the console."
                }
                Text {
                    "The arrow keys belong to the track rather than to the carousel as a whole, "
                    "so a slide may hold a text field and keep its own keys - the caret still "
                    "moves, Home and End still work inside it, and the carousel does not advance "
                    "underneath."
                }
                Text {
                    "The track itself is the tab stop, because it is the scrollable region and "
                    "that is how a keyboard user scrolls one. Arrow keys move a slide at a time, "
                    "Home and End jump to the ends, and the indicator strip is a single tab stop "
                    "whose arrows move both the focus and the slide. A control at either end goes "
                    Code { source: "aria-disabled" }
                    " but keeps its place in the tab order, so focus is never dropped."
                }
                Text {
                    "A live region announces the settled slide - \"Slide 3 of 7\" - and not every "
                    "scroll frame. Offscreen slides are deliberately not hidden: in a real scroll "
                    "container they are reachable, and hiding them would remove content. A slide "
                    "may hold focusable content, and tabbing into an offscreen one scrolls it into "
                    "view, which settles the index the same way a swipe does - the announcement "
                    "follows rather than desyncing, because the index comes from the scroll "
                    "position rather than from intent."
                }
                Text {
                    "With "
                    Code { source: "autoplay" }
                    ", the pause control is not optional and it is a real button, not a hover "
                    "affordance: hovering does pause, and so does focus landing anywhere inside, "
                    "but neither helps a touch user. While it is rotating unattended the live "
                    "region is "
                    Code { source: "aria-live=\"off\"" }
                    " - an unprompted change is not worth interrupting a screen reader for - and "
                    "it becomes polite again the moment it stops."
                }
                Text {
                    "Dragging with a mouse is opt-in and does not exist for touch, on purpose: "
                    "the swipe is already the platform's own scroll, and making the track a drag "
                    "handle would need "
                    Code { source: "touch-action: none" }
                    ", which would take that away."
                }
            }
        }
    }
}
