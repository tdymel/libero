use std::f64::consts::PI;

use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        common::{
            HtmlTag, Input, Part, States, Variables, base_color, base_props, names_itself,
            parts_enum, text_color, use_name_warning, variables,
        },
        layout::use_box,
    },
    hooks::{use_css, use_theme},
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CIRCULAR_PROGRESS_COLOR, CIRCULAR_PROGRESS_SIZE, CIRCULAR_PROGRESS_THICKNESS,
        CIRCULAR_PROGRESS_TRACK, CIRCULAR_PROGRESS_TRANSITION, CircularProgressDefaults, Color,
        ColorShade, ColorValue, Size,
    },
};

use super::progress_value::{aria_number, fraction, percentage, value_now};

const DETERMINATE_STATE: &str = "determinate";
const INDETERMINATE_STATE: &str = "indeterminate";

/// The share of the ring the indeterminate arc covers while it spins.
const SPIN_SHARE: f64 = 0.25;

static CIRCULAR_PROGRESS_SX: StaticSx = StaticSx::new(|| {
    let edge = CIRCULAR_PROGRESS_SIZE.value();
    CircularProgressDefaults::theme_vars()
        .position("relative")
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex("none")
        .width(edge.clone())
        .height(edge.clone())
        .vertical_align("middle")
        // The track: a ring as wide as the arc, under it.
        .selector(
            "&::before",
            sx().content("\"\"")
                .position("absolute")
                .inset("0")
                .border_radius("50%")
                .border_style("solid")
                .border_width(format!(
                    "calc({edge} * {})",
                    CIRCULAR_PROGRESS_THICKNESS.value_or("0.1")
                ))
                .border_color(CIRCULAR_PROGRESS_TRACK.value()),
        )
});

/// The box both the arc and its ink edge draw in.
fn ring_box_sx() -> Sx {
    sx().position("absolute")
        .inset("0")
        .width("100%")
        .height("100%")
        .overflow("visible")
        .selector(
            "& > circle",
            sx().transition(format!(
                "stroke-dashoffset {} ease",
                CIRCULAR_PROGRESS_TRANSITION.value_or("100ms")
            ))
            .media(REDUCED_MOTION, sx().transition("none")),
        )
}

static CIRCULAR_PROGRESS_ARC_SX: StaticSx = StaticSx::new(|| {
    ring_box_sx()
        .color(
            CIRCULAR_PROGRESS_COLOR
                .value_or(ColorValue::Text(Color::Primary, ColorShade::S6).value()),
        )
        .media(
            FORCED_COLORS,
            sx().forced_color_adjust("none").color("Highlight"),
        )
});

/// A frozen quarter would read as 25% done, so reduced motion shows a full dashed ring.
static CIRCULAR_PROGRESS_SPIN_SX: StaticSx = StaticSx::new(|| {
    sx().animation("lsx-loader-oval 1.4s linear infinite")
        .media(
            REDUCED_MOTION,
            sx().animation("none").selector(
                "& > circle",
                sx().stroke_dasharray(SPIN_DASHES).stroke_dashoffset("0"),
            ),
        )
});

/// Dash and gap in viewBox units, 24 of each around the `md` ring.
const SPIN_DASHES: &str = "7 4.78";

static CIRCULAR_PROGRESS_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .font_size(format!("calc({} / 4)", CIRCULAR_PROGRESS_SIZE.value()))
        .line_height("1")
});

/// The ring's circle in a `0 0 100 100` viewBox: radius, stroke width and length.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Ring {
    radius: f64,
    width: f64,
    length: f64,
}

impl Ring {
    fn new(thickness_percent: u8) -> Self {
        let width = f64::from(thickness_percent.clamp(1, 50));
        let radius = 50.0 - width / 2.0;
        Self {
            radius,
            width,
            length: 2.0 * PI * radius,
        }
    }

    /// The dash offset that leaves `share` of the ring drawn.
    fn offset(self, share: f64) -> f64 {
        self.length * (1.0 - share)
    }
}

/// Yellow stays under 3:1 on a light track even in the text role, so a warning arc
/// sits on a wider one in ink, as `ProgressBar`'s inset edge (todo 2033).
fn inks_an_edge(color: &ThemeAwareValue) -> bool {
    matches!(
        color,
        ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Warning, _))
    )
}

/// The ink edge's width on each side of the arc, in viewBox units: 1px on the `md` ring.
const EDGE_WIDTH: f64 = 2.8;

static CIRCULAR_PROGRESS_EDGE_SX: StaticSx = StaticSx::new(|| {
    ring_box_sx()
        .color(ColorValue::Shade(Color::Ink, ColorShade::S6).value())
        // Forced colours paint the arc `Highlight` already.
        .media(FORCED_COLORS, sx().display("none"))
});

/// One ring in a `0 0 100 100` box. SVG attributes, not CSS: native bakes an
/// inline svg from its attributes and `currentColor` only.
fn arc_svg(
    class: String,
    slot: Option<&'static str>,
    ring: Ring,
    width: f64,
    offset: f64,
) -> Element {
    let length = svg_number(ring.length);
    rsx! {
        svg {
            class,
            "data-slot": slot,
            "aria-hidden": "true",
            "viewBox": "0 0 100 100",
            circle {
                cx: "50",
                cy: "50",
                r: svg_number(ring.radius),
                fill: "none",
                stroke: "currentColor",
                "stroke-width": svg_number(width),
                "stroke-dasharray": "{length} {length}",
                "stroke-dashoffset": svg_number(offset),
                // Starts at 12 o'clock.
                transform: "rotate(-90 50 50)",
            }
        }
    }
}

/// Short SVG numbers: `282.74`, not `282.7433388230814`.
fn svg_number(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    format!("{rounded}")
}

parts_enum! {
    /// [`CircularProgress`]'s inner parts, for its `parts` prop. The root is the track.
    pub enum CircularProgressPart {
        /// The `<svg>` holding the value arc.
        Arc = "arc" => "& > [data-slot='arc']",
        /// The centred children.
        Label = "label" => "& > [data-slot='label']",
    }
}

base_props! {
    parts(CircularProgressPart);
    pub struct CircularProgressProps {
        /// Current progress, clamped into `min..=max`. `None` is indeterminate.
        #[props(into)]
        value: Option<f64>,
        /// Range start.
        #[props(default = 0.0)]
        min: f64,
        /// Range end.
        #[props(default = 100.0)]
        max: f64,
        /// The arc. A theme color name paints its text shade, as `ProgressBar`'s fill; a
        /// literal CSS color paints as given.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// The outer edge.
        #[props(default, into)]
        size: Input<Size>,
        /// The ring's width, a share of the edge that grows with the step.
        #[props(default, into)]
        thickness: Input<Size>,
        /// Announced instead of the percentage, e.g. `"3 of 8 files"`.
        #[props(default, into)]
        aria_valuetext: Option<String>,
        /// Drawn in the middle of the ring, e.g. the percentage or an icon. Hidden from
        /// screen readers: say the same in `aria_valuetext` when it differs from the percentage.
        #[props(default)]
        children: Element,
    }
}

/// A ring that fills clockwise from the top, or spins while the amount is unknown.
/// Give it an accessible name.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::CircularProgress;
/// # fn app() -> Element {
/// # let sent = use_signal(|| 3.0);
/// # rsx! {
/// CircularProgress { aria_label: "Upload", value: sent(), max: 8.0, size: "lg",
///     "{sent} of 8"
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/feedback/circular-progress>
#[component]
pub fn CircularProgress(props: CircularProgressProps) -> Element {
    let theme = use_theme();
    use_name_warning(
        names_itself(&props.attributes),
        "CircularProgress: no `aria_label` or `aria-labelledby`, so it is announced as just \
         \"progress bar\" and a percentage.",
    );
    let color = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or_else(|| ThemeAwareValue::from(theme.circular_progress.color));
    let color = base_color(Some(&color));
    let size = props.size.copied_or(theme.circular_progress.size);
    let thickness = props.thickness.copied_or(theme.circular_progress.thickness);
    let percent = theme.circular_progress.thicknesses.get(thickness);
    let ring = Ring::new(percent);

    let fraction = props
        .value
        .map(|value| fraction("CircularProgress", value, props.min, props.max));
    let offset = ring.offset(fraction.unwrap_or(SPIN_SHARE));

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(DETERMINATE_STATE, fraction.is_some())
        .with(INDETERMINATE_STATE, fraction.is_none())
        .into();
    let vars: Input<Variables> = variables()
        .with(CIRCULAR_PROGRESS_COLOR, text_color(&color))
        .with(
            CIRCULAR_PROGRESS_THICKNESS,
            Some(format!("{}", ring.width / 100.0)),
        )
        .into();

    let arc_class = use_css(Some(&CIRCULAR_PROGRESS_ARC_SX), CssLayer::Framework);
    let edge_class = use_css(Some(&CIRCULAR_PROGRESS_EDGE_SX), CssLayer::Framework);
    let spin_class = use_css(Some(&CIRCULAR_PROGRESS_SPIN_SX), CssLayer::Framework);
    let label_class = use_css(Some(&CIRCULAR_PROGRESS_LABEL_SX), CssLayer::Framework);
    let spin_class = spin_class.filter(|_| fraction.is_none());
    let classes = |own: Option<String>| {
        [own, spin_class.clone()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let edge = inks_an_edge(&color).then(|| {
        arc_svg(
            classes(edge_class),
            None,
            ring,
            ring.width + 2.0 * EDGE_WIDTH,
            offset,
        )
    });

    let inner = rsx! {
        {edge}
        {arc_svg(classes(arc_class), Some(CircularProgressPart::Arc.slot()), ring, ring.width, offset)}
        // Hidden: the value is already read, and "40%" twice is noise.
        span { class: label_class, "data-slot": CircularProgressPart::Label.slot(), "aria-hidden": "true", {props.children} }
    };

    let root = use_box()
        .framework_sx(&CIRCULAR_PROGRESS_SX)
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
                .and_then(|value| aria_number(value_now(value, props.min, props.max))),
        );

    // The percentage is only a default: a spread `aria-valuetext` beats it.
    let root = match props.aria_valuetext {
        Some(text) => root.attr("aria-valuetext", text),
        None => root.attr_default("aria-valuetext", fraction.map(percentage)),
    };

    root.render(HtmlTag::Span, props.attributes, inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        let table: Vec<_> = CircularProgressPart::ALL
            .iter()
            .map(|part| (part.slot(), part.selector()))
            .collect();

        assert_eq!(
            table,
            [
                ("arc", "& > [data-slot='arc']"),
                ("label", "& > [data-slot='label']"),
            ]
        );
    }

    #[test]
    fn the_ring_sits_inside_the_view_box() {
        let ring = Ring::new(10);
        assert_eq!(ring.radius + ring.width / 2.0, 50.0);
        assert_eq!(svg_number(ring.length), "282.74");
    }

    #[test]
    fn the_offset_leaves_the_share_drawn() {
        let ring = Ring::new(10);
        assert_eq!(ring.offset(0.0), ring.length);
        assert_eq!(ring.offset(1.0), 0.0);
        assert_eq!(
            svg_number(ring.offset(0.25)),
            svg_number(ring.length * 0.75)
        );
    }

    /// A theme step past half the edge would draw a negative radius.
    #[test]
    fn an_oversized_thickness_is_clamped() {
        assert!(Ring::new(200).radius >= 25.0);
        assert!(Ring::new(0).width >= 1.0);
    }

    /// The arc has no text, so it needs 3:1 on the track and the page (1.4.11), as
    /// `ProgressBar`'s fill; this pins the shipped shortfalls.
    #[test]
    fn every_shipped_arc_reaches_3_to_1_on_its_track_and_page() {
        use crate::{
            components::feedback::progress_value::sheet_hex,
            css::Stylesheet,
            theme::{ColorCss, HexColor, ThemeSet},
        };

        let mut short = Vec::new();
        for set in ThemeSet::CATALOGUE {
            for theme in [Some(set.light_theme()), set.dark_theme()]
                .into_iter()
                .flatten()
            {
                let css = Stylesheet::from(theme).as_str().to_string();
                let track = sheet_hex(
                    &css,
                    ColorCss::MUTED.value(CircularProgressDefaults::DEFAULT.track_shade),
                );
                for name in ["primary", "error", "info", "success", "warning"] {
                    let color = base_color(Some(&ThemeAwareValue::from(name)));
                    let reach = |paint: HexColor| {
                        paint
                            .contrast_ratio(track)
                            .min(paint.contrast_ratio(theme.surface))
                    };
                    let arc = reach(sheet_hex(
                        &css,
                        text_color(&color).expect("a palette colour"),
                    ));
                    // An edged arc counts when either its body or its edge clears.
                    let edge = inks_an_edge(&color).then(|| {
                        reach(sheet_hex(
                            &css,
                            ColorValue::Shade(Color::Ink, ColorShade::S6).value(),
                        ))
                    });
                    let ratio = arc.max(edge.unwrap_or(0.0));
                    if ratio < 3.0 {
                        let scheme = theme.surface.color_scheme();
                        short.push(format!("{} {scheme} {name}: {ratio:.2}:1", set.name()));
                    }
                }
            }
        }
        // As ProgressBar's: Nord's official pastel accents stay (todo 2034).
        assert_eq!(
            short,
            [
                "Nord light primary: 2.39:1",
                "Nord light info: 2.48:1",
                "Nord light success: 2.44:1",
            ],
            "the shipped arcs' contrast moved"
        );
    }
}
