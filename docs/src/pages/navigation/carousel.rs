use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Carousel, Code, CodeBlock, Input, Kbd, List, ListItem, Text},
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
                .min_height("160px")
                .height("100%")
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
                        // A slide has whatever length the carousel gives it -
                        // a vertical one is 300px of the 300px track - so the
                        // fixed height is a floor, not the size. It used to be
                        // `height`, and a vertical slide then painted 160 of
                        // its 300px and left the rest blank.
                        .min_height("160px")
                        .height("100%")
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
                        .default(theme.carousel.controls.to_string())
                        .doc("Prev/next buttons."),
                    prop("indicators", "bool")
                        .default(theme.carousel.indicators.to_string())
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
                        .default(theme.carousel.autoplay_delay.to_string())
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
                    // Opens at 3, not at the component's own default of 1.
                    // At 1 a slide fills the viewport, so all three `align`
                    // values coincide and that control looks dead - and the
                    // dot strip is one dot per slide, so `indicators` has
                    // nothing to say either. `code` prints against the real
                    // default, and unquoted: `per_view` is an `f64`.
                    Control::slider("per_view", ["1", "1.5", "2", "3", "4"])
                        .default("3")
                        .code(|_, values| match values.str("per_view").as_str() {
                            "1" => vec![],
                            value => vec![format!("per_view: {value}")],
                        }),
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
                    Control::switch("controls").default(theme.carousel.controls.to_string()),
                    // On, though the theme's default is off: above `per_view`
                    // 1 the dot strip is the only thing that shows `align`
                    // moving the reachable window - four dots reading 1-4,
                    // 2-5 or 3-6 - while the strip itself is clamped at rest
                    // and cannot move. `code` prints against the theme.
                    Control::switch("indicators").default("true").code(|_, values| {
                        match values.str("indicators").as_str() {
                            "true" => vec!["indicators: true".to_string()],
                            _ => vec![],
                        }
                    }),
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
                            // `Input::None`, not `""`: an empty string is a
                            // value, and it resolves to an empty declaration
                            // rather than to "no height".
                            height: match vertical {
                                true => Input::Value("300px".into()),
                                false => Input::None,
                            },
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
                title: "Accessibility",
                List {
                    ListItem {
                        "Set "
                        Code { source: "aria_label" }
                        ": it names the region."
                    }
                    ListItem {
                        "The track is a tab stop. "
                        Kbd { "←" } " " Kbd { "→" } ", or " Kbd { "↑" } " " Kbd { "↓" }
                        " when vertical: previous and next. "
                        Kbd { "Home" } " " Kbd { "End" } ": first and last."
                    }
                    ListItem {
                        "The indicators are one tab stop with the same keys, and focus follows "
                        "the slide."
                    }
                    ListItem {
                        "Announces \"Slide 3 of 7\" once a move settles. Autoplay stays silent "
                        "until it is paused - by its button, by hover or by focus."
                    }
                }
            }
        }
    }
}
