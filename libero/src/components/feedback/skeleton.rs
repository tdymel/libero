use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, ResponsiveScale, States, Variables, base_props, responsive_var_sx,
            variables, with_own,
        },
        layout::use_box,
    },
    hooks::{use_range_css, use_theme},
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CssVar, Responsive, SKELETON_ANIMATION, SKELETON_COLOR, SKELETON_DURATION, SKELETON_HEIGHT,
        SKELETON_RADIUS, SKELETON_WIDTH, SizeCss, SkeletonDefaults,
    },
};

const VISIBLE_STATE: &str = "visible";
const ANIMATE_STATE: &str = "animate";
const CIRCLE_STATE: &str = "circle";

/// Reduced-motion resting opacity; `0.4` would read as a disabled control.
const SETTLED_OPACITY: &str = "0.7";

static SKELETON_BASE_SX: StaticSx = StaticSx::new(|| {
    SkeletonDefaults::theme_vars()
        .height(SKELETON_HEIGHT.value_or("auto"))
        .width(SKELETON_WIDTH.value_or("100%"))
        .border_radius(SKELETON_RADIUS.value())
        // No transform: it would trap the loaded content's fixed and z-indexed children.
        // A circle without `height` shrinks to its children.
        .when(
            CIRCLE_STATE,
            sx().var(SKELETON_RADIUS, "1000px")
                .width(SKELETON_WIDTH.value_or("fit-content")),
        )
        .when(
            VISIBLE_STATE,
            // Children are hidden, not covered, so any z-index or surface works.
            // A descendant setting `visibility: visible` itself shows through.
            sx().position("relative")
                .overflow("hidden")
                .visibility("hidden")
                .selector(
                    "&::after",
                    sx().content("\"\"")
                        .position("absolute")
                        .inset("0")
                        .visibility("visible")
                        .background(SKELETON_COLOR.value())
                        // Forced colours would paint it `Canvas`: a blank card.
                        .media(FORCED_COLORS, sx().background("GrayText")),
                ),
        )
        // Only `::after` pulses: the root's opacity would fade the children too.
        .when(
            ANIMATE_STATE,
            sx().selector(
                "&::after",
                sx().animation(format!(
                    "{SKELETON_ANIMATION} {} linear infinite",
                    SKELETON_DURATION.value()
                ))
                .media(
                    REDUCED_MOTION,
                    sx().animation("none").opacity(SETTLED_OPACITY),
                ),
            ),
        )
});

/// `circle` copies the height into the width.
fn skeleton_variables(height: Option<String>, width: Option<String>, circle: bool) -> Variables {
    let width = if circle { height.clone() } else { width };
    variables()
        .with(SKELETON_HEIGHT, height)
        .with(SKELETON_WIDTH, width)
}

base_props! {
    pub struct SkeletonProps {
        /// Show the placeholder; `false` shows the children.
        #[props(default = true)]
        visible: bool,
        /// A CSS length. Unset, the height of the children.
        /// Per breakpoint too: `responsive("40px").md("64px")`.
        #[props(default, into)]
        height: Input<Responsive<ThemeAwareValue>>,
        /// A CSS length. Unset, `100%`. Ignored when `circle`. Per breakpoint too.
        #[props(default, into)]
        width: Input<Responsive<ThemeAwareValue>>,
        /// Round, with width equal to `height`. Without `height`, a pill around the children.
        #[props(default)]
        circle: bool,
        /// Corner: a size word or any CSS, as `radius: "0"`. Ignored when `circle`.
        /// Per breakpoint too: `responsive(Size::Xs).md(Size::Lg)`.
        #[props(default, into)]
        radius: Input<Responsive<ThemeAwareValue>>,
        /// Run the pulse.
        #[props(default)]
        animate: Option<bool>,
        /// The real content, when the skeleton wraps it.
        children: Element,
    }
}

/// A placeholder for content that is still loading. Its grey is decoration, `aria-hidden`
/// and `inert`: the caller's `aria-busy` region carries the state, not the 3:1 contrast.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Skeleton;
/// # fn app() -> Element {
/// # let loading = use_signal(|| true);
/// # rsx! {
/// Skeleton { height: "40px", circle: true }
/// Skeleton { visible: loading(), "Loaded text" }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/feedback/skeleton>
#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let theme = use_theme();
    let radius = ResponsiveScale::new(
        props.radius.as_ref(),
        theme.skeleton.radius,
        SizeCss::RADIUS,
    );
    // A circle is round by its own `1000px`, not by the radius.
    let radius_css = radius.custom_css().filter(|_| !props.circle);
    let height = props.height.as_ref();
    let width = match props.circle {
        true => height,
        false => props.width.as_ref(),
    };
    let plain = |value: Option<&Responsive<ThemeAwareValue>>| {
        value
            .filter(|value| value.is_plain())
            .and_then(|value| value.base_ref().resolve(None))
    };
    // Per breakpoint, the vars move to the user layer: an inline one would beat them.
    let var_sx = |value: Option<&Responsive<ThemeAwareValue>>, var: CssVar| {
        value.and_then(|value| responsive_var_sx(value, var, None))
    };
    let extra = [
        var_sx(height, SKELETON_HEIGHT),
        var_sx(width, SKELETON_WIDTH),
        radius.sx.clone().filter(|_| !props.circle),
    ]
    .into_iter()
    .flatten()
    .reduce(Sx::and);
    let own_sx = extra.map(|extra| with_own(Some(extra), &props.sx));

    let states = props
        .states
        .unwrap_or_default()
        .with(VISIBLE_STATE, props.visible)
        .with(
            ANIMATE_STATE,
            props.animate.unwrap_or(theme.skeleton.animate),
        )
        .with(CIRCLE_STATE, props.circle);
    let states: Input<States> = match props.circle {
        true => states,
        false => states.responsive(&radius.steps, |size| size.radius_state_name()),
    }
    .into();
    use_range_css(
        &SKELETON_BASE_SX,
        match props.circle {
            true => Vec::new(),
            false => radius
                .steps
                .range_copies(|size| size.radius_state_name(), SkeletonDefaults::radius_sx),
        },
    );

    let vars: Input<Variables> = skeleton_variables(plain(height), plain(width), props.circle)
        .with(SizeCss::RADIUS.override_var(), radius_css)
        .into();

    use_box()
        .framework_sx(&SKELETON_BASE_SX)
        .class(&props.class)
        .visibility(&props.hidden_from, &props.visible_from)
        .sx(own_sx.as_ref().unwrap_or(&props.sx))
        .states(&states)
        .variables(&vars)
        .prepare()
        .attr("aria-hidden", props.visible.then_some("true"))
        .attr("inert", props.visible)
        .render(HtmlTag::Div, props.attributes, props.children)
}
