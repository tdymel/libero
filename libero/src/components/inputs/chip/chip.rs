use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variant,
        a11y::VISUALLY_HIDDEN_SX,
        common::{
            base_color, base_props, contrast_color, contrast_shade_color, focus_ring_sx,
            ring_overlay, ring_overlay_sx, shade_color, variables,
        },
        form::Activation,
        inputs::{VariantColors, VariantVars, interactive_variant_sx, variant_colors},
        layout::use_box,
        navigation::InternalAnchor,
    },
    hooks::{use_cache, use_element, use_id, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ChipDefaults, ColorShade, CssVar, Size, SizeCss},
    utils::warn,
};

const CHIP_COLOR_VAR: CssVar = CssVar::new("--lsx-chip-color");
const CHIP_CONTRAST_VAR: CssVar = CssVar::new("--lsx-chip-contrast");
const CHIP_HOVER_VAR: CssVar = CssVar::new("--lsx-chip-hover");
const CHIP_CONTAINER_VAR: CssVar = CssVar::new("--lsx-chip-container");
const CHIP_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-chip-on-container");

const CHIP_VARS: VariantVars<'static> = VariantVars {
    color: &CHIP_COLOR_VAR,
    contrast: &CHIP_CONTRAST_VAR,
    container: &CHIP_CONTAINER_VAR,
    on_container: &CHIP_ON_CONTAINER_VAR,
};

static CHIP_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = ChipDefaults::theme_vars()
        // The containing block of a selectable chip's hidden checkbox, which
        // would otherwise be laid out against the viewport - see
        // `SegmentedControl`'s root.
        .position("relative")
        .display("inline-flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .border_style("solid")
        .border_width("1px")
        .font_weight("500")
        .white_space("nowrap")
        .user_select("none")
        .max_width("100%")
        // A `<button>` root inherits neither, and a chip has to look the same
        // whichever tag it lands on.
        .font_family("inherit")
        .text_decoration("none");

    Variant::ALL
        .iter()
        .fold(base, |base, &variant| {
            base.when(
                variant.state_name(),
                interactive_variant_sx(variant, &CHIP_VARS, &CHIP_HOVER_VAR),
            )
        })
        // Folded after the variants, which is what makes it win: equal
        // specificity, so source order decides. `variant` describes the
        // unselected look; selected is M3's secondary-container - a tint,
        // which also drops the outline the way the spec asks, since `Tonal`
        // sets `border-color: transparent`.
        .when(
            "checked",
            interactive_variant_sx(Variant::Tonal, &CHIP_VARS, &CHIP_HOVER_VAR),
        )
        .when("clickable", sx().cursor("pointer"))
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
        // The root is not focusable; the visually hidden checkbox inside it
        // is, and the overlay after its label draws the root's ring. Scoped
        // to that child so the delete button's own ring - every `Box` gets
        // one - does not draw a second one out here. Out by the border, as an
        // outline on the root would sit.
        .selector("& > [data-ring]", ring_overlay_sx().inset("-1px"))
        .selector("& > input:focus-visible ~ [data-ring]", focus_ring_sx())
        // The `<button>`/`<a>` root focuses itself.
        .focus_visible(focus_ring_sx())
});

static CHIP_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .cursor("pointer")
        .overflow("hidden")
        .text_overflow("ellipsis")
});

/// Depends on `(variant, checked, color)` alone - see the `use_cache` below.
fn chip_variables(variant: Variant, checked: bool, base: &ThemeAwareValue) -> String {
    // Selected is a tonal container, whatever the unselected look is - but one
    // step past `Tonal`'s own resting tint, or selecting a chip that is
    // already tonal would produce no visible change at all.
    let colors = if checked {
        VariantColors {
            container: shade_color(base, ColorShade::S2),
            on_container: contrast_shade_color(base, ColorShade::S2),
            hover: shade_color(base, ColorShade::S3),
            selected: None,
        }
    } else {
        variant_colors(variant, base)
    };

    variables()
        .with(CHIP_COLOR_VAR, base.resolve(None))
        .with(
            CHIP_CONTRAST_VAR,
            contrast_color(base).and_then(|color| color.resolve(None)),
        )
        .with(CHIP_HOVER_VAR, colors.hover)
        .with(CHIP_CONTAINER_VAR, colors.container)
        .with(CHIP_ON_CONTAINER_VAR, colors.on_container)
        .render()
}

base_props! {
    pub struct ChipProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// The *unselected* look; a checked chip is always filled.
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius, independent of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        /// Strictly controlled - pair it with `onchange`.
        #[props(default)]
        checked: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// A plain action: renders a `<button>` root.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router-aware link instead. Takes precedence over
        /// `onclick`, which an `<a>` has no use for.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// Text and `Icon` only: a `<label>` hijacks clicks on nested controls.
        children: Element,
    }
}

/// A compact token; `onchange` makes it a real checkbox.
#[component]
pub fn Chip(props: ChipProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or_default();
    let color = base_color(props.color.as_ref());
    let checked = props.checked.unwrap_or(false);
    let disabled = props.disabled.unwrap_or(false);

    let size = props.size.copied_or(theme.chip.size);
    let radius = props.radius.copied_or(theme.chip.radius);

    let selectable = props.onchange.is_some();
    let clickable = props.onclick.is_some() || props.to.as_ref().is_some();

    if selectable && clickable {
        warn("Chip: `onchange` ignores `onclick`/`to` - a checkbox chip is not a button.");
    }
    if props.checked.is_some() && !selectable {
        warn("Chip: `checked` without `onchange` can never change.");
    }
    if selectable && props.checked.is_none() {
        warn("Chip: `onchange` without `checked` can never appear selected.");
    }

    let style = use_cache((variant, checked, color), |(variant, checked, color)| {
        chip_variables(*variant, *checked, color)
    });
    let style = Some(style).filter(|style| !style.is_empty());

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("checked", checked)
        .with("disabled", disabled)
        .with("clickable", clickable && !selectable)
        .into();

    let id = use_id();
    let element = use_element();

    // Every hook above the branch, `prepare()` included - it is the hook, so a
    // chain built inside an `if` is a conditional hook. The unused ones are
    // simply dropped.
    let root = use_box()
        .framework_sx(&CHIP_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(style.clone())
        .prepare();
    let input = use_box().framework_sx(&VISUALLY_HIDDEN_SX).prepare();
    let label = use_box().framework_sx(&CHIP_LABEL_SX).prepare();

    if !selectable {
        // `InternalAnchor` has no `onclick`, so a link chip navigates for real.
        if let Some(to) = props.to.as_ref().cloned() {
            // `<a>` has no native `disabled`: dropping `to` stops navigation.
            if disabled {
                return root
                    .attr("aria-disabled", "true")
                    .attr("tabindex", "-1")
                    .render(HtmlTag::A, props.attributes, props.children);
            }

            return rsx! {
                InternalAnchor {
                    to,
                    target: props.target,
                    class: props.class,
                    sx: props.sx,
                    framework_sx: &CHIP_BASE_SX,
                    states,
                    style,
                    attributes: props.attributes,
                    {props.children}
                }
            };
        }

        if let Some(onclick) = props.onclick {
            // `<button>` defaults to `submit`; `attr_default` still lets a
            // caller ask for one.
            return root
                .event("onclick", move |event: Event<MouseData>| {
                    onclick.call(event)
                })
                .attr("disabled", disabled)
                .attr_default("type", "button")
                .render(HtmlTag::Button, props.attributes, props.children);
        }

        return root.attr("aria-disabled", disabled).render(
            HtmlTag::Span,
            props.attributes,
            props.children,
        );
    }

    let onchange = props.onchange;
    // The label is the chip's own, so it wires the label's half of the
    // activation too - the same fix `use_field` gives `Checkbox`.
    let activation = Activation::new(element, move || {
        if let Some(onchange) = &onchange
            && !disabled
        {
            onchange.call(!checked);
        }
    });
    let input = activation
        .wire(input)
        .attr("type", "checkbox")
        .attr("id", id())
        .attr("checked", checked)
        .attr("disabled", disabled)
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, Vec::new(), ());

    let label = label
        .attr("for", id())
        .event("onclick", activation.label_click())
        .render(HtmlTag::Label, Vec::new(), props.children);

    root.attr("aria-disabled", disabled).render(
        HtmlTag::Span,
        props.attributes,
        vec![input, label, ring_overlay()],
    )
}
