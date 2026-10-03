use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            BUTTON_COLOR_VAR, BUTTON_CONTAINER_VAR, BUTTON_CONTRAST_VAR, BUTTON_FILL_VAR,
            BUTTON_HOVER_VAR, BUTTON_ON_CONTAINER_VAR, BUTTON_ON_STATE_VAR, BUTTON_SELECTED_VAR,
            BUTTON_VARS, HtmlTag, Input, Part, States, TOOLBAR_ITEM, ToolbarItem, Variant,
            base_color, base_props, contrast_color, disabled_look_sx, fill_color, focus_ring_sx,
            input_from_str, interactive_variant_sx, literal_contrast, parts_enum, text_color,
            use_button_group, use_toolbar_item, variables, variant_colors, variant_selected_sx,
        },
        feedback::Loader,
        layout::{render_anchor, use_box},
    },
    hooks::{ripple_sx, use_cache, use_gradient_style, use_ripple, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{BUTTON_HEIGHT, ButtonDefaults, Gradient, LOADER_SIZE, SURFACE_LABEL, Size, SizeCss},
    utils::warn,
};

input_from_str!(NavigationTarget);

// Sealed: only `#[props(into)]` names it, through the impl below.
#[doc(hidden)]
#[allow(unnameable_types)]
pub struct RouteMarker;

// A typed `Route`, as on `Anchor`. `From<R: Routable>` would overlap the impls above.
impl<R: Routable> dioxus::core::SuperFrom<R, RouteMarker> for Input<NavigationTarget> {
    fn super_from(value: R) -> Self {
        Input::Value(value.into())
    }
}

static BUTTON_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = ripple_sx(ButtonDefaults::theme_vars())
        .display("inline-flex")
        .align_items("center")
        // `safe`: an overlong label is cut at its end, not at both (todo 1493).
        .justify_content("safe center")
        .border_style("solid")
        .border_width("1px")
        .font_weight("600")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .text_decoration("none")
        .outline("none")
        // One line, never wider than its container (WCAG 1.4.10). No ellipsis: it would wrap the children.
        .max_width("100%")
        .min_width("0")
        .selector(ButtonPart::Icon.selector(), button_icon_sx());

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
        .focus_visible(focus_ring_sx())
});

/// Before the children, never shrinking, as on `Chip`.
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

/// Children stay at `opacity: 0` under the loader: they keep the accessible name and the width.
/// One rule per size, since the button publishes no unsuffixed height.
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

/// The colour half of the `style` attribute. Depends on its arguments alone, so `Button` caches it.
/// `uncolored`: no caller `color`, so a `standard` label takes a fill's around it (todo 1663).
pub(crate) fn button_variables(
    variant: Variant,
    base: &ThemeAwareValue,
    selectable: bool,
    uncolored: bool,
) -> String {
    let contrast = contrast_color(base);
    let colors = variant_colors(variant, base);
    // Only a toggle button reads it; the rest skip the declaration.
    let selected = selectable.then_some(colors.selected).flatten();
    let label = text_color(base).map(|label| match variant {
        // The outlined border is the label colour: it follows too (todo 1833).
        Variant::Standard | Variant::Outlined if uncolored => {
            format!("var({}, {label})", SURFACE_LABEL.name())
        }
        _ => label,
    });

    variables()
        .with(BUTTON_COLOR_VAR, label)
        .with(BUTTON_FILL_VAR, fill_color(base))
        .with(
            BUTTON_CONTRAST_VAR,
            contrast
                .and_then(|c| c.resolve(None))
                .or_else(|| literal_contrast(base)),
        )
        .with(BUTTON_HOVER_VAR, colors.hover)
        .with(BUTTON_SELECTED_VAR, selected)
        .with(BUTTON_ON_STATE_VAR, colors.on_state)
        .with(BUTTON_CONTAINER_VAR, colors.container)
        .with(BUTTON_ON_CONTAINER_VAR, colors.on_container)
        .render()
}

parts_enum! {
    /// [`Button`]'s inner parts, for its `parts` prop.
    pub enum ButtonPart {
        /// The `icon` wrapper; under `loading` it sits in the hidden label's wrapper.
        Icon = "button-icon" => "& > [data-slot='button-icon'], & > span > [data-slot='button-icon']",
    }
}

base_props! {
    extends(button);
    parts(ButtonPart);
    pub struct ButtonProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        variant: Input<Variant>,
        /// Second stop and angle of the `gradient` variant, whose first stop is `color`:
        /// `("secondary", 45)` or a [`Gradient`]. Unset keys take the theme's.
        #[props(default, into)]
        gradient: Option<Gradient>,
        /// Corner radius, independent of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default)]
        full_width: Option<bool>,
        /// Toggle button: renders `aria-pressed`. `None` is a plain action.
        #[props(default)]
        selected: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// With `disabled`: stays in the Tab order, with `aria-disabled`.
        #[props(default)]
        focusable_when_disabled: Option<bool>,
        /// Overlays a `Loader` and swallows clicks; stays focusable. Ignored in link mode.
        #[props(default)]
        loading: Option<bool>,
        // `Option`: a defaulted `EventHandler` allocates on every render (~300 ns).
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router link instead of a `<button>`: a path, URL or typed route.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// Drawn before the label.
        #[props(default)]
        icon: Option<Element>,
        /// The label, on one line; a long one is cut at the edge.
        children: Element,
    }
}

/// A themed button, or a router link styled as one when `to` is set.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Button;
/// # fn app() -> Element {
/// rsx! {
///     Button { variant: "outlined", onclick: move |_| {}, "Save" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/button>
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let theme = use_theme();
    let group = use_button_group();
    let item = use_toolbar_item();
    let variant = props
        .variant
        .copied_or(group.variant.unwrap_or(theme.button.variant));
    let color_prop = props.color.as_ref().or(group.color.as_ref());
    let color = base_color(color_prop);
    let disabled = props.disabled.or(group.disabled).unwrap_or(false);
    let full_width = props.full_width.unwrap_or(false);
    let selectable = props.selected.is_some();
    let selected = props.selected.unwrap_or(false);
    let is_link = props.to.as_ref().is_some();
    let loading = props.loading.unwrap_or(false) && !is_link;
    // A toolbar keeps its disabled items in the arrow order.
    let soft_disabled = disabled && props.focusable_when_disabled.unwrap_or(item.is_some());

    if is_link && props.loading == Some(true) {
        warn("Button: `loading` is ignored on a link - an `<a>` has nothing to wait for.");
    }

    if selectable && is_link {
        warn(
            "Button: a link keeps the `selected` look but not `aria-pressed`, which `<a>` has no use for.",
        );
    }

    let ripple = use_ripple();

    let size = props
        .size
        .copied_or(group.size.unwrap_or(theme.button.size));
    let radius = props
        .radius
        .copied_or(group.radius.unwrap_or(theme.button.radius));

    let showing = ripple.showing();
    // ~790 ns uncached, and the key rarely changes between renders.
    let uncolored = color_prop.is_none();
    let style = use_cache(
        (variant, color, selectable, uncolored),
        |(variant, color, selectable, uncolored)| {
            button_variables(*variant, color, *selectable, *uncolored)
        },
    );
    let gradient = use_gradient_style(
        props.gradient.as_ref(),
        color_prop,
        variant == Variant::Gradient,
        false,
    );
    let style = match gradient {
        Some(gradient) => format!("{style}{gradient}"),
        None => style,
    };
    let style = match showing.as_ref() {
        Some(ripple) => ripple.with_point(style),
        None => style,
    };
    let style = Some(style).filter(|style| !style.is_empty());

    // One allocation; `.with()` scans and may regrow per state. Caller states merge.
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
        if let Some(item) = item {
            item.take_stop();
        }
        // Else a busy `type="submit"` still submits, by click or by Enter in a field.
        // Android's WebView sends a tap on a disabled button's child a click (990).
        if loading || disabled {
            event.prevent_default();
            return;
        }
        ripple.press(&event);
        if let Some(onclick) = &props.onclick {
            onclick.call(event);
        }
    };

    // Above the branch: `prepare` runs a hook, and hook order must not change.
    let boxed = use_box()
        .framework_sx(&BUTTON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&states)
        .style(style)
        .prepare();

    let label = rsx! {
        if let Some(icon) = props.icon {
            // Not `icon`: an `Alert`'s own slot is, and a button sits in one.
            span { "data-slot": ButtonPart::Icon.slot(), {icon} }
        }
        {props.children}
    };

    // Link mode: no `onclick`, no ripple.
    if let Some(to) = props.to.as_ref().cloned() {
        // `<a>` has no `disabled`: drop `href`; without it `<a>` is `generic`, so restore the role.
        if disabled {
            let tabindex = match item {
                Some(item) => item.tabindex(),
                None if soft_disabled => "0",
                None => "-1",
            };
            return boxed
                .attr_default("role", "link")
                .attr("aria-disabled", "true")
                .attr("tabindex", tabindex)
                .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
                .render(HtmlTag::A, props.attributes, label);
        }

        let mut attributes = props.attributes;
        attributes.extend(item.into_iter().flat_map(ToolbarItem::attributes));
        return render_anchor(
            boxed.into_style_attributes(),
            to,
            props.target,
            None::<fn(MountedEvent)>,
            attributes,
            label,
        );
    }

    let children = if loading {
        rsx! {
            span { {label} }
            Loader { size, color: "currentColor" }
        }
    } else {
        label
    };

    // `<button>` defaults to `submit`; a caller can still ask for it.
    boxed
        .event("onclick", handle_click)
        .attr("disabled", disabled && !soft_disabled)
        .attr("tabindex", item.map(ToolbarItem::tabindex))
        .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
        .attr("aria-busy", loading.then_some("true"))
        .attr(
            "aria-disabled",
            (loading || soft_disabled).then_some("true"),
        )
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
    use crate::{
        components::common::part_table,
        tokens::{Color, ColorShade, ColorValue},
    };

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<ButtonPart>(),
            [(
                "button-icon",
                "& > [data-slot='button-icon'], & > span > [data-slot='button-icon']"
            )]
        );
    }

    /// `Button` passes `base_color`'s output, already an explicit shade.
    #[test]
    fn a_filled_button_darkens_on_hover_where_an_outlined_one_tints() {
        let base = base_color(Some(&ThemeAwareValue::Color(Color::Primary)));
        let filled = button_variables(Variant::Filled, &base, false, false);
        let outlined = button_variables(Variant::Outlined, &base, false, false);

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
        let variables = button_variables(Variant::Filled, &base, false, false);

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

    /// Todos 1663, 1833: only an uncoloured `standard` or `outlined` label falls back through a fill's label.
    #[test]
    fn an_uncolored_standard_or_outlined_label_takes_the_surface_label() {
        let base = base_color(None);
        let label = |variant, uncolored| {
            let variables = button_variables(variant, &base, false, uncolored);
            variables
                .split(';')
                .find_map(|declaration| {
                    declaration.strip_prefix(&format!("{}:", BUTTON_COLOR_VAR.name()))
                })
                .unwrap()
                .to_string()
        };
        let surface = format!("var({}, ", SURFACE_LABEL.name());
        assert!(label(Variant::Standard, true).starts_with(&surface));
        assert!(!label(Variant::Standard, false).starts_with(&surface));
        assert!(label(Variant::Outlined, true).starts_with(&surface));
        assert!(!label(Variant::Outlined, false).starts_with(&surface));
        assert!(!label(Variant::Filled, true).starts_with(&surface));
    }

    /// A literal fill publishes black or white, not the page's text colour; an
    /// unparseable one leaves the pick to the browser.
    #[test]
    fn a_literal_color_publishes_a_readable_contrast() {
        for (color, contrast) in [("#ffeb3b", "#000000"), ("gold", "contrast-color(gold)")] {
            let variables =
                button_variables(Variant::Filled, &ThemeAwareValue::from(color), false, false);
            assert!(
                variables.contains(&format!("{}:{contrast};", BUTTON_CONTRAST_VAR.name())),
                "{color}: {variables}"
            );
        }
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

        let tonal = button_variables(Variant::Tonal, &base, false, false);
        assert!(tonal.contains(BUTTON_CONTAINER_VAR.name()));
        assert!(tonal.contains(BUTTON_ON_CONTAINER_VAR.name()));

        for variant in [Variant::Elevated, Variant::Outlined] {
            let variables = button_variables(variant, &base, false, false);
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
        let variables = button_variables(Variant::Tonal, &base, false, false);

        assert!(!variables.contains(BUTTON_CONTAINER_VAR.name()));
        assert!(!variables.contains(BUTTON_ON_CONTAINER_VAR.name()));
    }
}
