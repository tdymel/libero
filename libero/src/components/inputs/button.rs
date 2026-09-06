use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variant,
        common::{
            base_color, base_props, contrast_color, contrast_shade_color, fill_color,
            focus_ring_sx, hover_color, input_from_str, selected_color, shade_color, text_color,
            variables,
        },
        feedback::Loader,
        layout::use_box,
        navigation::InternalAnchor,
    },
    hooks::{ripple_sx, use_cache, use_ripple, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        BUTTON_HEIGHT, ButtonDefaults, ColorShade, CssVar, LOADER_SIZE, PAPER_BACKGROUND,
        PaperDefaults, Size, SizeCss,
    },
    utils::warn,
};

// Optional, since a Button is only a link when set. Costs the direct
// `to: Route::Foo {}` that `Anchor` gets: `#[props(into)]` can't chain a
// foreign conversion through a wrapper, so callers need
// `NavigationTarget::from(..)`.
input_from_str!(NavigationTarget);

impl From<NavigationTarget> for Input<NavigationTarget> {
    fn from(value: NavigationTarget) -> Self {
        Input::Value(value)
    }
}

// What `interactive_variant_sx` references by name.
pub(crate) const BUTTON_COLOR_VAR: CssVar = CssVar::new("--lsx-button-color");
pub(crate) const BUTTON_FILL_VAR: CssVar = CssVar::new("--lsx-button-fill");
pub(crate) const BUTTON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-button-contrast");
pub(crate) const BUTTON_HOVER_VAR: CssVar = CssVar::new("--lsx-button-hover");
pub(crate) const BUTTON_SELECTED_VAR: CssVar = CssVar::new("--lsx-button-selected");
pub(crate) const BUTTON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-button-container");
pub(crate) const BUTTON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-button-on-container");

/// The `var()` names the variant chrome reads. `Chip` owns a parallel set
/// under its own prefix, so it renders the same chrome from its own colour.
pub(crate) struct VariantVars<'a> {
    /// The colour in its **text** role: the label of an unfilled variant, and
    /// the outline that matches that label. Not the brand colour itself -
    /// see [`crate::tokens::ColorValue`] for why the two differ.
    pub color: &'a CssVar,
    /// The colour in its **fill** role, under a `contrast` foreground.
    pub fill: &'a CssVar,
    pub contrast: &'a CssVar,
    pub container: &'a CssVar,
    pub on_container: &'a CssVar,
}

/// `Button`'s own set, shared with `SegmentedControl`.
pub(crate) const BUTTON_VARS: VariantVars<'static> = VariantVars {
    color: &BUTTON_COLOR_VAR,
    fill: &BUTTON_FILL_VAR,
    contrast: &BUTTON_CONTRAST_VAR,
    container: &BUTTON_CONTAINER_VAR,
    on_container: &BUTTON_ON_CONTAINER_VAR,
};

// The lightest tint on the ramp. A darker one cannot be labelled legibly:
// black on `S1` clears 12:1 for every palette colour, on `S2` it is closer.
const TONAL_CONTAINER: ColorShade = ColorShade::S1;

// M3's elevated button rests at level 1 and lifts to level 2 on hover.
const ELEVATED_REST: Size = Size::Xs;
const ELEVATED_HOVER: Size = Size::Sm;

/// Structural chrome for `variant`, in `var()` names rather than resolved
/// values - which is what lets a second component reuse it under its own set.
///
/// The hover response is [`interactive_variant_sx`]'s: `Icon` is a static badge and
/// must not grow one, so it takes the chrome alone.
pub(crate) fn variant_chrome_sx(variant: Variant, vars: &VariantVars) -> Sx {
    let VariantVars {
        color,
        fill,
        contrast,
        container,
        on_container,
    } = vars;
    // A literal colour publishes neither role, so both fall back to the
    // colour var, which holds the literal itself.
    let fill = fill.value_or(color.value());

    match variant {
        Variant::Filled => sx()
            // The rim is the fill, not the accent: a filled control whose
            // border sat one ramp step lighter would show a bright hairline.
            .background(fill.clone())
            .border_color(fill)
            .color(contrast.value_or("inherit")),
        // M3's secondary-container pairing: a light tint of the colour under
        // the label that reads on it. A literal colour has no ramp, so every
        // fallback here lands back on the filled look.
        Variant::Tonal => sx()
            .background(container.value_or(color.value()))
            .border_color("transparent")
            .color(on_container.value_or(contrast.value_or("inherit"))),
        // The *surface*, not a tint of the colour - M3's elevated button is
        // separated from the page by its shadow alone, and the label carries
        // the accent. An opaque background is what the shadow needs to sit on.
        // The shadow stays M3's own pair of steps rather than Paper's resting
        // one: the hover lift is a fixed step above it.
        Variant::Elevated => sx()
            .and(PaperDefaults::background_sx())
            .border_color("transparent")
            .color(color.value())
            .box_shadow(SizeCss::SHADOW.value(ELEVATED_REST)),
        Variant::Outlined => sx()
            .background("transparent")
            .border_color(color.value())
            .color(color.value()),
        Variant::Standard => sx()
            .background("transparent")
            .border_color("transparent")
            .color(color.value()),
    }
}

/// [`variant_chrome_sx`] plus the hover response an interactive control needs:
/// a filled or tinted variant darkens, an unfilled one tints, and `Elevated`
/// lifts a level rather than changing its fill.
pub(crate) fn interactive_variant_sx(variant: Variant, vars: &VariantVars, hover: &CssVar) -> Sx {
    let color = vars.color;

    let fallback = match variant {
        Variant::Filled | Variant::Tonal => vars.fill.value_or(color.value()),
        Variant::Elevated => PAPER_BACKGROUND.value(),
        Variant::Outlined | Variant::Standard => "transparent".to_string(),
    };

    let mut hovered = sx().background(hover.value_or(fallback));
    if variant == Variant::Elevated {
        hovered = hovered.box_shadow(SizeCss::SHADOW.value(ELEVATED_HOVER));
    }

    variant_chrome_sx(variant, vars).hover(hovered)
}

/// Resolved values for the colour vars `variant` reads, from the caller's
/// `color`. `None` wherever a literal colour has no shade ramp to walk.
pub(crate) struct VariantColors {
    pub container: Option<String>,
    pub on_container: Option<String>,
    pub hover: Option<String>,
    pub selected: Option<String>,
}

/// Which shades a variant tints with. The tinted variants pin absolute
/// shades rather than stepping from `base`, the same way the hover tint
/// always has.
pub(crate) fn variant_colors(variant: Variant, base: &ThemeAwareValue) -> VariantColors {
    let filled = variant == Variant::Filled;
    // Only `Tonal` paints a container; `Elevated` sits on the surface itself.
    let (container, on_container) = match variant {
        Variant::Tonal => (
            shade_color(base, TONAL_CONTAINER),
            contrast_shade_color(base, TONAL_CONTAINER),
        ),
        _ => (None, None),
    };

    let (hover, selected) = match variant {
        Variant::Tonal => (
            shade_color(base, ColorShade::S2),
            shade_color(base, ColorShade::S3),
        ),
        // A faint tint over the surface, the way the other unfilled variants
        // hover - the shadow is what actually moves.
        Variant::Elevated => (
            shade_color(base, ColorShade::S1),
            shade_color(base, ColorShade::S2),
        ),
        _ => (hover_color(base, filled), selected_color(base, filled)),
    };

    VariantColors {
        container,
        on_container,
        hover,
        selected,
    }
}

/// The `selected` look for `variant`, nested inside that variant's own block.
/// Ties that block's own `:hover` on specificity, so it has to stay *after* it
/// in `interactive_variant_sx`'s output - source order is what settles the two.
///
/// Only the background moves: the enclosing variant already sets the label
/// colour every arm here would want, bar `Text`, whose accent label on its own
/// tint is ~2.3:1. A literal `color` has no shade scale and so no selected
/// tint - falling back to the base colour would paint a full-strength
/// background under an `inherit` label, so the untinted variants fall back to
/// nothing at all.
pub(crate) fn variant_selected_sx(
    variant: Variant,
    color_var: &CssVar,
    selected_var: &CssVar,
) -> Sx {
    match variant {
        Variant::Filled | Variant::Tonal | Variant::Elevated => {
            sx().background(selected_var.value_or(color_var.value()))
        }
        // Both label the surface's accent, which is ~2.3:1 on its own tint.
        Variant::Outlined | Variant::Standard => sx()
            .background(selected_var.value_or("transparent"))
            .color("inherit"),
    }
}

static BUTTON_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = ripple_sx(ButtonDefaults::theme_vars())
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .border_style("solid")
        .border_width("1px")
        .font_weight("600")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .text_decoration("none")
        .outline("none");

    Variant::ALL
        .iter()
        .fold(base, |base, &variant| {
            base.when(
                variant.state_name(),
                interactive_variant_sx(variant, &BUTTON_VARS, &BUTTON_HOVER_VAR)
                    // After the variant's `:hover`, which it ties on specificity.
                    .when(
                        "checked",
                        variant_selected_sx(variant, &BUTTON_COLOR_VAR, &BUTTON_SELECTED_VAR),
                    ),
            )
        })
        .when(
            "disabled",
            // Also stops the variant's `:hover` from ever triggering.
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
        .when("full-width", sx().width("100%"))
        .when("loading", loading_sx())
        // The base outline is suppressed above and re-added only here.
        .focus_visible(focus_ring_sx())
});

/// While `loading`, the button holds exactly two children: the caller's own,
/// wrapped, and the loader.
///
/// The children stay in the tree at `opacity: 0` rather than being swapped for
/// the loader: they are the button's accessible name, and they hold its width,
/// so the button neither goes nameless nor jumps when the wait starts. The
/// loader sits on top, centred, at half the active size's height - each step
/// gets its own rule, since the button republishes no unsuffixed height to
/// read.
fn loading_sx() -> Sx {
    let base = sx()
        .cursor("progress")
        .selector(
            "& > span:first-child",
            sx().display("inline-flex")
                .align_items("center")
                .justify_content("center")
                .gap("inherit")
                .opacity("0"),
        )
        .selector(
            "& > span:last-child",
            sx().position("absolute").inset("0").margin("auto"),
        );

    Size::ALL.into_iter().fold(base, |base, size| {
        base.when(
            size.state_name(),
            sx().selector(
                "& > span:last-child",
                sx().var(
                    LOADER_SIZE,
                    format!("calc({} / 2)", BUTTON_HEIGHT.value(size)),
                ),
            ),
        )
    })
}

/// The colour half of the `style` attribute, already rendered. Depends on
/// `(variant, base)` alone - see [`use_button_variables`], which is what keeps
/// it off the render path.
pub(crate) fn button_variables(
    variant: Variant,
    base: &ThemeAwareValue,
    selectable: bool,
) -> String {
    let contrast = contrast_color(base);
    let colors = variant_colors(variant, base);
    // Only a toggle button ever reads it, and every other button would pay
    // for the extra declaration in its `style` attribute.
    let selected = selectable.then_some(colors.selected).flatten();

    variables()
        .with(BUTTON_COLOR_VAR, text_color(base))
        .with(BUTTON_FILL_VAR, fill_color(base))
        .with(BUTTON_CONTRAST_VAR, contrast.and_then(|c| c.resolve(None)))
        .with(BUTTON_HOVER_VAR, colors.hover)
        .with(BUTTON_SELECTED_VAR, selected)
        .with(BUTTON_CONTAINER_VAR, colors.container)
        .with(BUTTON_ON_CONTAINER_VAR, colors.on_container)
        .render()
}

base_props! {
    extends(button);
    pub struct ButtonProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        variant: Input<Variant>,
        /// Corner radius, independent of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default)]
        full_width: Option<bool>,
        /// Toggle button: renders `aria-pressed` and the selected look.
        /// `None` leaves the button a plain action.
        #[props(default)]
        selected: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// Overlays a `Loader` on the label and swallows clicks, while leaving
        /// the button focusable - a busy control is still one the reader can
        /// find. Renders `aria-busy` and `aria-disabled` rather than native
        /// `disabled`, which would drop focus mid-wait. Ignored in link mode.
        #[props(default)]
        loading: Option<bool>,
        /// `Option`, not a bare `EventHandler`: a defaulted one allocates a
        /// `GenerationalBox` on every render of every button - ~300 ns - even
        /// where no caller ever passes a handler.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router-aware link instead of a `<button>`. No
        /// ripple/`onclick` then - see `is_link` below.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        children: Element,
    }
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.copied_or(theme.button.variant);
    let color = base_color(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);
    let full_width = props.full_width.unwrap_or(false);
    let selectable = props.selected.is_some();
    let selected = props.selected.unwrap_or(false);
    let is_link = props.to.as_ref().is_some();
    let loading = props.loading.unwrap_or(false) && !is_link;

    if is_link && props.loading == Some(true) {
        warn("Button: `loading` is ignored on a link - an `<a>` has nothing to wait for.");
    }

    if selectable && is_link {
        warn(
            "Button: a link keeps the `selected` look but not `aria-pressed`, which `<a>` has no use for.",
        );
    }

    let ripple = use_ripple();

    let size = props.size.copied_or(theme.button.size);
    let radius = props.radius.copied_or(theme.button.radius);

    let showing = ripple.showing();
    // Colour resolution plus rendering is ~790 ns, and `(variant, color)` is
    // the same on almost every render of almost every button.
    let style = use_cache(
        (variant, color, selectable),
        |(variant, color, selectable)| button_variables(*variant, color, *selectable),
    );
    let style = match showing.as_ref() {
        Some(ripple) => ripple.with_point(style),
        None => style,
    };
    let style = Some(style).filter(|style| !style.is_empty());

    // Built in one allocation rather than through `.with()`, which is a
    // `retain` scan and a possible regrow per state. The caller's own states,
    // which are usually absent, keep the merging path.
    let mut own = Vec::with_capacity(8);
    own.extend([
        ("disabled", disabled),
        ("loading", loading),
        ("full-width", full_width),
        ("checked", selected),
        (variant.state_name(), true),
        (size.state_name(), true),
        (radius.radius_state_name(), true),
    ]);
    if let Some(ripple) = showing.as_ref() {
        own.push((ripple.state(), true));
    }

    let states: Input<States> = match props.states.as_ref() {
        Some(caller) => own
            .into_iter()
            .fold(caller.clone(), |states, (state, active)| {
                states.with(state, active)
            })
            .into(),
        None => States::from(own).into(),
    };

    let handle_click = move |event: Event<MouseData>| {
        // `prevent_default` as well as returning: a busy `type="submit"`
        // would otherwise still submit its form, by click or by Enter in a
        // field, since implicit submission is a synthetic click here.
        if loading {
            event.prevent_default();
            return;
        }
        ripple.press(&event);
        if let Some(onclick) = &props.onclick {
            onclick.call(event);
        }
    };

    // One hook for every path, above the branch: `prepare` is where
    // `use_style_attributes` runs, and hook order has to be the same on every
    // render. The renders below are pure.
    let boxed = use_box()
        .framework_sx(&BUTTON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(style.clone())
        .prepare();

    // `InternalAnchor` has no `onclick`: a link-mode Button navigates for
    // real and loses the ripple, which only makes sense on a `<button>`.
    if let Some(to) = props.to.as_ref().cloned() {
        // `<a>` has no native `disabled`: dropping `to` stops navigation,
        // `aria-disabled`/`tabindex` handle the a11y tree and tab order.
        // `InternalAnchor` can't do this - it always resolves a real link.
        if disabled {
            return boxed
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
                framework_sx: &BUTTON_BASE_SX,
                states,
                style,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    // The loader is `aria-hidden` - the button already has a name, and
    // `aria-busy` on it is what says it is waiting.
    let children = if loading {
        rsx! {
            span { {props.children} }
            Loader { size, color: "currentColor" }
        }
    } else {
        props.children
    };

    // `<button>` defaults to `submit`, which submits an enclosing form; a
    // `Button` defaults to `button`. `attr_default`, so a caller asking for
    // `submit` or `reset` still gets it.
    boxed
        .event("onclick", handle_click)
        .attr("disabled", disabled)
        .attr("aria-busy", loading.then_some("true"))
        .attr("aria-disabled", loading.then_some("true"))
        .attr_default("type", "button")
        .attr_default(
            "aria-pressed",
            selectable.then_some(if selected { "true" } else { "false" }),
        )
        .render(HtmlTag::Button, props.attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::{Color, ColorShade, ColorValue};

    /// `Button` passes `base_color`'s output, already an explicit shade.
    #[test]
    fn a_filled_button_darkens_on_hover_where_an_outlined_one_tints() {
        let base = base_color(Some(&ThemeAwareValue::Color(Color::Primary)));
        let filled = button_variables(Variant::Filled, &base, false);
        let outlined = button_variables(Variant::Outlined, &base, false);

        assert!(filled.contains(&format!(
            "{}:{};",
            BUTTON_HOVER_VAR.name(),
            ColorValue::Fill(Color::Primary, ColorShade::S6.darker()).value()
        )));
        assert!(outlined.contains(&format!(
            "{}:{};",
            BUTTON_HOVER_VAR.name(),
            ColorValue::Fill(Color::Primary, ColorShade::S1).value()
        )));
    }

    /// One base, two vars: the label reads on the page, the fill reads under
    /// its foreground, and the caller's shade indexes both ramps.
    #[test]
    fn the_color_variables_are_the_base_color_in_its_two_roles() {
        let base = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Error, ColorShade::S7));
        let variables = button_variables(Variant::Filled, &base, false);

        assert!(variables.starts_with(&format!(
            "{}:{};",
            BUTTON_COLOR_VAR.name(),
            ColorValue::Text(Color::Error, ColorShade::S7).value()
        )));
        assert!(variables.contains(&format!(
            "{}:{};",
            BUTTON_FILL_VAR.name(),
            ColorValue::Fill(Color::Error, ColorShade::S7).value()
        )));
    }

    /// No computable contrast, so the var stays unset and the class's own
    /// fallback applies.
    #[test]
    fn an_unparseable_color_emits_no_contrast() {
        let base = ThemeAwareValue::String("gold".to_string());
        let variables = button_variables(Variant::Filled, &base, false);

        assert!(!variables.contains(BUTTON_CONTRAST_VAR.name()));
    }

    #[test]
    fn each_variant_renders_its_own_css() {
        let class_of =
            |variant| interactive_variant_sx(variant, &BUTTON_VARS, &BUTTON_HOVER_VAR).class_name();

        let classes: Vec<String> = Variant::ALL.iter().map(|v| class_of(*v)).collect();
        let mut unique = classes.clone();
        unique.sort();
        unique.dedup();

        assert_eq!(unique.len(), Variant::ALL.len());
        assert_eq!(classes[0], class_of(Variant::ALL[0]));
    }

    /// Only `Tonal` paints a container - `Elevated` keeps the surface, so a
    /// container var would be dead weight in every elevated button's `style`.
    #[test]
    fn only_the_tonal_variant_emits_a_container() {
        let base = base_color(None);

        let tonal = button_variables(Variant::Tonal, &base, false);
        assert!(tonal.contains(BUTTON_CONTAINER_VAR.name()));
        assert!(tonal.contains(BUTTON_ON_CONTAINER_VAR.name()));

        for variant in [Variant::Elevated, Variant::Outlined] {
            let variables = button_variables(variant, &base, false);
            assert!(
                !variables.contains(BUTTON_CONTAINER_VAR.name()),
                "{variant:?}"
            );
        }
    }

    /// A literal color has no ramp, so the vars stay unset and the CSS falls
    /// back to the filled look rather than painting an invisible label.
    #[test]
    fn a_literal_color_gets_no_container() {
        let base = ThemeAwareValue::String("gold".to_string());
        let variables = button_variables(Variant::Tonal, &base, false);

        assert!(!variables.contains(BUTTON_CONTAINER_VAR.name()));
        assert!(!variables.contains(BUTTON_ON_CONTAINER_VAR.name()));
    }
}
