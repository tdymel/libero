use std::cell::Cell;
use std::rc::Rc;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    CssLayer,
    components::{
        accessibility::VISUALLY_HIDDEN_SX,
        common::{
            ACTIVE, COARSE_POINTER, Glyph, HtmlTag, Input, Part, ScaleOrCss, States, Variant,
            VariantColors, VariantVars, base_color_or, base_props, coarse_hit_area_sx,
            contrast_color, contrast_shade_color, disabled_look_sx, fill_color, focus_ring_sx,
            interactive_variant_sx, on_ring_sx, on_state_sx, parts_enum, parts_under_sx,
            ring_overlay, ring_overlay_sx, shade_color, text_color, variables, variant_colors,
        },
        form::{Activation, use_bound},
        layout::{InternalAnchor, use_box},
        navigation::{NewTabHint, wants_new_tab_hint},
    },
    context::IconSlot,
    hooks::{ElementHandle, use_cache, use_css, use_element, use_id, use_theme},
    platform::reads_click_targets,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ChipDefaults, ColorShade, CssVar, Size, SizeCss},
    utils::warn,
};

const CHIP_COLOR_VAR: CssVar = CssVar::new("--lsx-chip-color");
const CHIP_FILL_VAR: CssVar = CssVar::new("--lsx-chip-fill");
const CHIP_CONTRAST_VAR: CssVar = CssVar::new("--lsx-chip-contrast");
const CHIP_HOVER_VAR: CssVar = CssVar::new("--lsx-chip-hover");
const CHIP_SELECTED_VAR: CssVar = CssVar::new("--lsx-chip-selected");
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
        // Contains the hidden checkbox, which would otherwise lay out against the viewport.
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
        // Cuts a long label (todo 358). Not on a selectable chip: its label clips
        // itself, and the ring overlay would be clipped too.
        .overflow("hidden")
        // A `<button>` root inherits neither, and a chip has to look the same
        // whichever tag it lands on.
        .font_family("inherit")
        .letter_spacing("inherit")
        .text_decoration("none")
        // The `icon` slot never shrinks, on the root or in a checkbox chip's label.
        .selector(ChipPart::Icon.selector(), chip_icon_sx())
        .selector(CHECK, chip_icon_sx())
        .selector(
            format!("{CHECK} > svg"),
            sx().width("1.125em").height("1.125em"),
        );

    Variant::ALL
        .iter()
        .fold(base, |base, &variant| {
            base.when(
                variant.state_name(),
                interactive_variant_sx(
                    variant,
                    &CHIP_VARS,
                    &CHIP_HOVER_VAR,
                    &CHIP_SELECTED_VAR,
                    &CHIP_ON_STATE_VAR,
                ),
            )
        })
        // After the variants so it wins on source order. Selected is M3's
        // secondary-container tint; `Tonal` also drops the outline.
        .when(
            "checked",
            interactive_variant_sx(
                Variant::Tonal,
                &CHIP_VARS,
                &CHIP_HOVER_VAR,
                &CHIP_SELECTED_VAR,
                &CHIP_ON_STATE_VAR,
            )
            .and(on_state_sx(None))
            // An `Elevated` chip's hover lift and press would drop the ring otherwise.
            .selector(
                "&:hover:not(:where(:disabled, [data-state~=\"disabled\"]))",
                on_ring_sx(None),
            )
            .selector(ACTIVE, on_ring_sx(None)),
        )
        .when("selectable", sx().overflow("visible"))
        // A finger-sized hit area; `clip` cuts a long label across only, so it reaches out.
        .when(
            "clickable",
            sx().cursor("pointer")
                .and(coarse_hit_area_sx("::before"))
                .media(
                    COARSE_POINTER,
                    sx().overflow_x("clip").overflow_y("visible"),
                ),
        )
        .when("disabled", disabled_sx())
        // A button chip ignores a disabled `Fieldset`, but the browser still
        // disables its `<button>` (todo 499).
        .selector("&:disabled", disabled_sx())
        // The hidden checkbox has the focus; the overlay draws the root's ring.
        // Scoped to that child so the delete button's ring is not doubled.
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
        .and(coarse_hit_area_sx("&::after"))
});

/// The check a selected chip draws before its label (`ChipDefaults::selected_check`).
const CHECK: &str = "& > label > [data-slot='chip-check']";

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
    // One step past `Tonal`'s resting tint, or selecting a tonal chip would
    // change nothing visible.
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
        .with(CHIP_SELECTED_VAR, colors.selected)
        .with(CHIP_ON_STATE_VAR, colors.on_state)
        .with(CHIP_CONTAINER_VAR, colors.container)
        .with(CHIP_ON_CONTAINER_VAR, colors.on_container)
        .render()
}

parts_enum! {
    /// [`Chip`]'s inner parts, for its `parts` prop. Each is a child of the
    /// root or of a checkbox chip's label, so a chip in `trailing` keeps its own.
    pub enum ChipPart {
        /// The leading glyph's wrapper.
        Icon = "chip-icon" => "& > [data-slot='chip-icon'], & > label > [data-slot='chip-icon']",
        /// The slot after the label, with `trailing`.
        Trailing = "chip-trailing" => "& > [data-slot='chip-trailing']",
        /// The new-tab icon after the label, on a `to` chip with `target: "_blank"` only.
        NewTab = "new-tab" => "& > [data-slot='new-tab']",
    }
}

base_props! {
    parts(ChipPart);
    pub struct ChipProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// The *unselected* look; a checked chip is always filled.
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        size: Input<Size>,
        /// Corner radius, independent of `size`: a size word or any CSS, as `radius: "0"`.
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Pair it with `onchange`; left out, a named chip keeps its own state.
        #[props(default)]
        checked: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// A checkbox chip stays focusable and posted, but does not toggle.
        #[props(default)]
        readonly: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// Makes the chip a checkbox that posts under this name. A path also
        /// binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<bool>,
        /// What a checked chip posts under `name`; defaults to `on`.
        #[props(default, into)]
        value: Option<String>,
        /// A plain action: renders a `<button>` root.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router-aware link; takes precedence over `onclick`.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// As on `Anchor`: the new-tab icon and hidden hint. `false` drops both.
        #[props(default = true)]
        new_tab_hint: bool,
        /// Drawn before the label; it never shrinks.
        #[props(default)]
        icon: Option<Element>,
        /// Drawn after the label, e.g. a remove x. No button on an `onclick` or `to` chip.
        #[props(default)]
        trailing: Option<Element>,
        /// The label. Text and `Icon` only: a `<label>` hijacks nested clicks.
        /// An icon-only chip is named by its `Icon`'s `aria_label`.
        children: Element,
    }
}

/// A compact token; `onchange` makes it a real checkbox.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Chip;
/// # fn app() -> Element {
/// let mut open = use_signal(|| false);
/// rsx! {
///     Chip { checked: open(), onchange: move |next| open.set(next), "Open only" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/chip>
#[component]
pub fn Chip(props: ChipProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or(theme.chip.variant);
    let color = base_color_or(props.color.as_ref(), theme.chip.color);
    let bound = use_bound(&props.name, props.onchange.is_some());
    let checked = bound
        .value()
        .or(props.checked)
        .or(bound.entered())
        .unwrap_or(false);

    let size = props.size.copied_or(theme.chip.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.chip.radius);

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
    if props.readonly == Some(true) && !selectable {
        warn("Chip: `readonly` without `onchange` or `name` has nothing to lock.");
    }

    let style = use_cache((variant, checked, color), |(variant, checked, color)| {
        chip_variables(*variant, *checked, color)
    });
    let style = match radius.custom_css(SizeCss::RADIUS) {
        Some(css) => format!("{style}{}:{css};", SizeCss::RADIUS.override_var().name()),
        None => style,
    };
    let style = Some(style).filter(|style| !style.is_empty());

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(variant.state_name(), true)
        .with(size.state_name(), true)
        .with(radius.size.radius_state_name(), true)
        .with("checked", checked)
        .with("selectable", selectable)
        .with("disabled", disabled)
        .with("clickable", clickable && !selectable)
        .into();

    let id = use_id();
    let element = use_element();

    // Every hook above the branch: `prepare()` inside an `if` is a conditional hook.
    let root = use_box()
        .framework_sx(&CHIP_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(style.clone())
        .prepare();
    let input = use_box().framework_sx(&VISUALLY_HIDDEN_SX).prepare();
    let label = use_box().framework_sx(&CHIP_LABEL_SX).prepare();
    let trailing_class = use_css(Some(&CHIP_TRAILING_SX), CssLayer::Framework);

    // Hidden: the checkbox's own checked state is what is read out.
    let check = selectable && checked && theme.chip.selected_check;
    // Children unwrapped, so an icon among them keeps the gap and centring.
    let labelled = rsx! {
        if check {
            span { "data-slot": "chip-check", "aria-hidden": "true",
                Glyph { slot: IconSlot::Check, icon: lucide::check::outlined }
            }
        }
        if let Some(icon) = props.icon {
            span { "data-slot": ChipPart::Icon.slot(), {icon} }
        }
        {props.children}
    };
    let trailing_slot = use_element();
    use_nested_control_warning(
        trailing_slot,
        clickable && !selectable && props.trailing.is_some(),
    );
    // Where no click target can be read, a click in the trailing slot counts as
    // its control's, so the padding does not toggle the chip too (993).
    let trailing_hit = use_hook(|| Rc::new(Cell::new(false)));
    let mark_trailing = {
        let trailing_hit = trailing_hit.clone();
        move |_: MouseEvent| trailing_hit.set(selectable && !reads_click_targets())
    };
    let trailing = rsx! {
        if let Some(trailing) = props.trailing {
            // Not `trailing`: a field frame's own slot is, and a chip sits in one.
            span {
                class: trailing_class,
                "data-slot": ChipPart::Trailing.slot(),
                onmounted: trailing_slot.mount(),
                onclick: mark_trailing,
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
                    sx: parts_under_sx(&props.parts, props.sx),
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
    // HTML `readonly` does not apply to a checkbox: refused here, as on `Checkbox`.
    let readonly = props.readonly.unwrap_or(false);
    // The label is the chip's own, so it wires the label's half of the activation too.
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
        // No attribute keeps the browser's `on` default.
        .attr("value", props.value)
        .attr("checked", checked)
        .attr("data-controlled", true)
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
    // The pill's padding, which the label's `::after` covers on the web only.
    let mut padding_click = activation.padding_click("span[data-state~=\"selectable\"]");
    root.attr("aria-disabled", disabled)
        .event("onclick", move |event: Event<MouseData>| {
            if !trailing_hit.replace(false) {
                padding_click(event);
            }
        })
        .render(
            HtmlTag::Span,
            props.attributes,
            vec![input, label, trailing, ring_overlay()],
        )
}

/// What an `onclick`/`to` chip's `trailing` may not hold: a control of its own.
#[cfg(debug_assertions)]
const NESTED_CONTROL: &str = "a[href], button, input, select, textarea, [tabindex]";

/// Debug builds: warns on a control in an `onclick`/`to` chip's `trailing`,
/// nested interactive content (todo 661).
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

    /// The coarse-pointer inset follows the resting one, so it wins on a finger (todo 2707).
    #[test]
    fn a_finger_grows_the_labels_hit_area() {
        let css = Stylesheet::from(&CHIP_LABEL_SX);
        let css = css.as_str();
        let resting = css.find("inset:0").unwrap_or_else(|| panic!("{css}"));
        let coarse = css
            .find("(pointer: coarse)")
            .unwrap_or_else(|| panic!("{css}"));
        assert!(resting < coarse, "{css}");
    }

    /// A plain chip's root clips (todo 358); a selectable one lets the ring out.
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
