use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props, variables},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{
        SKELETON_ANIMATION, SKELETON_COLOR, SKELETON_DURATION, SKELETON_HEIGHT, SKELETON_RADIUS,
        SKELETON_WIDTH, Size, SkeletonDefaults,
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
        // Own compositing layer: the pulse does not repaint the subtree.
        .transform("translateZ(0)")
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
        #[props(default, into)]
        height: Input<ThemeAwareValue>,
        /// A CSS length. Unset, `100%`. Ignored when `circle`.
        #[props(default, into)]
        width: Input<ThemeAwareValue>,
        /// Round, with width equal to `height`.
        #[props(default)]
        circle: bool,
        /// Corner. Ignored when `circle`.
        #[props(default, into)]
        radius: Input<Size>,
        /// Run the pulse.
        #[props(default)]
        animate: Option<bool>,
        /// The real content, when the skeleton wraps it.
        children: Element,
    }
}

/// A placeholder for content that is still loading.
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
    let radius = props.radius.copied_or(theme.skeleton.radius);

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(VISIBLE_STATE, props.visible)
        .with(
            ANIMATE_STATE,
            props.animate.unwrap_or(theme.skeleton.animate),
        )
        .with(CIRCLE_STATE, props.circle)
        .with(radius.radius_state_name(), !props.circle)
        .into();

    let vars: Input<Variables> = skeleton_variables(
        props.height.resolve(None),
        props.width.resolve(None),
        props.circle,
    )
    .into();

    use_box()
        .framework_sx(&SKELETON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&vars)
        .prepare()
        .attr("aria-hidden", props.visible.then_some("true"))
        .attr("inert", props.visible)
        .render(HtmlTag::Div, props.attributes, props.children)
}
