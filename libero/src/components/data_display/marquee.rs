use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Orientation, Part, ScaleOrCss, States, Variables, base_props,
            parts_enum, variables,
        },
        layout::use_box,
    },
    context::IconSlot,
    hooks::{use_localization, use_theme},
    sx::{REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{
        FOCUS_RING_HALO_SPREAD, MARQUEE_ANIMATION, MARQUEE_DURATION, MARQUEE_FADE_SIZE,
        MARQUEE_GAP, MARQUEE_MIN_REPEAT, MARQUEE_REPEAT, MARQUEE_SHIFT, PAPER_BACKGROUND, Size,
        SizeCss,
    },
    utils::warn,
};

parts_enum! {
    /// [`Marquee`]'s inner parts, for its `parts` prop. Each is matched by
    /// path, so a nested `Marquee` keeps its own styles.
    pub enum MarqueePart {
        /// The moving row of copies.
        Track = "track" => "& > [data-slot='track']",
        /// One copy of the children; all but the first are `inert`.
        Group = "group" => "& > [data-slot='track'] > [data-slot='group']",
        /// The pause toggle.
        Pause = "pause" => "& > [data-slot='pause']",
    }
}

/// One copy plus one gap: the track plus the missing gap, over `repeat`. Right to
/// left the copies trail leftward, so the track moves right (todo 2401).
fn shift(axis: &str, rtl: bool) -> String {
    let (track, sign) = if rtl { ("100%", "+") } else { ("-100%", "-") };
    format!(
        "translate{axis}(calc(({track} {sign} {}) / {}))",
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
    let (track, copies, pause) = (
        MarqueePart::Track.selector(),
        MarqueePart::Group.selector(),
        MarqueePart::Pause.selector(),
    );

    // Reduced-motion guards sit in the selector they undo: a bare `@media` rule
    // loses on specificity. Under it: one copy, no fade, no toggle.
    let base = sx()
        .position("relative")
        .selector(
            track,
            sx().display("flex")
                .gap(MARQUEE_GAP.value())
                .animation(format!(
                    "{MARQUEE_ANIMATION} {} linear infinite",
                    MARQUEE_DURATION.overridable()
                ))
                .media(REDUCED_MOTION, sx().animation("none")),
        )
        .selector(
            copies,
            sx().display("flex")
                .flex_shrink("0")
                .gap(MARQUEE_GAP.value()),
        )
        .selector(
            format!("{copies}:not(:first-child)"),
            sx().media(REDUCED_MOTION, sx().display("none")),
        )
        .selector(
            pause,
            sx().position("absolute")
                .z_index("2")
                .right(offset.clone())
                .rtl(sx().right("auto").left(offset.clone()))
                .media(REDUCED_MOTION, sx().display("none")),
        );

    // A focused track clips both axes: room for a link's ring, the cross axis taken back
    // by the margin. The scroll axis is clipped by the root, so it pads in (todo 2424).
    let ring_room = FOCUS_RING_HALO_SPREAD.value();
    let back = format!("calc(-1 * {ring_room})");

    // `clip` on one axis keeps focus rings visible on the other (`hidden` forces
    // it to `auto`). The zero minimum lets the non-scroller shrink as a flex item.
    let horizontal = sx()
        .overflow_x("clip")
        .min_width("0")
        .media(REDUCED_MOTION, sx().overflow_x("auto"))
        .var(MARQUEE_SHIFT, shift("X", false))
        .rtl(sx().var(MARQUEE_SHIFT, shift("X", true)))
        .selector(track, sx().width("max-content"))
        .selector(
            format!("{track}:focus-within"),
            sx().width("auto")
                .overflow_x("hidden")
                .padding(ring_room.clone())
                .margin_top(back.clone())
                .margin_bottom(back.clone())
                .with("scroll-padding-inline", ring_room.clone()),
        )
        .selector(
            pause,
            sx().top("0")
                .bottom("0")
                .margin_top("auto")
                .margin_bottom("auto"),
        );

    let vertical = sx()
        .overflow_y("clip")
        .min_height("0")
        .media(REDUCED_MOTION, sx().overflow_y("auto"))
        .var(MARQUEE_SHIFT, shift("Y", false))
        .selector(track, sx().flex_direction("column"))
        .selector(
            format!("{track}:focus-within"),
            sx().max_height("100%")
                .box_sizing("border-box")
                .overflow_y("hidden")
                .padding(ring_room.clone())
                .margin_left(back.clone())
                .margin_right(back)
                .with("scroll-padding-block", ring_room),
        )
        .selector(copies, sx().flex_direction("column"))
        .selector(pause, sx().bottom(offset));

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
            sx().selector(track, sx().animation_direction("reverse")),
        )
        .when(
            "paused",
            sx().selector(track, sx().animation_play_state("paused")),
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
        .selector(format!("{track}:focus-within"), sx().animation("none"))
        // The toggle could hide a focused link (2.4.11): transparent, not hidden, so Tab
        // still reaches it. A sibling rule, not `:has()`, which Blitz drops.
        .selector(
            format!("{track}:focus-within ~ [data-slot='pause']"),
            sx().opacity("0").pointer_events("none"),
        )
        .when("fade-edges", fade_edges)
});

base_props! {
    parts(MarqueePart);
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
        /// Between copies: a size word or any CSS, as `gap: "0"`.
        #[props(default, into)]
        gap: Input<ThemeAwareValue>,
        /// Copies in a row, at least 2. Raise it when a gap crosses the view; a
        /// debug build warns when the copies do not fill the box.
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
        /// Rendered once per copy, so give it no `id`: it would repeat.
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
///     Marquee { repeat: 8, fade_edges: true,
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
    let gap = ScaleOrCss::new(props.gap.as_ref(), defaults.gap);
    let horizontal = orientation == Orientation::Horizontal;

    // Debug builds only: a blank strip is a fault of the caller's `repeat` or content.
    let mut lengths = use_signal(|| (0.0_f64, 0.0_f64));
    let warned = use_hook(|| CopyValue::new(false));
    use_effect(use_reactive!(|repeat| {
        let (view, copy) = lengths();
        let mut warned = warned;
        if view > 0.0 && (f64::from(repeat) - 1.0) * copy < view && !warned() {
            warned.set(true);
            warn(&format!(
                "Marquee: {repeat} copies of {copy:.0}px leave a blank strip in {view:.0}px; raise `repeat` or add content."
            ));
        }
    }));
    let measure = use_callback(move |(view, event): (bool, Event<ResizeData>)| {
        let Ok(size) = event.get_border_box_size() else {
            return;
        };
        let length = if horizontal { size.width } else { size.height };
        let (seen_view, seen_copy) = *lengths.peek();
        let next = if view {
            (length, seen_copy)
        } else {
            (seen_view, length)
        };
        if next != (seen_view, seen_copy) {
            lengths.set(next);
        }
    });
    let watch = cfg!(debug_assertions);

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
        .with(MARQUEE_GAP, gap.resolve(SizeCss::SPACING))
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
        .parts(&props.parts)
        .states(&states)
        .variables(&variables)
        .prepare()
        .event(
            "onresize",
            watch.then_some(move |event: Event<ResizeData>| measure.call((true, event))),
        )
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                div { "data-slot": MarqueePart::Track.slot(),
                    for copy in 0..repeat {
                        div {
                            key: "{copy}",
                            "data-slot": MarqueePart::Group.slot(),
                            "aria-hidden": (copy > 0).then_some("true"),
                            inert: (copy > 0).then_some(true),
                            onresize: move |event| {
                                if watch && copy == 0 {
                                    measure.call((false, event));
                                }
                            },
                            {props.children.clone()}
                        }
                    }
                }
                if pause_control {
                    ActionIcon {
                        "data-slot": MarqueePart::Pause.slot(),
                        variant: "elevated",
                        size: "xs",
                        aria_label: labels.pause,
                        aria_pressed: paused.to_string(),
                        onclick: toggle,
                        if paused {
                            Glyph { slot: IconSlot::Play, icon: lucide::play::outlined }
                        } else {
                            Glyph { slot: IconSlot::Pause, icon: lucide::pause::outlined }
                        }
                    }
                }
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<MarqueePart>(),
            [
                ("track", "& > [data-slot='track']"),
                ("group", "& > [data-slot='track'] > [data-slot='group']"),
                ("pause", "& > [data-slot='pause']"),
            ]
        );
    }
}
