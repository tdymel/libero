use dioxus::prelude::*;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            HtmlTag, Input, Orientation, PauseIcon, PlayIcon, States, Variables, base_props,
            variables,
        },
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

/// One copy plus one gap: the track plus the missing gap, over `repeat`.
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

    // Reduced-motion guards sit in the selector they undo: a bare `@media` rule
    // loses on specificity. Under it: one copy, no fade, no toggle.
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

    // `clip` on one axis keeps focus rings visible on the other (`hidden` forces
    // it to `auto`). The zero minimum lets the non-scroller shrink as a flex item.
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

    // Between the track and the toggle. Off under reduced motion, where it
    // would cover the first copy.
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
        // A focused link must be in view (2.4.11): the track stops at its start
        // and scrolls to it. Focusing the toggle keeps the motion.
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
        /// The scroll axis. A vertical marquee needs a height from the caller.
        #[props(default, into)]
        orientation: Input<Orientation>,
        /// Scrolls towards the end instead of the start.
        #[props(default)]
        reverse: Option<bool>,
        /// Milliseconds per full cycle.
        #[props(default)]
        duration: Option<u32>,
        /// Between copies.
        #[props(default, into)]
        gap: Input<Size>,
        /// Copies in a row, at least 2. Raise it when a gap crosses the view.
        #[props(default, into)]
        repeat: Input<u8>,
        /// Pointer hover pauses; not a pause mechanism on its own.
        #[props(default)]
        pause_on_hover: Option<bool>,
        /// Controlled when set, paired with `onpausechange`.
        #[props(default)]
        paused: Option<bool>,
        /// The built-in toggle was pressed, with the state it asks for.
        #[props(default)]
        onpausechange: Option<EventHandler<bool>>,
        /// The pause toggle. Turn it off only when the page drives `paused` itself.
        #[props(default)]
        pause_control: Option<bool>,
        /// Fades both ends into the surface colour.
        #[props(default)]
        fade_edges: Option<bool>,
        /// What scrolls. Only the first copy is interactive; the others are `inert`.
        children: Element,
    }
}

/// Content scrolling in an endless loop, with a pause toggle (WCAG 2.2.2).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Marquee;
/// # fn app() -> Element {
/// rsx! {
///     Marquee { fade_edges: true,
///         span { "Rust" }
///         span { "Dioxus" }
///         span { "Libero" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/marquee>
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
                        size: "xs",
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
