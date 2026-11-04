use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_color, base_props, input_from_str, label_color,
            variables, warning_edge,
        },
        layout::use_box,
    },
    hooks::{use_range_css, use_theme},
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{CssVar, LOADER_COLOR, LOADER_SIZE, LoaderDefaults, Responsive, Size},
};

pub use crate::theme::LoaderVariant;

input_from_str!(LoaderVariant);

/// The whole inset shadow, unset for most inks, so they paint none.
const LOADER_EDGE: CssVar = CssVar::new("--lsx-loader-edge");

fn ink() -> String {
    LOADER_COLOR.value_or("currentColor")
}

/// Reduced motion: stops the animation and pins the visible end, so a static
/// style that starts hidden cannot make the loader vanish. Every variant uses it.
fn stop_motion(restore: Sx) -> Sx {
    restore.animation("none")
}

/// Reduced-motion arm per child: a bare `& > span` loses to `:nth-child(n)` on
/// specificity, and the middle dot would keep pulsing.
fn staggered(animation: String, restore: Sx) -> Sx {
    sx().animation(animation)
        .media(REDUCED_MOTION, stop_motion(restore))
}

fn bar(delay: &str) -> Sx {
    staggered(
        format!("lsx-loader-bars 1.2s cubic-bezier(0, 0.5, 0.5, 1) {delay} infinite"),
        sx().transform("scale(1)").opacity("1"),
    )
}

fn dot(delay: &str) -> Sx {
    staggered(
        format!("lsx-loader-dots 0.8s linear {delay} infinite"),
        sx().transform("scale(1)").opacity("1"),
    )
}

static LOADER_BASE_SX: StaticSx = StaticSx::new(|| {
    let edge = LOADER_SIZE.value();
    // Sizes and gaps together fill the edge exactly; change one and the shape
    // stops being square.
    let gap = format!("calc({edge} / 5)");
    let ring = format!("calc({edge} / 8)");
    let dot_size = format!("calc({edge} / 3 - {edge} / 15)");
    let dot_gap = format!("calc({edge} / 10)");
    let c = ink();

    let base = LoaderDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex("none")
        .width(edge.clone())
        .height(edge.clone())
        // Lines up with a word beside it instead of hanging below the baseline.
        .vertical_align("middle")
        .user_select("none");

    let oval = sx().selector(
        "&::after",
        sx().content("\"\"")
            .width("100%")
            .height("100%")
            .border_radius("50%")
            .border_style("solid")
            .border_width(ring)
            // The gap makes the rotation visible; a full ring looks static.
            .border_color(format!("{c} {c} {c} transparent"))
            .box_shadow(LOADER_EDGE.value_or("none"))
            .animation("lsx-loader-oval 1.2s linear infinite")
            .media(REDUCED_MOTION, stop_motion(sx()))
            // Forced colours keep alpha only on backgrounds: the gap would close.
            .media(
                FORCED_COLORS,
                sx().forced_color_adjust("none")
                    .box_shadow("none")
                    .border_color("CanvasText CanvasText CanvasText transparent"),
            ),
    );

    // Negative delays phase the bars into one wave.
    // Forced colours paint backgrounds `Canvas`; the oval's border survives.
    let forced = || sx().background_color("CanvasText");
    let bars = sx()
        .gap(gap)
        .selector(
            "& > span",
            sx().flex("1")
                .height("100%")
                .background_color(c.clone())
                .box_shadow(LOADER_EDGE.value_or("none"))
                .media(FORCED_COLORS, forced()),
        )
        .selector("& > span:nth-child(1)", bar("-240ms"))
        .selector("& > span:nth-child(2)", bar("-120ms"))
        .selector("& > span:nth-child(3)", bar("0ms"));

    let dots = sx()
        .gap(dot_gap)
        .selector(
            "& > span",
            sx().width(dot_size.clone())
                .height(dot_size)
                .border_radius("50%")
                .background_color(c)
                .box_shadow(LOADER_EDGE.value_or("none"))
                .media(FORCED_COLORS, forced()),
        )
        .selector("& > span:nth-child(1)", dot("0ms"))
        // Half a cycle behind: the row breathes from the middle out.
        .selector("& > span:nth-child(2)", dot("0.4s"))
        .selector("& > span:nth-child(3)", dot("0ms"));

    base.when("oval", oval)
        .when("bars", bars)
        .when("dots", dots)
});

/// The text role: alone in a busy region the ink is the only cue, so it needs 3:1 on the page
/// (todo 5303, as `ProgressBar`'s 1577). A literal paints as given.
fn loader_variables(color: &ThemeAwareValue) -> Variables {
    variables()
        .with(LOADER_COLOR, label_color(color))
        .with(LOADER_EDGE, ink_edge(color))
}

/// Yellow stays under 3:1 on a light page even in the text role, so warning gets an inset edge
/// in ink (todo 2033). On the oval it runs round the ring's inside, gap included.
fn ink_edge(color: &ThemeAwareValue) -> Option<String> {
    warning_edge(color).map(|ink| format!("inset 0 0 0 1px {ink}"))
}

base_props! {
    pub struct LoaderProps {
        /// Which shape to draw.
        #[props(default, into)]
        variant: Input<LoaderVariant>,
        /// The square edge. Per breakpoint too: `responsive(Size::Sm).md(Size::Lg)`.
        #[props(default, into)]
        size: Input<Responsive<Size>>,
        /// The ink.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
    }
}

/// An indeterminate, silent (`aria-hidden`) busy indicator: a ring, bars or dots.
///
/// Announce the wait from an always-mounted status region outside the busy one:
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{Box, Loader, VisuallyHidden};
/// # fn app() -> Element {
/// # let loading = true;
/// # let rows: Vec<String> = Vec::new();
/// # rsx! {
/// Box { "aria-busy": loading,
///     if loading { Loader {} } else { ResultList { rows } }
/// }
/// VisuallyHidden { role: "status", if loading { "Loading results" } }
/// # } }
/// # #[component] fn ResultList(rows: Vec<String>) -> Element { rsx! {} }
/// ```
///
/// Docs: <https://libero-ui.dev/feedback/loader>
#[component]
pub fn Loader(props: LoaderProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or(theme.loader.variant);
    let sizes = props.size.copied_or(theme.loader.size.into());

    let requested = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or(ThemeAwareValue::Color(theme.loader.color));
    let color = base_color("Loader", Some(&requested));
    let variables: Input<Variables> = loader_variables(&color).into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .responsive(&sizes, |size| size.state_name())
        .into();
    use_range_css(
        &LOADER_BASE_SX,
        sizes.range_copies(|size| size.state_name(), LoaderDefaults::size_sx),
    );

    let boxed = use_box()
        .framework_sx(&LOADER_BASE_SX)
        .class(&props.class)
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare();

    // Not `role="presentation"`: a `<span>` has no implicit role to strip.
    let boxed = boxed.attr("aria-hidden", "true");

    // Three delayed animations need three boxes; one element has one `::after`.
    let children = match variant {
        LoaderVariant::Oval => rsx! {},
        LoaderVariant::Bars | LoaderVariant::Dots => rsx! {
            span {}
            span {}
            span {}
        },
    };

    boxed.render(HtmlTag::Span, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 5303: the ink needs 3:1 on the page, by its body or, for warning, its edge.
    #[test]
    fn every_shipped_ink_reaches_3_to_1_on_the_page() {
        use crate::{
            components::feedback::progress_value::sheet_hex, css::Stylesheet, theme::ThemeSet,
        };

        let mut short = Vec::new();
        for set in ThemeSet::CATALOGUE {
            for theme in [Some(set.light_theme()), set.dark_theme()]
                .into_iter()
                .flatten()
            {
                let css = Stylesheet::from(theme).as_str().to_string();
                let on_page = |paint: String| sheet_hex(&css, paint).contrast_ratio(theme.surface);
                for name in [
                    "primary",
                    "secondary",
                    "error",
                    "info",
                    "success",
                    "warning",
                    "neutral",
                    "muted",
                ] {
                    let color = base_color("Loader", Some(&ThemeAwareValue::from(name)));
                    let body = on_page(label_color(&color).expect("a palette colour"));
                    let edge = warning_edge(&color).map(on_page).unwrap_or(0.0);
                    if body.max(edge) < 3.0 {
                        let scheme = theme.surface.color_scheme();
                        let ratio = body.max(edge);
                        short.push(format!("{} {scheme} {name}: {ratio:.2}:1", set.name()));
                    }
                }
            }
        }
        assert!(
            short.is_empty(),
            "the shipped inks' contrast moved: {short:?}"
        );
    }

    #[test]
    fn a_literal_ink_paints_as_given_with_no_edge() {
        for literal in ["currentColor", "#123456"] {
            let vars =
                loader_variables(&base_color("Loader", Some(&ThemeAwareValue::from(literal))));
            let vars = vars.to_string();
            assert!(
                vars.contains(&format!("{}:{literal};", LOADER_COLOR.name())),
                "{vars}"
            );
            assert!(!vars.contains(LOADER_EDGE.name()), "{vars}");
        }
    }

    #[test]
    fn a_theme_ink_takes_its_text_role_and_only_warning_an_edge() {
        let vars = |name: &str| {
            loader_variables(&base_color("Loader", Some(&ThemeAwareValue::from(name)))).to_string()
        };
        assert!(vars("primary").contains("--lsx-primary-text-6"));
        assert!(vars("muted").contains("--lsx-muted-text-6"));
        assert!(!vars("primary").contains(LOADER_EDGE.name()));
        assert!(vars("warning").contains(LOADER_EDGE.name()));
    }
}
