use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_color, base_props},
        layout::use_box,
        variables,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ColorCss, ColorShade, INDETERMINATE_WIDTH, PROGRESS_BAR_ANIMATION, PROGRESS_BAR_COLOR,
        PROGRESS_BAR_FILL, PROGRESS_BAR_INDETERMINATE_STATE, PROGRESS_BAR_RADIUS,
        PROGRESS_BAR_SIZE, PROGRESS_BAR_TRACK, PROGRESS_BAR_TRANSITION, ProgressBarDefaults, Size,
    },
    utils::{names_itself, use_name_warning, warn},
};

/// The `data-state` the determinate arm keys on. Named rather than left as the
/// absence of `indeterminate`, because an `Sx` condition cannot say "not".
const DETERMINATE_STATE: &str = "determinate";

const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

/// How long one indeterminate sweep takes.
const SWEEP_DURATION: &str = "1.4s";

static PROGRESS_BAR_TRACK_SX: StaticSx = StaticSx::new(|| {
    ProgressBarDefaults::theme_vars()
        .display("block")
        .width("100%")
        // The fill is a plain block sized by a percentage; without this its
        // square corners escape the track's radius at both ends.
        .overflow("hidden")
        .height(PROGRESS_BAR_SIZE.value())
        .border_radius(PROGRESS_BAR_RADIUS.value())
        .background(PROGRESS_BAR_TRACK.value())
});

static PROGRESS_BAR_FILL_SX: StaticSx = StaticSx::new(|| {
    sx().height("100%")
        // Picks up the near edge of the track, so a pill track gets a pill
        // fill without restating the radius.
        .border_radius("inherit")
        .background(PROGRESS_BAR_COLOR.value_or(ColorCss::PRIMARY.value(ColorShade::S6)))
        .when(
            DETERMINATE_STATE,
            sx().width(PROGRESS_BAR_FILL.value_or("0%"))
                // The sheet's "nice progress animation": the bar eases to each
                // new value instead of jumping. No keyframes involved.
                .transition(format!(
                    "width {} ease",
                    PROGRESS_BAR_TRANSITION.value_or("100ms")
                ))
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .when(
            PROGRESS_BAR_INDETERMINATE_STATE,
            sx().width(INDETERMINATE_WIDTH)
                .animation(format!(
                    "{PROGRESS_BAR_ANIMATION} {SWEEP_DURATION} ease-in-out infinite"
                ))
                // Under reduced motion the sweep would otherwise freeze as a
                // quarter-width stub parked at the left, which reads as a
                // determinate 25% - a worse lie than no animation. A dimmed
                // full-width track says "busy, amount unknown" while standing
                // still.
                .media(
                    REDUCED_MOTION,
                    sx().animation("none")
                        .transform("none")
                        .width("100%")
                        .opacity("0.5"),
                ),
        )
});

/// The drawn share of the track, in `0.0..=1.0`.
///
/// `max <= min` is a caller error with no sensible reading, so it warns and
/// draws empty rather than dividing by zero.
fn fraction(value: f64, min: f64, max: f64) -> f64 {
    if max <= min || !(min.is_finite() && max.is_finite()) {
        warn("ProgressBar: `max` must be greater than `min`.");
        return 0.0;
    }
    if !value.is_finite() {
        warn("ProgressBar: `value` must be a finite number.");
        return 0.0;
    }
    (value.clamp(min, max) - min) / (max - min)
}

/// The value `aria-valuenow` reports: clamped like the fill, because ARIA
/// requires it to sit inside `min..=max`. A broken range reports the raw value,
/// because `clamp` panics on `min > max` and `fraction` has already warned.
/// `NaN` reports `min`, matching the empty fill it draws.
fn value_now(value: f64, min: f64, max: f64) -> f64 {
    if value.is_nan() {
        min
    } else if min < max {
        value.clamp(min, max)
    } else {
        value
    }
}

/// `42%`, the rounded percentage the fill is drawn at and the a11y floor for
/// `aria-valuetext`.
fn percentage(fraction: f64) -> String {
    format!("{}%", (fraction * 100.0).round())
}

/// A raw bound for `aria-value*`. Whole numbers print without a decimal tail,
/// so a `0..=10` range announces "3 of 10" and not "3 of 10.0".
fn aria_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

fn progress_bar_variables(color: &ThemeAwareValue, fill: Option<String>) -> Variables {
    variables()
        .with(PROGRESS_BAR_COLOR, color.resolve(None))
        .with(PROGRESS_BAR_FILL, fill)
}

base_props! {
    pub struct ProgressBarProps {
        /// Current progress, clamped into `min..=max`.
        ///
        /// `None` is indeterminate: the bar sweeps and drops `aria-valuenow`,
        /// which is how ARIA spells "busy, amount unknown". Required, because
        /// a progress bar with no value at all is not a thing - pass `None`
        /// deliberately.
        #[props(into)]
        value: Option<f64>,
        /// Range start.
        #[props(default = 0.0)]
        min: f64,
        /// Range end.
        #[props(default = 100.0)]
        max: f64,
        /// The fill. A theme color name or a literal CSS color.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Track thickness.
        #[props(default, into)]
        size: Input<Size>,
        /// Corner of the track and the fill.
        ///
        /// Mostly inert on a thin bar: a track is a full pill once the radius
        /// reaches half its height, so on the default `md` track (8px) only
        /// `xs` differs and every step from `sm` up draws the same pill.
        #[props(default, into)]
        radius: Input<Size>,
        /// What a screen reader announces instead of the percentage - a music
        /// player's `"1:34 of 4:02"`, a download's `"4.2 MB of 12 MB"`, neither
        /// of which three numbers can express.
        ///
        /// Unset, the rounded percentage is announced.
        #[props(default, into)]
        aria_valuetext: Option<String>,
    }
}

/// A determinate or indeterminate progress bar.
///
/// Output, not a control: no focus, no keyboard, nothing posted. One `<div>`
/// carrying `role="progressbar"` and the `aria-value*` set, wrapping one
/// decorative fill.
///
/// `aria-valuenow`/`min`/`max` carry the **raw** values rather than the
/// percentage, which is what lets a screen reader say "3 of 10". The bar is
/// deliberately **not** a live region - one ticking sixty times a second inside
/// `role="status"` floods the announcement queue. A caller who needs progress
/// announced wraps it and announces at milestones.
///
/// It needs an accessible name, and `role` sits on the root, so a plain
/// `aria_label` reaches exactly the right element:
///
/// ```rust,ignore
/// ProgressBar {
///     aria_label: "Downloading update",
///     value: downloaded() as f64,
///     max: total() as f64,
///     size: "lg",
///     color: "success",
/// }
/// ```
#[component]
pub fn ProgressBar(props: ProgressBarProps) -> Element {
    let theme = use_theme();
    use_name_warning(
        names_itself(&props.attributes),
        "ProgressBar: no `aria_label` or `aria-labelledby`, so it is announced as just \
         \"progress bar\" and a percentage.",
    );
    let color = base_color(props.color.as_ref());

    let size = props.size.copied_or(theme.progress_bar.size);
    let radius = props.radius.copied_or(theme.progress_bar.radius);

    let fraction = props
        .value
        .map(|value| fraction(value, props.min, props.max));
    let percentage = fraction.map(percentage);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with(DETERMINATE_STATE, fraction.is_some())
        .with(PROGRESS_BAR_INDETERMINATE_STATE, fraction.is_none())
        .into();

    let vars: Input<Variables> = progress_bar_variables(&color, percentage.clone()).into();

    // Its own `data-state`, because the indeterminate and determinate arms of
    // its style are conditions, and a child inherits vars but not state.
    let fill = use_box()
        .framework_sx(&PROGRESS_BAR_FILL_SX)
        .states(&states)
        .prepare()
        .attr("aria-hidden", "true")
        .render(HtmlTag::Div, Vec::new(), ());

    let track = use_box()
        .framework_sx(&PROGRESS_BAR_TRACK_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&vars)
        .prepare()
        .attr("role", "progressbar")
        .attr("aria-valuemin", aria_number(props.min))
        .attr("aria-valuemax", aria_number(props.max))
        // Omitted entirely while indeterminate - the ARIA-correct spelling.
        // `Option::None` renders no attribute.
        .attr(
            "aria-valuenow",
            props
                .value
                .map(|value| aria_number(value_now(value, props.min, props.max))),
        );

    // The prop wins outright; the percentage is only a default, so a caller
    // spreading their own `aria-valuetext` still beats it. Plain `attr` would
    // silently replace theirs - see `attribute-precedence`.
    let track = match props.aria_valuetext {
        Some(text) => track.attr("aria-valuetext", text),
        None => track.attr_default("aria-valuetext", percentage),
    };

    track.render(HtmlTag::Div, props.attributes, fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fraction_maps_the_range_onto_zero_to_one() {
        assert_eq!(fraction(0.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(50.0, 0.0, 100.0), 0.5);
        assert_eq!(fraction(100.0, 0.0, 100.0), 1.0);
    }

    /// The download case: a range that does not start at zero and does not end
    /// at a hundred.
    #[test]
    fn a_range_that_is_not_zero_to_a_hundred() {
        assert_eq!(fraction(5.0, 0.0, 10.0), 0.5);
        assert_eq!(fraction(30.0, 20.0, 60.0), 0.25);
    }

    #[test]
    fn a_value_outside_the_range_is_clamped_not_wrapped() {
        assert_eq!(fraction(-40.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(140.0, 0.0, 100.0), 1.0);
    }

    /// The house rule for a bad prop combination: warn and draw nothing,
    /// rather than divide by zero.
    #[test]
    fn a_max_at_or_below_min_draws_empty() {
        assert_eq!(fraction(5.0, 10.0, 10.0), 0.0);
        assert_eq!(fraction(5.0, 10.0, 0.0), 0.0);
    }

    #[test]
    fn non_finite_input_draws_empty_rather_than_emitting_nan_percent() {
        assert_eq!(fraction(f64::NAN, 0.0, 100.0), 0.0);
        assert_eq!(fraction(f64::INFINITY, 0.0, 100.0), 0.0);
        assert_eq!(fraction(0.0, 0.0, f64::INFINITY), 0.0);
    }

    /// The fill clamps, so the announced value has to as well, or a screen
    /// reader says "140" over a bar drawn full.
    #[test]
    fn aria_valuenow_is_clamped_like_the_fill() {
        assert_eq!(value_now(140.0, 0.0, 100.0), 100.0);
        assert_eq!(value_now(-5.0, 0.0, 100.0), 0.0);
        assert_eq!(value_now(42.0, 0.0, 100.0), 42.0);
        assert_eq!(value_now(5.0, 10.0, 0.0), 5.0);
        assert_eq!(value_now(f64::NAN, 0.0, 100.0), 0.0);
    }

    #[test]
    fn the_percentage_is_rounded_to_whole_units() {
        assert_eq!(percentage(0.0), "0%");
        assert_eq!(percentage(0.425), "43%");
        assert_eq!(percentage(1.0), "100%");
    }

    /// So a `0..=10` range announces "3 of 10", not "3 of 10.0".
    #[test]
    fn whole_aria_bounds_print_without_a_decimal_tail() {
        assert_eq!(aria_number(10.0), "10");
        assert_eq!(aria_number(0.0), "0");
        assert_eq!(aria_number(-3.0), "-3");
        assert_eq!(aria_number(2.5), "2.5");
    }

    #[test]
    fn the_fill_percentage_is_only_written_when_there_is_one() {
        let color = ThemeAwareValue::from("primary");

        let determinate = progress_bar_variables(&color, Some("42%".to_string()));
        assert!(
            determinate
                .to_string()
                .contains(&format!("{}:42%;", PROGRESS_BAR_FILL.name()))
        );

        let indeterminate = progress_bar_variables(&color, None);
        assert!(!indeterminate.to_string().contains(PROGRESS_BAR_FILL.name()));
    }
}
