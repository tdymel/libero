use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, States, Variables, base_color, base_props, input_from_str, variables,
        },
        layout::use_box,
    },
    hooks::use_theme,
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{LOADER_COLOR, LOADER_SIZE, LoaderDefaults, Size},
};

pub use crate::theme::LoaderVariant;

input_from_str!(LoaderVariant);

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
            .animation("lsx-loader-oval 1.2s linear infinite")
            .media(REDUCED_MOTION, stop_motion(sx())),
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

fn loader_variables(color: &ThemeAwareValue) -> Variables {
    variables().with(LOADER_COLOR, color.resolve(None))
}

base_props! {
    pub struct LoaderProps {
        /// Which shape to draw.
        #[props(default, into)]
        variant: Input<LoaderVariant>,
        /// The square edge.
        #[props(default, into)]
        size: Input<Size>,
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
    let size = props.size.copied_or(theme.loader.size);

    let requested = props
        .color
        .as_ref()
        .cloned()
        .unwrap_or(ThemeAwareValue::Color(theme.loader.color));
    let color = base_color(Some(&requested));
    let variables: Input<Variables> = loader_variables(&color).into();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .with(size.state_name(), true)
        .into();

    let boxed = use_box()
        .framework_sx(&LOADER_BASE_SX)
        .class(&props.class)
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
