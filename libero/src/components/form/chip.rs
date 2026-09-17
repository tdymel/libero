use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States, Variant,
        accessibility::VISUALLY_HIDDEN_SX,
        buttons::{VariantColors, VariantVars, interactive_variant_sx, variant_colors},
        common::{
            base_color, base_props, contrast_color, contrast_shade_color, disabled_look_sx,
            fill_color, focus_ring_sx, on_ring_sx, on_state_sx, ring_overlay, ring_overlay_sx,
            shade_color, text_color, variables,
        },
        form::{Activation, use_bound},
        layout::use_box,
        navigation::{InternalAnchor, NewTabHint, wants_new_tab_hint},
    },
    hooks::{ElementHandle, use_cache, use_css, use_element, use_id, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ChipDefaults, ColorShade, CssVar, Size, SizeCss},
    utils::warn,
};

const CHIP_COLOR_VAR: CssVar = CssVar::new("--lsx-chip-color");
const CHIP_FILL_VAR: CssVar = CssVar::new("--lsx-chip-fill");
const CHIP_CONTRAST_VAR: CssVar = CssVar::new("--lsx-chip-contrast");
const CHIP_HOVER_VAR: CssVar = CssVar::new("--lsx-chip-hover");
const CHIP_CONTAINER_VAR: CssVar = CssVar::new("--lsx-chip-container");
const CHIP_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-chip-on-container");
const CHIP_ON_STATE_VAR: CssVar = CssVar::new("--lsx-chip-on-state");

const CHIP_VARS: VariantVars<'static> = VariantVars {
    color: &CHIP_COLOR_VAR,
    fill: &CHIP_FILL_VAR,
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
        // A long label is cut at the pill's edge instead of running past it
        // (todo 358). Not on a selectable chip: its label clips itself, and the
        // ring overlay drawn out by the border would be clipped with it.
        .overflow("hidden")
        // A `<button>` root inherits neither, and a chip has to look the same
        // whichever tag it lands on.
        .font_family("inherit")
        .text_decoration("none")
        // The `icon` slot never shrinks, on the root or in a checkbox chip's label.
        .selector("& > [data-slot='chip-icon']", chip_icon_sx())
        .selector("& > label > [data-slot='chip-icon']", chip_icon_sx());

    Variant::ALL
        .iter()
        .fold(base, |base, &variant| {
            base.when(
                variant.state_name(),
                interactive_variant_sx(variant, &CHIP_VARS, &CHIP_HOVER_VAR, &CHIP_ON_STATE_VAR),
            )
        })
        // Folded after the variants, which is what makes it win: equal
        // specificity, so source order decides. `variant` describes the
        // unselected look; selected is M3's secondary-container - a tint,
        // which also drops the outline the way the spec asks, since `Tonal`
        // sets `border-color: transparent`.
        .when(
            "checked",
            interactive_variant_sx(
                Variant::Tonal,
                &CHIP_VARS,
                &CHIP_HOVER_VAR,
                &CHIP_ON_STATE_VAR,
            )
            .and(on_state_sx(None))
            // An `Elevated` chip's hover lift would drop the ring otherwise.
            .selector(
                "&:hover:not(:where(:disabled, [data-state~=\"disabled\"]))",
                on_ring_sx(None),
            ),
        )
        .when("selectable", sx().overflow("visible"))
        .when("clickable", sx().cursor("pointer"))
        .when("disabled", disabled_sx())
        // A button chip ignores a disabled `Fieldset`, but the browser still
        // disables its `<button>` (todo 499).
        .selector("&:disabled", disabled_sx())
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

fn chip_icon_sx() -> Sx {
    sx().display("inline-flex")
        .align_items("center")
        .flex("0 0 auto")
}

// No `pointer-events: none`, so `not-allowed` shows (todo 596): the variant's
// `:hover` skips a disabled chip, and no click path is left to block.
fn disabled_sx() -> Sx {
    disabled_look_sx("not-allowed").selector("& > label", sx().cursor("not-allowed"))
}

static CHIP_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .cursor("pointer")
        // No `text-overflow`: on a flex root it never applies (todo 94), and
        // the children stay unwrapped (todo 674).
        .overflow("hidden")
        // The whole pill is the hit area, not only the text: stretched over
        // the positioned root, and not clipped by the label's own overflow.
        .selector(
            "&::after",
            sx().content("\"\"").position("absolute").inset("0"),
        )
});

/// After the label, never shrinking, so a long label cannot clip a remove x.
/// Lifted over a selectable chip's label `::after`, which would take its clicks.
static CHIP_TRAILING_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .flex("0 0 auto")
        .position("relative")
        .z_index("1")
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
            on_state: None,
        }
    } else {
        variant_colors(variant, base)
    };

    variables()
        .with(CHIP_COLOR_VAR, text_color(base))
        .with(CHIP_FILL_VAR, fill_color(base))
        .with(
            CHIP_CONTRAST_VAR,
            contrast_color(base).and_then(|color| color.resolve(None)),
        )
        .with(CHIP_HOVER_VAR, colors.hover)
        .with(CHIP_ON_STATE_VAR, colors.on_state)
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
        /// Pair it with `onchange`. Left out, a chip with a `name` keeps its
        /// own state unless that name binds it to the form around it.
        #[props(default)]
        checked: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// A checkbox chip stays focusable and posted with the form, but
        /// clicks and Space no longer toggle it, as on `Checkbox`. `None` is
        /// "not stated".
        #[props(default)]
        readonly: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// Makes the chip a checkbox that posts under this name. A path -
        /// `Filters::FIELDS.open()` - also binds it to the surrounding
        /// `Form`'s value when the chip has no `onchange`, as on `Checkbox`.
        #[props(default, into)]
        name: crate::components::FieldName<bool>,
        /// What the chip posts under its `name` when it is checked, so a row
        /// of filter chips can share one name: `Chip { name: "tags", value:
        /// "rust" }` posts `tags=rust`. Left out, it posts the browser's
        /// `name=on`, as `Checkbox` does.
        #[props(default, into)]
        value: Option<String>,
        /// A plain action: renders a `<button>` root.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router-aware link instead. Takes precedence over
        /// `onclick`, which an `<a>` has no use for.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// As on `Anchor`: with `to` and `target: "_blank"`, an external icon
        /// plus a hidden "(opens in a new tab)". `false` drops both.
        #[props(default = true)]
        new_tab_hint: bool,
        /// Drawn before the label, with a gap; it never shrinks.
        #[props(default)]
        icon: Option<Element>,
        /// Drawn after the label, with a gap; it never shrinks - a remove x.
        /// Outside a checkbox chip's `<label>`, so it may be a button - but not
        /// on an `onclick` or `to` chip, whose root is one already.
        #[props(default)]
        trailing: Option<Element>,
        /// The label, laid out as the chip's own flex items. Text and `Icon`
        /// only: a `<label>` hijacks clicks on nested controls.
        children: Element,
    }
}

/// A compact token; `onchange` makes it a real checkbox.
///
/// An icon goes in `icon` and a remove x in `trailing`; both keep their gap
/// and never shrink: `Chip { icon: rsx! { MyIcon {} }, "rust" }`. A long label
/// is cut at the edge; for an ellipsis, give the text a span of its own:
/// `span { style: "min-width: 0; overflow: hidden; text-overflow: ellipsis", "{label}" }`.
#[component]
pub fn Chip(props: ChipProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or(theme.chip.variant);
    let color = base_color(props.color.as_ref());
    let bound = use_bound(&props.name, props.onchange.is_some());
    let checked = bound
        .value()
        .or(props.checked)
        .or(bound.entered())
        .unwrap_or(false);

    let size = props.size.copied_or(theme.chip.size);
    let radius = props.radius.copied_or(theme.chip.radius);

    // A `name` alone makes it a checkbox too: it has something to post.
    let selectable = props.onchange.is_some() || !props.name.is_empty();
    // A disabled `Fieldset` around a checkbox chip counts, as for every
    // field. A plain or button chip keeps only its own `disabled`.
    let disabled = match selectable {
        true => bound.disabled(props.disabled),
        false => props.disabled.unwrap_or(false),
    };
    let clickable = props.onclick.is_some() || props.to.as_ref().is_some();

    if selectable && clickable {
        warn("Chip: `onchange` ignores `onclick`/`to` - a checkbox chip is not a button.");
    }
    if props.checked.is_some() && props.onchange.is_none() && !bound.is_bound() {
        warn("Chip: `checked` without `onchange` can never change.");
    }
    if props.onchange.is_some() && props.checked.is_none() {
        warn("Chip: `onchange` without `checked` can never appear selected.");
    }
    if props.value.is_some() && props.name.is_empty() {
        warn("Chip: `value` without `name` is posted by nothing.");
    }
    if props.readonly.is_some() && !selectable {
        warn("Chip: `readonly` without `onchange` or `name` has nothing to lock.");
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
        .with("selectable", selectable)
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
    let trailing_class = use_css(Some(&CHIP_TRAILING_SX), CssLayer::Framework);

    // Children unwrapped, so an icon among them keeps the gap and centring.
    let labelled = rsx! {
        if let Some(icon) = props.icon {
            span { "data-slot": "chip-icon", {icon} }
        }
        {props.children}
    };
    let trailing_slot = use_element();
    use_nested_control_warning(
        trailing_slot,
        clickable && !selectable && props.trailing.is_some(),
    );
    let trailing = rsx! {
        if let Some(trailing) = props.trailing {
            // Not `trailing`: a field frame's own slot is, and a chip sits in one.
            span {
                class: trailing_class,
                "data-slot": "chip-trailing",
                onmounted: trailing_slot.mount(),
                {trailing}
            }
        }
    };
    let content = rsx! {
        {labelled.clone()}
        {trailing.clone()}
    };

    if !selectable {
        // `InternalAnchor` has no `onclick`, so a link chip navigates for real.
        if let Some(to) = props.to.as_ref().cloned() {
            // `<a>` has no native `disabled`: dropping `to` stops navigation.
            // Without `href` it is `generic`, so the role comes back by hand.
            if disabled {
                return root
                    .attr_default("role", "link")
                    .attr("aria-disabled", "true")
                    .attr("tabindex", "-1")
                    .render(HtmlTag::A, props.attributes, content);
            }

            // After the children, so a caller's ellipsis span never cuts it.
            let hint = wants_new_tab_hint(props.target.as_deref(), props.new_tab_hint);
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
                    {labelled}
                    if hint {
                        NewTabHint {}
                    }
                    {trailing}
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
                .render(HtmlTag::Button, props.attributes, content);
        }

        return root.attr("aria-disabled", disabled).render(
            HtmlTag::Span,
            props.attributes,
            content,
        );
    }

    let onchange = bound.emit(props.onchange);
    // Refused here and said with `aria-readonly`, as on `Checkbox`: HTML's
    // `readonly` does not apply to a checkbox.
    let readonly = props.readonly.unwrap_or(false);
    // The label is the chip's own, so it wires the label's half of the
    // activation too - the same fix `use_field` gives `Checkbox`.
    let activation = Activation::new(element, move || {
        if let Some(onchange) = &onchange
            && !disabled
            && !readonly
        {
            onchange(!checked);
        }
    });
    let input = activation
        .wire(input)
        .attr("type", "checkbox")
        .attr("id", id())
        .attr("name", bound.name().map(str::to_string))
        // No attribute is how the `on` fallback is kept: the browser's own
        // default for a valueless checkbox, exactly as `Checkbox` posts.
        .attr("value", props.value)
        .attr("checked", checked)
        .attr("disabled", disabled)
        .attr("aria-readonly", readonly.then_some("true"))
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, Vec::new(), ());

    let label = label
        .attr("for", id())
        .event("onclick", activation.label_click())
        .render(HtmlTag::Label, Vec::new(), labelled);

    // The trailing slot sits beside the label, not in it: a `<label>` would
    // take a button's click for the checkbox.
    root.attr("aria-disabled", disabled).render(
        HtmlTag::Span,
        props.attributes,
        vec![input, label, trailing, ring_overlay()],
    )
}

/// What an `onclick`/`to` chip's `trailing` may not hold: a control of its own.
#[cfg(debug_assertions)]
const NESTED_CONTROL: &str = "a[href], button, input, select, textarea, [tabindex]";

/// Debug builds: an `onclick`/`to` chip's `trailing` lands inside its
/// `<button>`/`<a>`, so a control there is nested interactive content (todo
/// 661). Read off the DOM: a badge there is fine, a button is not.
fn use_nested_control_warning(slot: ElementHandle, nested: bool) {
    #[cfg(debug_assertions)]
    use_effect(use_reactive!(|nested| {
        let _ = slot.mount_token();
        if nested && crate::platform::ElementApi::query_selector(&slot, NESTED_CONTROL).is_ok() {
            warn(
                "Chip: `trailing` holds a control inside an `onclick`/`to` chip's \
                 button or link - nested interactive content. Drop `onclick`/`to`, or \
                 the control.",
            );
        }
    }));
    #[cfg(not(debug_assertions))]
    let _ = (slot, nested);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The label is a flex root, where `text-overflow` never applies. The
    /// declaration promised an ellipsis the chip never drew (todo 94).
    #[test]
    fn the_label_promises_no_ellipsis_it_cannot_draw() {
        let css = Stylesheet::from(&CHIP_LABEL_SX);
        assert!(!css.as_str().contains("text-overflow"), "{}", css.as_str());
        assert!(css.as_str().contains("overflow:hidden"), "{}", css.as_str());
    }

    /// A plain chip's root clips, or a long label runs past the pill (todo
    /// 358). A selectable one does not: its label clips, and the root has to
    /// let the ring overlay out by the border.
    #[test]
    fn a_plain_chip_clips_and_a_selectable_one_lets_its_ring_out() {
        let css = Stylesheet::from(&CHIP_BASE_SX);
        let css = css.as_str();
        let selectable = css
            .find("[data-state~=\"selectable\"]")
            .unwrap_or_else(|| panic!("no selectable arm: {css}"));
        assert!(css[..selectable].contains("overflow:hidden"), "{css}");
        assert!(css[selectable..].contains("overflow:visible"), "{css}");
    }
}
