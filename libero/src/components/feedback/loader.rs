use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_color, base_props, input_from_str, variables},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{LOADER_COLOR, LOADER_SIZE, LoaderDefaults, Size},
};

pub use crate::theme::LoaderVariant;

input_from_str!(LoaderVariant);

/// The ink, with a fallback so a `Loader` still draws if a caller ever renders
/// one outside the theme's `variables()` path.
fn ink() -> String {
    LOADER_COLOR.value_or("currentColor")
}

/// Cancelling an animation drops the element back on its *static* style, not
/// the `from` frame, so the bars stay visible under `prefers-reduced-motion:
/// reduce` even without `restore`. It pins the visible end anyway, so a static
/// style that starts hidden cannot make the loader vanish for exactly the
/// readers who asked for less motion.
///
/// Every variant goes through this helper so that the *next* one cannot be
/// added without meeting the question.
fn stop_motion(restore: Sx) -> Sx {
    restore.animation("none")
}

/// One bar or one dot, with its own delay and its own reduced-motion arm.
///
/// The arm is repeated per child rather than written once against `& > span`
/// because the delay lives on `:nth-child(n)`, and a bare `& > span` inside the
/// media block loses to it on specificity - the middle dot would keep pulsing
/// while the outer two stopped.
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
    // Every number below is a fraction of the one edge. Mantine's geometry,
    // and the two divisions are not independent: three bars at `edge / 5` with
    // two gaps of `edge / 5` fill the row exactly, and three dots at
    // `edge / 3 - edge / 15` with two gaps of `edge / 10` do the same. Change
    // one and the shape stops being square.
    let gap = format!("calc({edge} / 5)");
    let ring = format!("calc({edge} / 8)");
    let dot_size = format!("calc({edge} / 3 - {edge} / 15)");
    let dot_gap = format!("calc({edge} / 10)");
    let c = ink();

    let base = LoaderDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        // A glyph, not a box: never stretched or squeezed by a flex parent.
        .flex("none")
        .width(edge.clone())
        .height(edge.clone())
        // The root is `inline-flex`, so it sits on the text baseline by
        // default and hangs below it. A spinner beside a word should line up
        // with the word.
        .vertical_align("middle")
        // Not a target and not text: no I-beam, no accidental selection.
        .user_select("none");

    let oval = sx().selector(
        "&::after",
        sx().content("\"\"")
            .width("100%")
            .height("100%")
            .border_radius("50%")
            .border_style("solid")
            .border_width(ring)
            // Three sides inked and one transparent is what makes the
            // rotation legible - a complete ring rotating looks static.
            .border_color(format!("{c} {c} {c} transparent"))
            .animation("lsx-loader-oval 1.2s linear infinite")
            // Only `transform` is animated here, so cancelling leaves a
            // static ring with its gap: visible and distinguishable.
            .media(REDUCED_MOTION, stop_motion(sx())),
    );

    // The negative delays start each bar partway through one shared 1.2s
    // cycle, 120ms apart, which is what makes the three read as one wave
    // rather than three loops that happen to be near each other.
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
        // Half the cycle behind its neighbours, so the row breathes from the
        // middle out instead of in lockstep.
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

/// An indeterminate busy indicator: a rotating ring, three bars or three dots.
///
/// Indeterminate on purpose - it says *something is happening*, never how much
/// is left. A known fraction is `ProgressBar`, and showing a spinner for one is
/// a downgrade the reader notices.
///
/// The root is a `<span>`, so a loader is legal inside a `<p>` or a `<button>`.
///
/// # Announcing it
///
/// A loader is always silent: `aria-hidden="true"`, on the assumption that
/// something else on screen already says what is going on. A spinner next to
/// the word "Uploading…" that also announced itself would be read twice.
///
/// | The loader is… | write | the root renders |
/// |---|---|---|
/// | beside its own visible text | `Loader {}` | `aria-hidden="true"` |
/// | inside an already-named control | `Loader {}` | `aria-hidden="true"`; the control's name plus `aria-busy` carries it |
/// | the only content of a region | `Loader {}`, `aria-busy="true"` on the region, and the text in an always-mounted `role="status"` region outside it | `aria-hidden="true"`; the status region carries it |
///
/// The status region must exist before the wait starts and must not sit inside
/// the busy element. Some screen readers skip a live region that mounts with
/// its text, and some hold back changes inside an `aria-busy` subtree until it
/// is no longer busy - by which time the loader is gone. `ComboboxCore` does
/// it this way:
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
/// There is no `label` prop that would make the loader its own status region:
/// that region would mount together with its text, which is the first risk
/// above.
///
/// Not focusable, and no keyboard behaviour at all.
#[component]
pub fn Loader(props: LoaderProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or(theme.loader.variant);
    let size = props.size.copied_or(theme.loader.size);

    // The theme names a `Color`; `base_color` is what turns that - or a
    // caller's literal - into a resolvable value at the default shade.
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

    // `aria-hidden`, not `role="presentation"`: `presentation` strips an
    // element's *implicit* role, and a `<span>` has none - it would be a no-op
    // and leave a nameless node in the tree.
    let boxed = boxed.attr("aria-hidden", "true");

    // `oval` draws itself with `::after` and takes no children; the other two
    // are three real elements, because three independently delayed animations
    // need three boxes and an element has only one `::before`/`::after` pair.
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
