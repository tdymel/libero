use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, variables},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
    theme::{
        SKELETON_ANIMATION, SKELETON_COLOR, SKELETON_DURATION, SKELETON_HEIGHT, SKELETON_RADIUS,
        SKELETON_WIDTH, Size, SkeletonDefaults,
    },
};

const VISIBLE_STATE: &str = "visible";
const ANIMATE_STATE: &str = "animate";
const CIRCLE_STATE: &str = "circle";

/// Where the pulse stops under reduced motion: between its two ends. Stopping
/// on `0.4` reads as a disabled control rather than a placeholder.
const SETTLED_OPACITY: &str = "0.7";

static SKELETON_BASE_SX: StaticSx = StaticSx::new(|| {
    SkeletonDefaults::theme_vars()
        // Unset, a skeleton is the size of its children - the wrapper case
        // needs neither prop.
        .height(SKELETON_HEIGHT.value_or("auto"))
        .width(SKELETON_WIDTH.value_or("100%"))
        .border_radius(SKELETON_RADIUS.value())
        // Its own compositing layer, so the pulse on `::after` does not
        // repaint the subtree underneath.
        .transform("translateZ(0)")
        // A circle with no `height` has no width to copy either: it wraps
        // its children, so it shrinks to them instead of spanning the row.
        .when(
            CIRCLE_STATE,
            sx().var(SKELETON_RADIUS, "1000px")
                .width(SKELETON_WIDTH.value_or("fit-content")),
        )
        .when(
            VISIBLE_STATE,
            // The children are hidden, not covered: every descendant inherits
            // `visibility: hidden`, so it keeps its layout but is not painted,
            // whatever its z-index, and whatever the surface behind it. Only
            // the grey opts back in. A descendant that sets `visibility:
            // visible` itself would show through - the one hole.
            sx().position("relative")
                .overflow("hidden")
                .visibility("hidden")
                .selector(
                    "&::after",
                    sx().content("\"\"")
                        .position("absolute")
                        .inset("0")
                        .visibility("visible")
                        .background(SKELETON_COLOR.value()),
                ),
        )
        // Only `::after` pulses, never the root: the root's opacity would fade
        // the children as well once the skeleton is no longer visible. Without
        // `visible` there is no `::after` to animate.
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

/// `circle` makes the width the height, so one number draws a round avatar
/// placeholder. Without a height the width stays unset, and the circle is as
/// wide as its children.
fn skeleton_variables(height: Option<String>, width: Option<String>, circle: bool) -> Variables {
    let width = if circle { height.clone() } else { width };
    variables()
        .with(SKELETON_HEIGHT, height)
        .with(SKELETON_WIDTH, width)
}

base_props! {
    pub struct SkeletonProps {
        /// Cover the children, or draw the standalone shape. `false` shows the
        /// children as they are.
        #[props(default = true)]
        visible: bool,
        /// A CSS length. Unset, the height of the children.
        #[props(default, into)]
        height: Input<ThemeAwareValue>,
        /// A CSS length. Unset, `100%`. Ignored when `circle`.
        #[props(default, into)]
        width: Input<ThemeAwareValue>,
        /// Width equals `height`, corners fully round. Without `height`, as wide
        /// as the children.
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
/// Two ways to use it. With `height`/`width` and no children it is a grey
/// shape, and a layout of them stands in for the real one. Wrapped around the
/// real content, it covers that content while `visible` and gets out of the
/// way when `visible` turns `false` - the layout is written once, and the
/// placeholder is exactly its size because it *is* the content underneath.
///
/// While `visible` the root carries `aria-hidden="true"` and `inert`: covered
/// content is neither announced nor reachable with Tab. Announcing the wait is
/// the caller's region's job - `aria-busy="true"` on it, the same rule as
/// `Loader`.
///
/// The children are hidden with `visibility: hidden`, so the grey is right on
/// any surface. A descendant that sets `visibility: visible` on itself shows
/// through - do not do that under a skeleton.
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
