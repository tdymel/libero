use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Carousel, CarouselPart, Input, Text},
    sx::sx,
    use_theme,
};

/// Fixed slides, controlled so a control change keeps the reader's slide. Opens on slide 3:
/// at slide 1 every `align` rests the same way.
const FIXED: [&str; 4] = [
    r#"aria_label: "Product photos""#,
    "index: index()",
    "onindexchange: move |next| index.set(next)",
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
                        // A floor, not the size: as `height`, a vertical 300px slide painted only 160px.
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
    let mut index = use_signal(|| 2);

    rsx! {
        DocPage {
            title: "Carousel",
            source: "libero/src/components/data_display/carousel",
            markdown: "/md/carousel.md",
            properties: vec![
                props("Carousel", vec![
                    prop("slides", "Vec<Element>").default("vec![]").doc("The slides, in order."),
                    prop("slide_label", "Callback<usize, String>")
                        .default("{n} of {m}")
                        .doc("Each slide's accessible name."),
                    prop("index", "Option<usize>")
                        .default("None, uncontrolled")
                        .doc("The current slide. Set it and the carousel follows."),
                    prop("onindexchange", "EventHandler<usize>")
                        .default("None")
                        .doc("Fires once a scroll settles, and on every control, key, indicator and autoplay step. Safe to write straight back into `index`. An `index` out of reach is clamped and reported here."),
                    prop("per_view", "f64")
                        .default(theme.carousel.per_view.to_string())
                        .doc("Slides visible at once. A fraction lets the next one peek in."),
                    prop("gap", "Size").default(theme.carousel.gap.as_str()).doc("Between slides."),
                    prop("align", "CarouselAlign")
                        .default(theme.carousel.align.as_str())
                        .doc("Where a snapped slide comes to rest. Shows best with a fractional `per_view`. Above `per_view` 1 it also moves which slides the strip can reach."),
                    prop("orientation", "Orientation")
                        .default("horizontal")
                        .doc("Scroll axis."),
                    prop("height", "ThemeAwareValue")
                        .default("auto")
                        .doc("Required for a vertical carousel, which has nothing else to take its height from. A slide is as long as the carousel makes it, so give its content `height: 100%`."),
                    prop("controls", "bool")
                        .default(theme.carousel.controls.to_string())
                        .doc("Previous and next buttons."),
                    prop("indicators", "bool")
                        .default(theme.carousel.indicators.to_string())
                        .doc("The dot strip, one dot per place the strip can rest. That is fewer than the slides when `per_view` is above 1."),
                    prop("aria_label", "String")
                        .default("localization label")
                        .doc("Names the region. Unset, it falls back to the localization's label and warns."),
                    prop("draggable", "bool")
                        .default("false")
                        .doc("Drag to scroll with a mouse. Touch swipes without it. On Blitz and the WebView a drag stops once the pointer leaves the track."),
                    prop("autoplay", "bool")
                        .default("false")
                        .doc("Advances on a timer, with a pause button first in Tab order. Hover pauses it, and focus stops it until the button is pressed. Under `prefers-reduced-motion: reduce` it opens paused. Without `loop` it stops on the last slide and presses Pause; Play there starts over from the first."),
                    prop("r#loop", "bool")
                        .default("false")
                        .doc("Wraps around at both ends."),
                    prop("autoplay_delay", "u32")
                        .default(theme.carousel.autoplay_delay.to_string())
                        .doc("Milliseconds between advances."),
                    prop("parts", "Parts<CarouselPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .parts("CarouselPart", vec![
                    (CarouselPart::Viewport, "Holds the track and the controls, and clips the strip."),
                    (CarouselPart::Track, "The scrolling strip, the tab stop."),
                    (CarouselPart::Slide, "One slide. The current one also has `data-current=\"true\"`."),
                    (CarouselPart::Controls, "The strip holding previous and next."),
                    (CarouselPart::Control, "The previous or the next button."),
                    (CarouselPart::Indicators, "The group of dots."),
                    (CarouselPart::Indicator, "One dot."),
                    (CarouselPart::Pause, "The autoplay toggle."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters the track, then Previous and Next, then the indicators as one tab stop. With `autoplay`, the pause button comes first.")
                .key(["Left", "Right"], "Moves to the previous or next slide, on the track or the indicators.")
                .key(["Up", "Down"], "The same, when vertical.")
                .key(["Home", "End"], "Goes to the first or last slide.")
                .handles([
                    "On the indicators, focus follows the slide.",
                    "With `autoplay`, focus entering the carousel stops it until the pause button is pressed. Hover only pauses it.",
                ])
                .must(["Set `aria_label` to name the region."]),
            lead: rsx! {
                Text {
                    "A strip of slides that snaps as it scrolls and knows which slide it is "
                    "on. The browser does the scrolling, so touch and momentum feel native. A "
                    "swipe, an arrow key, a button and autoplay all land on the same slide."
                }
            },
            // snippet: let mut index = use_signal(|| 2);
            Demo {
                component: "Carousel",
                children_text: "",
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    // Opens at 1.5: `align` only shows at a fractional `per_view` (todo 153).
                    // `code` prints against the real default of 1, unquoted (`f64`).
                    Control::slider("per_view", ["1", "1.5", "2", "3", "4"])
                        .default("1.5")
                        .code(|_, values| match values.str("per_view").as_str() {
                            "1" => vec![],
                            // `{:?}` keeps the `.0` a whole step needs.
                            value => {
                                let per_view = value.parse::<f64>().unwrap_or(1.0);
                                vec![format!("per_view: {per_view:?}")]
                            }
                        }),
                    Control::slider("gap", ["xs", "sm", "md", "lg", "xl"])
                        .default(theme.carousel.gap.as_str()),
                    Control::toggle("align", ["start", "center", "end"])
                        .labels(["Start", "Center", "End"])
                        .default(theme.carousel.align.as_str()),
                    // A vertical strip has no height of its own, so the demo
                    // sets one - and prints it, because the component needs it.
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .code(
                        |_, values| match values.str("orientation").as_str() {
                            "vertical" => vec![
                                r#"orientation: "vertical""#.to_string(),
                                r#"height: "300px""#.to_string(),
                            ],
                            _ => vec![],
                        },
                    ),
                    Control::switch("controls").default(theme.carousel.controls.to_string()),
                    // On, though the theme's default is off: the dots show `align` moving the
                    // reachable window. `code` prints against the theme.
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
                            index: index(),
                            onindexchange: move |next| index.set(next),
                            per_view: values.str("per_view").parse::<f64>().unwrap_or(1.0),
                            gap: values.str("gap"),
                            align: values.str("align"),
                            orientation: values.str("orientation"),
                            // `Input::None`, not `""`: an empty string is a value, not "no height".
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
        }
    }
}
