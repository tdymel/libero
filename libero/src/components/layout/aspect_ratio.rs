use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, REPLACED_ELEMENTS, Variables, base_props, inset_focus_ring_sx,
            inset_outline_ring_sx, ring_overlay_sx, variables,
        },
        layout::use_box,
    },
    sx::{StaticSx, Sx, sx},
    theme::{ASPECT_RATIO, FOCUS_RING_WIDTH},
    utils::warn,
};

static ASPECT_RATIO_BASE_SX: StaticSx = StaticSx::new(|| {
    // Relative so a static child's overlay anchors here, never moving a positioned child (todo 2535).
    sx().aspect_ratio(ASPECT_RATIO.overridable())
        .position("relative")
        .overflow("hidden")
        // `object-fit` crops a replaced child instead of stretching it (todo 2533).
        .selector(
            "& > *",
            sx().width("100%").height("100%").object_fit("cover"),
        )
        // An inline child ignores the 100% size (todo 2534).
        .selector("& > :where(a, picture, span)", sx().display("block"))
        // The child fills the clipped box, so a ring drawn outside it is cut away.
        // Doubled to outrank a `Button`'s own ring, which ties it otherwise.
        .selector("& > *:focus-visible:focus-visible", inset_ring())
        // Replaced content covers inset shadows and takes no `::after` (todo 2532).
        .selector(
            format!("& > :is({REPLACED_ELEMENTS}):focus-visible:focus-visible"),
            inset_outline_ring_sx(&ring_offset()),
        )
        // A picture inside the child paints over its inset shadows; the overlay paints over it (todo 2480).
        .selector(
            "& > *:focus-visible::after",
            ring_overlay_sx().content("\"\"").and(inset_ring()),
        )
});

fn ring_offset() -> String {
    format!("calc(-1 * {})", FOCUS_RING_WIDTH.value())
}

fn inset_ring() -> Sx {
    inset_focus_ring_sx(&ring_offset())
}

fn aspect_ratio_variables(ratio: Option<&f32>) -> Variables {
    // CSS drops a ratio that is not positive and finite; the box falls back to `auto`.
    if let Some(ratio) = ratio.filter(|ratio| !(ratio.is_finite() && **ratio > 0.0)) {
        warn(&format!(
            "AspectRatio: ratio {ratio} is not a positive finite number; the browser ignores it."
        ));
    }
    variables().with(ASPECT_RATIO.override_var(), ratio.map(f32::to_string))
}

base_props! {
    pub struct AspectRatioProps {
        /// Width-to-height ratio, e.g. `16.0 / 9.0`.
        #[props(default, into)]
        ratio: Input<f32>,
        children: Element,
    }
}

/// Holds its child to a fixed width-to-height ratio.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::AspectRatio;
/// # fn app() -> Element {
/// rsx! {
///     AspectRatio { ratio: 16.0 / 9.0,
///         img { src: "/cover.jpg", alt: "Cover" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/aspect-ratio>
#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let variables: Input<Variables> = aspect_ratio_variables(props.ratio.as_ref()).into();

    use_box()
        .framework_sx(&ASPECT_RATIO_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .render(HtmlTag::Div, props.attributes, props.children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::take_warnings;

    /// Sets the `-override` twin: the base var is the theme default this has
    /// to beat.
    #[test]
    fn a_ratio_sets_the_override_variable() {
        let variables = aspect_ratio_variables(Some(&1.5));

        assert_eq!(
            variables.to_string(),
            format!("{}:1.5;", ASPECT_RATIO.override_var().name())
        );
    }

    #[test]
    fn no_ratio_emits_no_variable() {
        assert_eq!(aspect_ratio_variables(None).to_string(), "");
    }

    #[test]
    fn a_ratio_css_rejects_warns() {
        for ratio in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            take_warnings();
            aspect_ratio_variables(Some(&ratio));
            assert_eq!(take_warnings().len(), 1, "ratio {ratio}");
        }
        aspect_ratio_variables(Some(&1.5));
        aspect_ratio_variables(None);
        assert!(take_warnings().is_empty());
    }

    /// Todo 637: `.x > *:focus-visible` ties a `Button`'s `.y:focus-visible`
    /// at 0-2-0; the doubled pseudo-class wins whatever the order.
    #[test]
    fn the_inset_ring_outranks_a_childs_own_ring() {
        let sheet = crate::css::Stylesheet::from(&ASPECT_RATIO_BASE_SX);

        assert!(
            sheet.as_str().contains(" > *:focus-visible:focus-visible{"),
            "{}",
            sheet.as_str()
        );
    }

    /// Todo 2480: a linked picture's stripe rides an overlay over the picture.
    #[test]
    fn a_focused_child_paints_its_ring_over_its_picture() {
        let sheet = crate::css::Stylesheet::from(&ASPECT_RATIO_BASE_SX);
        let css = sheet.as_str();
        let overlay = css
            .split(" > *:focus-visible::after{")
            .nth(1)
            .and_then(|rule| rule.split('}').next())
            .unwrap_or_else(|| panic!("no overlay: {css}"));
        assert!(overlay.contains("box-shadow:inset"), "{overlay}");
    }

    fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
        css.split(selector)
            .nth(1)
            .and_then(|rule| rule.split('}').next())
            .unwrap_or_else(|| panic!("no `{selector}` rule: {css}"))
    }

    /// Todo 2532: a `video` paints over inset shadows, so its stripe is a painted outline.
    #[test]
    fn a_focused_replaced_child_draws_a_painted_outline() {
        let sheet = crate::css::Stylesheet::from(&ASPECT_RATIO_BASE_SX);
        let replaced = rule(sheet.as_str(), "video, iframe");

        assert!(!replaced.contains("solid transparent"), "{replaced}");
        assert!(
            replaced.contains("outline-offset:calc(-1 * var(--lsx-focus-ring-width))"),
            "{replaced}"
        );
    }

    /// Todos 2533, 2534: a replaced child is cropped, an inline one takes the full size.
    #[test]
    fn the_child_is_cropped_and_an_inline_one_made_a_block() {
        let sheet = crate::css::Stylesheet::from(&ASPECT_RATIO_BASE_SX);
        let css = sheet.as_str();

        assert!(rule(css, " > *{").contains("object-fit:cover"), "{css}");
        assert!(
            rule(css, " > :where(a, picture, span){").contains("display:block"),
            "{css}"
        );
    }

    /// Todo 2535: the overlay anchors on the root, so a positioned child keeps its place.
    #[test]
    fn focus_leaves_a_childs_position_alone() {
        let sheet = crate::css::Stylesheet::from(&ASPECT_RATIO_BASE_SX);
        let css = sheet.as_str();

        let focused = rule(css, " > *:focus-visible:focus-visible{");
        assert!(!focused.contains("position"), "{focused}");
        assert!(css.contains("position:relative"), "{css}");
    }
}
