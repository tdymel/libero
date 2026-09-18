use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            BUTTON_COLOR_VAR, BUTTON_CONTAINER_VAR, BUTTON_CONTRAST_VAR, BUTTON_FILL_VAR,
            BUTTON_HOVER_VAR, BUTTON_ON_CONTAINER_VAR, BUTTON_ON_STATE_VAR, BUTTON_SELECTED_VAR,
            BUTTON_VARS, HtmlTag, Input, States, Variant, base_color, base_props, contrast_color,
            disabled_look_sx, fill_color, focus_ring_sx, input_from_str, interactive_variant_sx,
            text_color, variables, variant_colors, variant_selected_sx,
        },
        feedback::Loader,
        layout::{render_anchor, use_box},
    },
    hooks::{ripple_sx, use_cache, use_ripple, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{BUTTON_HEIGHT, ButtonDefaults, LOADER_SIZE, Size, SizeCss},
    utils::warn,
};

input_from_str!(NavigationTarget);

impl From<NavigationTarget> for Input<NavigationTarget> {
    fn from(value: NavigationTarget) -> Self {
        Input::Value(value)
    }
}

// Sealed: only `#[props(into)]` names it, through the impl below.
#[doc(hidden)]
#[allow(unnameable_types)]
pub struct RouteMarker;

// A typed `Route`, as `Anchor`'s `to` takes. A `From<R: Routable>` would
// overlap the impls above, so this rides `#[props(into)]`'s marker instead.
impl<R: Routable> dioxus::core::SuperFrom<R, RouteMarker> for Input<NavigationTarget> {
    fn super_from(value: R) -> Self {
        Input::Value(value.into())
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
        .outline("none")
        // One line that never outgrows its container (WCAG 1.4.10): a long
        // label is cut at the edge. No ellipsis: that would wrap the children.
        .max_width("100%")
        .min_width("0")
        .selector("& > [data-slot='button-icon']", button_icon_sx())
        // Inside the loading wrapper.
        .selector("& > span > [data-slot='button-icon']", button_icon_sx());

    Variant::ALL
        .iter()
        .fold(base, |base, &variant| {
            base.when(
                variant.state_name(),
                interactive_variant_sx(
                    variant,
                    &BUTTON_VARS,
                    &BUTTON_HOVER_VAR,
                    &BUTTON_ON_STATE_VAR,
                )
                // After the variant's `:hover`, which it ties on specificity.
                .when(
                    "checked",
                    variant_selected_sx(
                        variant,
                        &BUTTON_VARS,
                        &BUTTON_SELECTED_VAR,
                        &BUTTON_ON_STATE_VAR,
                    ),
                ),
            )
        })
        // No `pointer-events: none`: the variant's `:hover` skips it already.
        .when("disabled", disabled_look_sx("not-allowed"))
        // A disabled `Fieldset` disables the `<button>` natively (todo 499).
        .selector("&:disabled", disabled_look_sx("not-allowed"))
        .when("full-width", sx().width("100%"))
        .when("loading", loading_sx())
        // The base outline is suppressed above and re-added only here.
        .focus_visible(focus_ring_sx())
});

/// Before the children, never shrinking, one spacing step from them - as on `Chip`.
fn button_icon_sx() -> Sx {
    sx().display("inline-flex")
        .align_items("center")
        .flex("0 0 auto")
        .margin_right(SizeCss::SPACING.value(Size::Xs))
        .rtl(
            sx().margin_right("0")
                .margin_left(SizeCss::SPACING.value(Size::Xs)),
        )
}

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
        .with(BUTTON_ON_STATE_VAR, colors.on_state)
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
        /// ripple/`onclick` then - see `is_link` below. A path/URL or a typed
        /// route (`Route::Foo {}`), as on `Anchor`.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// Drawn before the label, with a gap; it never shrinks.
        #[props(default)]
        icon: Option<Element>,
        /// The label, on one line, laid out as the button's own flex items. A
        /// long one is cut at the edge; pass `title` to show it on hover.
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
        .style(style)
        .prepare();

    // Children unwrapped: they are the button's flex items, whatever they are.
    let label = rsx! {
        if let Some(icon) = props.icon {
            // Not `icon`: an `Alert`'s own slot is, and a button sits in one.
            span { "data-slot": "button-icon", {icon} }
        }
        {props.children}
    };

    // `InternalAnchor` has no `onclick`: a link-mode Button navigates for
    // real and loses the ripple, which only makes sense on a `<button>`.
    if let Some(to) = props.to.as_ref().cloned() {
        // `<a>` has no native `disabled`: dropping `to` stops navigation,
        // `aria-disabled`/`tabindex` handle the a11y tree and tab order.
        // `InternalAnchor` can't do this - it always resolves a real link.
        // An `<a>` without `href` is `generic`, so the role comes back by hand.
        if disabled {
            return boxed
                .attr_default("role", "link")
                .attr("aria-disabled", "true")
                .attr("tabindex", "-1")
                .render(HtmlTag::A, props.attributes, label);
        }

        // Styling already resolved above: an `InternalAnchor` scope would
        // resolve it again.
        return render_anchor(
            boxed.into_style_attributes(),
            to,
            props.target,
            None::<fn(MountedEvent)>,
            props.attributes,
            label,
        );
    }

    // The loader is `aria-hidden` - the button already has a name, and
    // `aria-busy` on it is what says it is waiting.
    let children = if loading {
        rsx! {
            span { {label} }
            Loader { size, color: "currentColor" }
        }
    } else {
        label
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
        let class_of = |variant| {
            interactive_variant_sx(
                variant,
                &BUTTON_VARS,
                &BUTTON_HOVER_VAR,
                &BUTTON_ON_STATE_VAR,
            )
            .class_name()
        };

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
