use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Carousel, CarouselPart, Input, Orientation, Text},
    sx::ThemeAwareValue,
    theme::{Responsive, Size, responsive},
    use_theme,
};

fn other(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Horizontal => Orientation::Vertical,
        Orientation::Vertical => Orientation::Horizontal,
    }
}

/// The height a vertical strip needs, per range.
fn height_in(orientation: Orientation) -> ThemeAwareValue {
    match orientation {
        Orientation::Vertical => "300px".into(),
        Orientation::Horizontal => "auto".into(),
    }
}

/// The other orientation below `md`, the picked one from it, while `responsive` is on; a
/// vertical range takes a height.
fn orientation_of(
    values: &DemoValues,
) -> (Responsive<Orientation>, Input<Responsive<ThemeAwareValue>>) {
    let picked = Orientation::from(values.str("orientation").as_str());
    match (values.str("responsive") == "true", picked) {
        (true, picked) => (
            responsive(other(picked)).md(picked),
            responsive(height_in(other(picked)))
                .md(height_in(picked))
                .into(),
        ),
        (false, Orientation::Vertical) => (picked.into(), "300px".into()),
        (false, Orientation::Horizontal) => (picked.into(), Input::None),
    }
}

fn orientation_code(_: &Control, values: &DemoValues) -> Vec<String> {
    let picked = Orientation::from(values.str("orientation").as_str());
    let height = |orientation| match orientation {
        Orientation::Vertical => "\"300px\"",
        Orientation::Horizontal => "\"auto\"",
    };
    match (values.str("responsive") == "true", picked) {
        (true, picked) => vec![
            format!(
                "orientation: responsive(Orientation::{:?}).md(Orientation::{picked:?})",
                other(picked)
            ),
            format!(
                "height: responsive(ThemeAwareValue::from({})).md({}.into())",
                height(other(picked)),
                height(picked)
            ),
        ],
        (false, Orientation::Vertical) => vec![
            r#"orientation: "vertical""#.to_string(),
            r#"height: "300px""#.to_string(),
        ],
        (false, Orientation::Horizontal) => vec![],
    }
}

/// A gap step as a value: `"0"` is CSS, the rest are sizes.
fn gap_value(gap: &str) -> ThemeAwareValue {
    match gap {
        "0" => "0".into(),
        size => Size::from(size).into(),
    }
}

/// One slide a view below `md`, the pick from it, while `responsive` is on.
fn per_view_of(values: &DemoValues) -> Responsive<f64> {
    let per_view = values.str("per_view").parse::<f64>().unwrap_or(1.0);
    match values.str("responsive") == "true" {
        true => responsive(1.0).md(per_view),
        false => per_view.into(),
    }
}

/// One slide a step below `md`, the pick from it, while `responsive` is on.
fn slides_to_scroll_of(values: &DemoValues) -> Responsive<usize> {
    let step = values.str("slides_to_scroll").parse::<usize>().unwrap_or(1);
    match values.str("responsive") == "true" {
        true => responsive(1).md(step),
        false => step.into(),
    }
}

/// `xs` below `md`, the chosen gap from it, while `responsive` is on.
fn gap_of(values: &DemoValues) -> Input<Responsive<ThemeAwareValue>> {
    let gap = gap_value(&values.str("gap"));
    match values.str("responsive") == "true" {
        true => responsive(ThemeAwareValue::from(Size::Xs)).md(gap).into(),
        false => gap.into(),
    }
}

fn gap_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let gap = values.str("gap");
    let value = match gap.as_str() {
        "0" => r#""0".into()"#.to_string(),
        size => format!("Size::{:?}.into()", Size::from(size)),
    };
    match values.str("responsive") == "true" {
        true => vec![format!(
            "gap: responsive(ThemeAwareValue::from(Size::Xs)).md({value})"
        )],
        false if gap == control.default => vec![],
        false => vec![format!("gap: {gap:?}")],
    }
}

mod demo;
use demo::demo_slides;

/// Fixed slides, controlled so a control change keeps the reader's slide. Opens on slide 3:
/// at slide 1 every `align` rests the same way.
const FIXED: [&str; 4] = [
    r#"aria_label: "Product photos""#,
    "index: index()",
    "onindexchange: move |next| index.set(next)",
    "slides: demo_slides()",
];

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
                    prop("per_view", "Responsive<f64>")
                        .default(theme.carousel.per_view.to_string())
                        .doc("Slides visible at once. A fraction lets the next one peek in. The strip runs out of scroll before the slides, so six slides three-up stop at index 3. Or one per breakpoint, `per_view: responsive(1.0).md(3.0)`: the slide widths follow the window at once, the status, dots and last stop of a server-rendered page after mount. Breakpoints follow the window, not the parent."),
                    prop("gap", "Responsive<ThemeAwareValue>").default(theme.carousel.gap.as_str()).doc("Between slides, or any CSS, e.g. `gap: \"0\"`. Or one per breakpoint, `gap: responsive(ThemeAwareValue::from(Size::Xs)).md(Size::Lg.into())`. Breakpoints follow the window, not the parent."),
                    prop("align", "CarouselAlign")
                        .default(theme.carousel.align.as_str())
                        .doc("Where a snapped slide comes to rest, `start`, `center` or `end`. Shows best with a fractional `per_view`. Above `per_view` 1 it also moves which slides the strip can reach."),
                    prop("orientation", "Responsive<Orientation>")
                        .default("horizontal")
                        .doc("Scroll axis. Or one per breakpoint, `orientation: responsive(Orientation::Horizontal).md(Orientation::Vertical)`, with `height` from the same breakpoint. The controls, keys and drag follow the window after mount: a server-rendered page draws the base orientation until then."),
                    prop("height", "Responsive<ThemeAwareValue>")
                        .default("auto")
                        .doc("Required for a vertical carousel, which has nothing else to take its height from. A slide is as long as the carousel makes it, so give its content `height: 100%`. Or one per breakpoint, `height: responsive(ThemeAwareValue::from(\"240px\")).md(\"400px\".into())`."),
                    prop("controls", "bool")
                        .default(theme.carousel.controls.to_string())
                        .doc("Previous and next buttons. Hidden while every slide fits."),
                    prop("indicators", "bool")
                        .default(theme.carousel.indicators.to_string())
                        .doc("The dot strip, one dot per place the strip can rest. That is fewer than the slides when `per_view` is above 1."),
                    prop("aria_label", "String")
                        .default("localization label")
                        .doc("Names the region. Unset, it falls back to the localization's label and warns."),
                    prop("draggable", "bool")
                        .default("false")
                        .doc("Drag to scroll with a mouse. Touch swipes without it. On Blitz and the WebView a drag stops once the pointer leaves the track. With `controls` and `indicators` both off a drag is the only pointer way to move (WCAG 2.5.7), and a debug build warns."),
                    prop("drag_free", "bool")
                        .default("false")
                        .doc("A drag or swipe rests where it ends instead of snapping to a slide; pair it with `draggable` for a mouse. Arrow keys, controls and dots still land on a slide. Ignored with `loop`."),
                    prop("slides_to_scroll", "Responsive<usize>")
                        .default("1")
                        .doc("Slides one control, arrow key or autoplay step moves, stopping at the last place the strip can rest. Zero counts as 1; a looping strip caps it at the slides in view. The dots stay one per place. Or one per breakpoint, `slides_to_scroll: responsive(1).md(3)`; a server-rendered page steps by the base until it mounts."),
                    prop("autoplay", "bool")
                        .default("false")
                        .doc("Advances on a timer, with a pause button first in Tab order. Hover pauses it, and focus stops it until the button is pressed. Under `prefers-reduced-motion: reduce` it opens paused. Without `loop` it stops on the last slide and presses Pause; Play there starts over from the first."),
                    prop("r#loop", "bool")
                        .default("false")
                        .doc("Wraps around at both ends through cloned slides, which repeat a slide's DOM: give interactive slide content no `id` or form `name`. The strip can show the end and the start at once, and the status then reads \"Slides 6 to 2 of 6, wrapping around\"."),
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
                .key(["Left", "Right"], "Moves to the previous or next slide, on the track or the indicators, when horizontal.")
                .key(["Up", "Down"], "The same, when vertical. The other axis scrolls the page.")
                .key(["Home", "End"], "Goes to the first or last slide.")
                .handles([
                    "The carousel is a region named by `aria_label`, and each slide is a group a screen reader calls a slide. Slides out of view leave the Tab and reading order.",
                    "After each move a hidden status says which slide shows, out of how many (\"Slides 1–3 of 6\", counting a slide at least half in view). It stays quiet while autoplay rotates (WCAG 2.2.2).",
                    "Previous and Next are buttons. The indicators are one Tab stop with the current one marked, and focus follows the slide on them.",
                    "A carousel whose slides all fit, without `loop`, drops the status, the dots and the track's Tab stop: none of them has a place to go.",
                    "With `autoplay`, focus inside the carousel stops it until the pause button is pressed. Hover only pauses it.",
                ])
                .must(["Set `aria_label` to name the region. Without it the theme's generic name is used, and a debug build warns."])
                .example("A product gallery: `Carousel { aria_label: \"Product photos\", .. }`. A screen reader lists a \"Product photos\" region, and after Next it hears which photo shows, out of how many."),
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
                wide_preview: true,
                children_text: "",
                fixed: FIXED.map(str::to_string).to_vec(),
                file: DemoFile(include_str!("carousel/demo.rs")),
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
                                match values.str("responsive") == "true" {
                                    true => vec![format!("per_view: responsive(1.0).md({per_view:?})")],
                                    false => vec![format!("per_view: {per_view:?}")],
                                }
                            }
                        }),
                    Control::slider("gap", ["0", "xs", "sm", "md", "lg", "xl"])
                        .default(theme.carousel.gap.as_str())
                        .code(gap_code),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    Control::toggle("align", ["start", "center", "end"])
                        .labels(["Start", "Center", "End"])
                        .default(theme.carousel.align.as_str()),
                    // A vertical strip has no height of its own, so the demo
                    // sets one - and prints it, because the component needs it.
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .code(orientation_code),
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
                    Control::switch("drag_free"),
                    Control::slider("slides_to_scroll", ["1", "2", "3"]).code(|_, values| {
                        match values.str("slides_to_scroll").as_str() {
                            "1" => vec![],
                            value if values.str("responsive") == "true" => {
                                vec![format!("slides_to_scroll: responsive(1).md({value})")]
                            }
                            value => vec![format!("slides_to_scroll: {value}")],
                        }
                    }),
                    Control::switch("autoplay"),
                    Control::switch("loop").code(|_, values| {
                        match values.str("loop").as_str() {
                            "true" => vec!["r#loop: true".to_string()],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| {
                    let (orientation, height) = orientation_of(&values);

                    rsx! {
                        Carousel {
                            aria_label: "Product photos",
                            index: index(),
                            onindexchange: move |next| index.set(next),
                            per_view: per_view_of(&values),
                            gap: gap_of(&values),
                            align: values.str("align"),
                            orientation,
                            // `None`, not `""`: an empty string is a value, not "no height".
                            height,
                            controls: values.str("controls") == "true",
                            indicators: values.str("indicators") == "true",
                            draggable: values.str("draggable") == "true",
                            drag_free: values.str("drag_free") == "true",
                            slides_to_scroll: slides_to_scroll_of(&values),
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
