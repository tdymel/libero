use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Carousel, Code, Input, Kbd, List, ListItem, Text},
    sx::sx,
    use_theme,
};

/// The slides are fixed markup, not a control: a carousel has nothing to show
/// without them, and there is nothing here for a reader to vary. It opens on
/// slide 3 because at slide 1 the strip is at its very start, where every
/// `align` rests the same way; on a middle slide switching it moves the slide
/// to the left, the middle or the right at once.
const FIXED: [&str; 3] = [
    r#"aria_label: "Product photos""#,
    "index: 2",
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
                        .doc("Where a snapped slide comes to rest - start, center or end. Visible with a fractional per_view; at a whole one the alignments can share their resting offsets. Above per_view 1 it also moves which indices are reachable, because the browser clamps the scroll at both ends."),
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
                    // Opens at 1.5, not at the component's own default of 1.
                    // `align` only moves a slide where the three alignments
                    // snap to different offsets, and with equal slides that
                    // takes a fractional `per_view`: at 1 a slide fills the
                    // viewport, and at 3 the centred and end-aligned snaps
                    // land on the same whole-slide offsets as the start ones,
                    // so the control looked dead (todo 153). At 1.5 the slide
                    // rests at the left, in the middle or at the right, with
                    // the neighbours peeking round it. `code` prints against
                    // the real default, and unquoted: `per_view` is an `f64`.
                    Control::slider("per_view", ["1", "1.5", "2", "3", "4"])
                        .default("1.5")
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
                    // 1 the dot strip shows `align` moving the reachable
                    // window - end-aligned at 1.5 has no dot for slide 1, and
                    // at 3 the dots read 1-4, 2-5 or 3-6. `code` prints
                    // against the theme.
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
                            index: 2,
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
