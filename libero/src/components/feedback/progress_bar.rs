use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, States, Variables, base_color, base_props, names_itself,
            parts_enum, use_name_warning, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{
        ColorCss, ColorShade, INDETERMINATE_WIDTH, PROGRESS_BAR_ANIMATION, PROGRESS_BAR_COLOR,
        PROGRESS_BAR_FILL, PROGRESS_BAR_INDETERMINATE_STATE, PROGRESS_BAR_RADIUS,
        PROGRESS_BAR_SIZE, PROGRESS_BAR_TRACK, PROGRESS_BAR_TRANSITION, ProgressBarDefaults, Size,
    },
    utils::warn,
};

/// Its own state, as an `Sx` condition cannot say "not indeterminate".
const DETERMINATE_STATE: &str = "determinate";

const SWEEP_DURATION: &str = "1.4s";

static PROGRESS_BAR_TRACK_SX: StaticSx = StaticSx::new(|| {
    ProgressBarDefaults::theme_vars()
        .display("block")
        .width("100%")
        // Clips the fill's square corners to the track's radius.
        .overflow("hidden")
        .height(PROGRESS_BAR_SIZE.value())
        .border_radius(PROGRESS_BAR_RADIUS.value())
        .background(PROGRESS_BAR_TRACK.value())
        // Forced colours drop the background but paint a transparent outline
        // (todo 506).
        .outline("1px solid transparent")
});

static PROGRESS_BAR_FILL_SX: StaticSx = StaticSx::new(|| {
    sx().height("100%")
        .border_radius("inherit")
        .background(PROGRESS_BAR_COLOR.value_or(ColorCss::PRIMARY.value(ColorShade::S6)))
        .media(FORCED_COLORS, sx().background("Highlight"))
        .when(
            DETERMINATE_STATE,
            sx().width(PROGRESS_BAR_FILL.value_or("0%"))
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
                // A frozen sweep would read as 25% done; a dimmed full bar says
                // "busy, amount unknown".
                .media(
                    REDUCED_MOTION,
                    sx().animation("none")
                        .transform("none")
                        .width("100%")
                        .opacity("0.5"),
                ),
        )
});

/// The drawn share, in `0.0..=1.0`. A broken range warns and draws empty.
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

/// `aria-valuenow`, clamped like the fill. A broken range reports the raw
/// value, as `clamp` panics on `min > max`.
fn value_now(value: f64, min: f64, max: f64) -> f64 {
    if value.is_nan() {
        min
    } else if min < max {
        value.clamp(min, max)
    } else {
        value
    }
}

fn percentage(fraction: f64) -> String {
    format!("{}%", (fraction * 100.0).round())
}

/// Whole numbers without a decimal tail: "3 of 10", not "3 of 10.0".
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

parts_enum! {
    /// [`ProgressBar`]'s inner parts, for its `parts` prop. The root is the track.
    pub enum ProgressBarPart {
        /// The drawn share, or the indeterminate sweep.
        Fill = "fill" => "& > [data-slot='fill']",
    }
}

base_props! {
    parts(ProgressBarPart);
    pub struct ProgressBarProps {
        /// Current progress, clamped into `min..=max`. `None` is indeterminate.
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
        #[props(default, into)]
        radius: Input<Size>,
        /// Announced instead of the percentage, e.g. `"4.2 MB of 12 MB"`.
        #[props(default, into)]
        aria_valuetext: Option<String>,
    }
}

/// A determinate or indeterminate progress bar. Give it an accessible name.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::ProgressBar;
/// # fn app() -> Element {
/// # let downloaded = use_signal(|| 0u64);
/// # let total = use_signal(|| 100u64);
/// # rsx! {
/// ProgressBar {
///     aria_label: "Downloading update",
///     value: downloaded() as f64,
///     max: total() as f64,
///     size: "lg",
///     color: "success",
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/feedback/progress-bar>
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

    // Its own `data-state`: a child inherits vars but not state.
    let fill = use_box()
        .framework_sx(&PROGRESS_BAR_FILL_SX)
        .states(&states)
        .prepare()
        .attr("data-slot", ProgressBarPart::Fill.slot())
        .attr("aria-hidden", "true")
        .render(HtmlTag::Div, Vec::new(), ());

    let track = use_box()
        .framework_sx(&PROGRESS_BAR_TRACK_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .variables(&vars)
        .prepare()
        .attr("role", "progressbar")
        .attr("aria-valuemin", aria_number(props.min))
        .attr("aria-valuemax", aria_number(props.max))
        // Omitted while indeterminate, as ARIA requires.
        .attr(
            "aria-valuenow",
            props
                .value
                .map(|value| aria_number(value_now(value, props.min, props.max))),
        );

    // The percentage is only a default: a spread `aria-valuetext` beats it.
    let track = match props.aria_valuetext {
        Some(text) => track.attr("aria-valuetext", text),
        None => track.attr_default("aria-valuetext", percentage),
    };

    track.render(HtmlTag::Div, props.attributes, fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let table: Vec<_> = ProgressBarPart::ALL
            .iter()
            .map(|part| (part.slot(), part.selector()))
            .collect();

        assert_eq!(table, [("fill", "& > [data-slot='fill']")]);
    }

    #[test]
    fn the_fraction_maps_the_range_onto_zero_to_one() {
        assert_eq!(fraction(0.0, 0.0, 100.0), 0.0);
        assert_eq!(fraction(50.0, 0.0, 100.0), 0.5);
        assert_eq!(fraction(100.0, 0.0, 100.0), 1.0);
    }

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
