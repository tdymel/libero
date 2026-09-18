use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, HtmlTag, Input, Orientation, States, Variables,
        common::{PauseIcon, PlayIcon, base_props, variables},
        layout::use_box,
    },
    hooks::{use_localization, use_theme},
    sx::{REDUCED_MOTION, StaticSx, sx},
    theme::{
        MARQUEE_ANIMATION, MARQUEE_DURATION, MARQUEE_FADE_SIZE, MARQUEE_GAP, MARQUEE_MIN_REPEAT,
        MARQUEE_REPEAT, MARQUEE_SHIFT, PAPER_BACKGROUND, Size, SizeCss,
    },
};

const TRACK: &str = "& > [data-slot='track']";
const COPIES: &str = "& > [data-slot='track'] > [data-slot='group']";
const PAUSE: &str = "& > [data-slot='pause']";

/// One copy plus one gap, as a share of the whole track: the track is `repeat`
/// copies and `repeat - 1` gaps long, so adding the one missing gap and
/// dividing by `repeat` lands exactly on the next copy.
fn shift(axis: &str) -> String {
    format!(
        "translate{axis}(calc((-100% - {}) / {}))",
        MARQUEE_GAP.value(),
        MARQUEE_REPEAT.value()
    )
}

fn fade(side: &str) -> String {
    format!(
        "linear-gradient(to {side}, {}, transparent)",
        PAPER_BACKGROUND.value()
    )
}

static MARQUEE_BASE_SX: StaticSx = StaticSx::new(|| {
    let offset = SizeCss::SPACING.value(Size::Xs);

    // Every guard for reduced motion is nested in the selector it undoes: a
    // bare `@media` rule would lose to the per-instance selector on
    // specificity whatever its source order. Under it the same DOM becomes a
    // strip the reader scrolls themselves - one copy, no fade, no toggle.
    let base = sx()
        .position("relative")
        .selector(
            TRACK,
            sx().display("flex")
                .gap(MARQUEE_GAP.value())
                .animation(format!(
                    "{MARQUEE_ANIMATION} {} linear infinite",
                    MARQUEE_DURATION.overridable()
                ))
                .media(REDUCED_MOTION, sx().animation("none")),
        )
        .selector(
            COPIES,
            sx().display("flex")
                .flex_shrink("0")
                .gap(MARQUEE_GAP.value()),
        )
        .selector(
            format!("{COPIES}:not(:first-child)"),
            sx().media(REDUCED_MOTION, sx().display("none")),
        )
        .selector(
            PAUSE,
            sx().position("absolute")
                .z_index("2")
                .right(offset.clone())
                .rtl(sx().right("auto").left(offset.clone()))
                .media(REDUCED_MOTION, sx().display("none")),
        );

    // `clip`, not `hidden`, and only along the axis: the other one stays
    // visible, so the toggle's focus ring and the content's own shadows are
    // not cut off at the top and bottom of a strip barely taller than them.
    // `hidden` would force the other axis to `auto`. A `clip` box is not a
    // scroll container, so as a flex item it would refuse to shrink below
    // its copies; the zero minimum gives that back.
    let horizontal = sx()
        .overflow_x("clip")
        .min_width("0")
        .media(REDUCED_MOTION, sx().overflow_x("auto"))
        .var(MARQUEE_SHIFT, shift("X"))
        .selector(TRACK, sx().width("max-content"))
        .selector(
            format!("{TRACK}:focus-within"),
            sx().width("auto").overflow_x("hidden"),
        )
        .selector(
            PAUSE,
            sx().top("0")
                .bottom("0")
                .margin_top("auto")
                .margin_bottom("auto"),
        );

    let vertical = sx()
        .overflow_y("clip")
        .min_height("0")
        .media(REDUCED_MOTION, sx().overflow_y("auto"))
        .var(MARQUEE_SHIFT, shift("Y"))
        .selector(TRACK, sx().flex_direction("column"))
        .selector(
            format!("{TRACK}:focus-within"),
            sx().max_height("100%").overflow_y("hidden"),
        )
        .selector(COPIES, sx().flex_direction("column"))
        .selector(PAUSE, sx().bottom(offset));

    // Above the moving track, which is its own stacking context, and below
    // the toggle. Only the hidden seam is faded, so under reduced motion,
    // where the first copy starts at the edge, the fade would cover content.
    let fade_edges = sx()
        .selector(
            "&::before, &::after",
            sx().content("\"\"")
                .position("absolute")
                .z_index("1")
                .pointer_events("none")
                .media(REDUCED_MOTION, sx().display("none")),
        )
        .when(
            "horizontal",
            sx().selector(
                "&::before, &::after",
                sx().top("0").bottom("0").width(MARQUEE_FADE_SIZE.value()),
            )
            .selector("&::before", sx().left("0").background(fade("right")))
            .selector("&::after", sx().right("0").background(fade("left"))),
        )
        .when(
            "vertical",
            sx().selector(
                "&::before, &::after",
                sx().left("0").right("0").height(MARQUEE_FADE_SIZE.value()),
            )
            .selector("&::before", sx().top("0").background(fade("bottom")))
            .selector("&::after", sx().bottom("0").background(fade("top"))),
        );

    base.when("horizontal", horizontal)
        .when("vertical", vertical)
        .when(
            "reverse",
            sx().selector(TRACK, sx().animation_direction("reverse")),
        )
        .when(
            "paused",
            sx().selector(TRACK, sx().animation_play_state("paused")),
        )
        .when(
            "pause-on-hover",
            sx().selector(
                "&:hover > [data-slot='track']",
                sx().animation_play_state("paused"),
            ),
        )
        // A focused link must be in view (WCAG 2.4.11): pausing could freeze it
        // outside the clip, so the track stops at its start and scrolls to it.
        // The track and not the root, so focusing the toggle keeps the motion.
        .selector(
            "& > [data-slot='track']:focus-within",
            sx().animation("none"),
        )
        // The toggle could hide a focused link (2.4.11): transparent, not hidden, so Tab
        // still reaches it. A sibling rule, not `:has()`, which Blitz drops.
        .selector(
            "& > [data-slot='track']:focus-within ~ [data-slot='pause']",
            sx().opacity("0").pointer_events("none"),
        )
        .when("fade-edges", fade_edges)
});

base_props! {
    pub struct MarqueeProps {
        /// The axis it scrolls along. A vertical marquee needs a height from
        /// the caller - without one it is as tall as all its copies.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Scrolls towards the end instead of the start.
        #[props(default)]
        reverse: Option<bool>,
        /// Milliseconds per full cycle. The same number moves a longer strip
        /// faster.
        #[props(default)]
        duration: Option<u32>,
        /// Between copies, and between the last and the first.
        #[props(default, into)]
        gap: Input<Size>,
        /// Copies laid in a row. Raise it when the content is shorter than
        /// the marquee and a gap crosses the view. At least 2.
        #[props(default, into)]
        repeat: Input<u8>,
        /// Pointer hover pauses. Not a pause mechanism on its own: neither a
        /// keyboard nor a touch screen can hover.
        #[props(default)]
        pause_on_hover: Option<bool>,
        /// Strictly controlled when set - pair it with `onpausechange`. `None`
        /// leaves the state to the built-in toggle.
        #[props(default)]
        paused: Option<bool>,
        /// The built-in toggle was pressed, with the state it asks for.
        #[props(default)]
        onpausechange: Option<EventHandler<bool>>,
        /// Renders the pause toggle. Turn it off only when the page offers
        /// its own control, through `paused`.
        #[props(default)]
        pause_control: Option<bool>,
        /// Fades both ends into the surface colour, `Paper`'s background.
        #[props(default)]
        fade_edges: Option<bool>,
        /// What scrolls. Rendered once per copy; interactive children work
        /// only in the first copy, the others are `inert`.
        children: Element,
    }
}

/// Content scrolling on its own in an endless loop - a logo strip, a ticker.
///
/// The children are rendered `repeat` times in a row, and the row moves by
/// exactly one copy and one gap per cycle, so the restart cannot be seen.
/// Nothing is measured: it renders the same under SSR and on every renderer.
/// Every copy after the first is `aria-hidden` and `inert`, so it is read and
/// tabbed through once.
///
/// It ships a pause toggle by default. WCAG 2.2.2 asks for a way to stop any
/// motion that starts on its own and runs longer than five seconds, and
/// hover is not one. Under `prefers-reduced-motion: reduce` it does not move
/// at all: it shows one copy in a strip the reader scrolls themselves.
#[component]
pub fn Marquee(props: MarqueeProps) -> Element {
    let theme = use_theme();
    let defaults = theme.marquee;
    let labels = use_localization().marquee;
    let orientation = props.orientation.copied_or(Orientation::Horizontal);
    let repeat = props
        .repeat
        .copied_or(defaults.repeat)
        .max(MARQUEE_MIN_REPEAT);
    let gap = props.gap.copied_or(defaults.gap);

    let mut own_paused = use_signal(|| false);
    let controlled = props.paused.is_some();
    let paused = props.paused.unwrap_or(own_paused());
    let onpausechange = props.onpausechange;
    let toggle = move |_| {
        if !controlled {
            own_paused.set(!paused);
        }
        if let Some(onpausechange) = &onpausechange {
            onpausechange.call(!paused);
        }
    };

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(orientation.state_name(), true)
        .with("reverse", props.reverse.unwrap_or(false))
        .with("paused", paused)
        .with(
            "pause-on-hover",
            props.pause_on_hover.unwrap_or(defaults.pause_on_hover),
        )
        .with(
            "fade-edges",
            props.fade_edges.unwrap_or(defaults.fade_edges),
        )
        .into();

    let variables: Input<Variables> = variables()
        .with(MARQUEE_REPEAT, repeat.to_string())
        .with(MARQUEE_GAP, SizeCss::SPACING.value(gap))
        .with(
            MARQUEE_DURATION.override_var(),
            props.duration.map(|duration| format!("{duration}ms")),
        )
        .into();

    let pause_control = props.pause_control.unwrap_or(defaults.pause_control);

    use_box()
        .framework_sx(&MARQUEE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div { "data-slot": "track",
                    for copy in 0..repeat {
                        div {
                            key: "{copy}",
                            "data-slot": "group",
                            "aria-hidden": (copy > 0).then_some("true"),
                            inert: (copy > 0).then_some(true),
                            {props.children.clone()}
                        }
                    }
                }
                if pause_control {
                    ActionIcon {
                        "data-slot": "pause",
                        variant: "elevated",
                        aria_label: labels.pause,
                        aria_pressed: paused.to_string(),
                        onclick: toggle,
                        if paused {
                            PlayIcon {}
                        } else {
                            PauseIcon {}
                        }
                    }
                }
            },
        )
}
