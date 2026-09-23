use dioxus::prelude::*;

use crate::{
    components::{
        common::{ButtonGroupContext, use_button_group},
        common::{HtmlTag, Input, States, Variables, Variant, base_props, variables},
        common::{
            VariantVars, base_color, borderless_on_state_sx, contrast_color, disabled_look_sx,
            fill_color, focus_ring_sx, interactive_variant_sx, literal_contrast, names_itself,
            text_color, use_name_warning, variant_colors, variant_selected_sx,
        },
        data_display::{Pictogram, SvgData},
        feedback::Loader,
        layout::{InternalAnchor, use_box},
    },
    hooks::{clipped_ripple_sx, use_gradient_style, use_ripple, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        ACTION_ICON_GLYPH, ACTION_ICON_RADIUS, ACTION_ICON_SIZE, BUTTON_HEIGHT, CssVar, Gradient,
        ICON_SIZE, LOADER_SIZE, SizeCss,
    },
    utils::warn,
};

const ACTION_ICON_COLOR_VAR: CssVar = CssVar::new("--lsx-action-icon-color");
const ACTION_ICON_FILL_VAR: CssVar = CssVar::new("--lsx-action-icon-fill");
const ACTION_ICON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-action-icon-contrast");
const ACTION_ICON_HOVER_VAR: CssVar = CssVar::new("--lsx-action-icon-hover");
const ACTION_ICON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-action-icon-container");
const ACTION_ICON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-action-icon-on-container");
const ACTION_ICON_SELECTED_VAR: CssVar = CssVar::new("--lsx-action-icon-selected");
const ACTION_ICON_ON_STATE_VAR: CssVar = CssVar::new("--lsx-action-icon-on-state");

/// Half the shortfall below 24px, out on each side of the box; `0` from 24px up.
const HIT_AREA_INSET: &str = "min(0px, calc((100% - 24px) / 2))";

const ACTION_ICON_VARS: VariantVars<'static> = VariantVars {
    color: &ACTION_ICON_COLOR_VAR,
    fill: &ACTION_ICON_FILL_VAR,
    contrast: &ACTION_ICON_CONTRAST_VAR,
    container: &ACTION_ICON_CONTAINER_VAR,
    on_container: &ACTION_ICON_ON_CONTAINER_VAR,
};

// Variant chrome only behind a state set by `variant`/`color`: an ungated declaration
// would beat a caller `sx`'s `:hover` (`Code`'s copy button).
static ACTION_ICON_BASE_SX: StaticSx = StaticSx::new(|| {
    let base = clipped_ripple_sx(sx())
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex_shrink("0")
        .border("none")
        .background("transparent")
        // The UA's `buttontext` is black natively on either scheme (todo 628).
        .color("inherit")
        .padding("0")
        .cursor("pointer")
        .outline("none")
        .width(ACTION_ICON_SIZE.overridable())
        .height(ACTION_ICON_SIZE.overridable())
        .border_radius(ACTION_ICON_RADIUS.overridable())
        .selector("& svg", sx().width("100%").height("100%"))
        // A bare glyph; one in a wrapper follows the wrapper.
        .selector(
            "& > svg",
            sx().width(ACTION_ICON_GLYPH.overridable())
                .height(ACTION_ICON_GLYPH.overridable()),
        )
        // WCAG 2.5.8: an invisible 24x24 press target round a smaller icon (todos 505, 566).
        .selector(
            "::before",
            sx().content("\"\"")
                .position("absolute")
                .inset(HIT_AREA_INSET),
        )
        // A chromeless toggle's pressed look; the variants repeat it over their fill.
        .when(
            "checked",
            borderless_on_state_sx().focus_visible(focus_ring_sx()),
        );

    let base = Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            // The ungated base is `border: none`, so the variants add the border here.
            interactive_variant_sx(
                variant,
                &ACTION_ICON_VARS,
                &ACTION_ICON_HOVER_VAR,
                &ACTION_ICON_ON_STATE_VAR,
            )
            .border_style("solid")
            .border_width("1px")
            // After the variant's `:hover`, which it ties on specificity.
            .when(
                "checked",
                variant_selected_sx(
                    variant,
                    &ACTION_ICON_VARS,
                    &ACTION_ICON_SELECTED_VAR,
                    &ACTION_ICON_ON_STATE_VAR,
                ),
            ),
        )
    });

    // `:disabled` too: a disabled `Fieldset` disables the `<button>` (todo 499).
    let disabled = || disabled_look_sx("not-allowed");
    base.when("disabled", disabled())
        .selector("&:disabled", disabled())
        .when("loading", loading_sx())
});

/// `Button`'s loading shape; the loader takes a share of the box, not of a line height.
fn loading_sx() -> Sx {
    sx().cursor("progress")
        .selector(
            "& > span:first-child",
            sx().display("inline-flex")
                .width("100%")
                .height("100%")
                .opacity("0"),
        )
        .selector(
            "& > span:last-child",
            sx().position("absolute").inset("0").margin("auto").var(
                LOADER_SIZE,
                format!("calc({} * 0.6)", ACTION_ICON_SIZE.overridable()),
            ),
        )
}

fn action_icon_variables(
    props: &ActionIconProps,
    variant: Variant,
    has_variant_styling: bool,
) -> Variables {
    let selectable = props.selected.is_some();
    // A size word is `Button`'s height round `Icon`'s glyph; a length is both.
    let glyph = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => Some(ICON_SIZE.value(*size)),
        Some(_) => Some("100%".to_string()),
        None => None,
    };
    let result = variables()
        .with(
            ACTION_ICON_SIZE.override_var(),
            props.size.resolve(Some(BUTTON_HEIGHT)),
        )
        .with(ACTION_ICON_GLYPH.override_var(), glyph)
        .with(
            ACTION_ICON_RADIUS.override_var(),
            props.radius.resolve(Some(SizeCss::RADIUS)),
        );

    if !has_variant_styling {
        return result;
    }

    let base = base_color(props.color.as_ref());
    let contrast = contrast_color(&base);
    let colors = variant_colors(variant, &base);

    result
        .with(ACTION_ICON_COLOR_VAR, text_color(&base))
        .with(ACTION_ICON_FILL_VAR, fill_color(&base))
        .with(
            ACTION_ICON_CONTRAST_VAR,
            contrast
                .and_then(|c| c.resolve(None))
                .or_else(|| literal_contrast(&base)),
        )
        .with(ACTION_ICON_HOVER_VAR, colors.hover)
        .with(ACTION_ICON_ON_STATE_VAR, colors.on_state)
        .with(ACTION_ICON_CONTAINER_VAR, colors.container)
        .with(ACTION_ICON_ON_CONTAINER_VAR, colors.on_container)
        .with(
            ACTION_ICON_SELECTED_VAR,
            selectable.then_some(colors.selected).flatten(),
        )
}

/// The enclosing `ButtonGroup`'s defaults, where the caller left a prop unset.
fn fill_from_group(props: &mut ActionIconProps, group: ButtonGroupContext) {
    fn fill<T>(prop: &mut Input<T>, group: Option<T>) {
        if let (Input::None, Some(value)) = (&*prop, group) {
            *prop = Input::Value(value);
        }
    }
    fill(&mut props.size, group.size.map(ThemeAwareValue::Size));
    fill(&mut props.radius, group.radius.map(ThemeAwareValue::Size));
    fill(&mut props.variant, group.variant);
    fill(&mut props.color, group.color);
    props.disabled = props.disabled.or(group.disabled);
}

base_props! {
    pub struct ActionIconProps {
        #[props(default, into)]
        variant: Input<Variant>,
        /// Second stop and angle of the `gradient` variant, whose first stop is `color`:
        /// `("secondary", 45)` or a [`Gradient`]. Unset keys take the theme's.
        #[props(default, into)]
        gradient: Option<Gradient>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// A size word takes `Button`'s height, the glyph `Icon`'s size; a length
        /// sizes both. Below 24px the press target stays 24x24 (WCAG 2.5.8).
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Required: an icon-only button has no text to announce.
        aria_label: String,
        #[props(default)]
        disabled: Option<bool>,
        /// With `disabled`: keeps the button in the Tab order, as on `Button`.
        #[props(default)]
        focusable_when_disabled: Option<bool>,
        /// Toggle button: renders `aria-pressed`. `None` is a plain action.
        #[props(default)]
        selected: Option<bool>,
        /// Overlays a `Loader` and swallows clicks; stays focusable. Ignored in link mode.
        #[props(default)]
        loading: Option<bool>,
        // `Option`, not a bare `EventHandler`: see `Button`.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router link instead of a `<button>`.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// The glyph, drawn as a [`Pictogram`]; `children` for anything else.
        #[props(default, into)]
        icon: Option<SvgData>,
        #[props(default)]
        children: Option<Element>,
    }
}

/// An icon-only button, for actions like copy or close.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::ActionIcon;
/// # fn app() -> Element {
/// rsx! {
///     ActionIcon { aria_label: "Close", onclick: move |_| {}, icon: pictogram_icons_lucide::x::outlined }
///     ActionIcon { aria_label: "Menu", svg { view_box: "0 0 24 24" } }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/action-icon>
#[component]
pub fn ActionIcon(mut props: ActionIconProps) -> Element {
    let group = use_button_group();
    fill_from_group(&mut props, group);
    let disabled = props.disabled.unwrap_or(false);
    let selectable = props.selected.is_some();
    let selected = props.selected.unwrap_or(false);
    let is_link = props.to.as_ref().is_some();
    let loading = props.loading.unwrap_or(false) && !is_link;
    let soft_disabled = disabled && props.focusable_when_disabled.unwrap_or(false);

    if is_link && props.loading == Some(true) {
        warn("ActionIcon:`loading` is ignored on a link - an `<a>` has nothing to wait for.");
    }
    let glyph = match (props.icon, props.children.take()) {
        (Some(icon), children) => {
            if children.is_some() {
                warn("ActionIcon: both `icon` and `children` set; `children` is ignored.");
            }
            rsx! { Pictogram { icon } }
        }
        (None, children) => children.unwrap_or_else(VNode::empty),
    };
    if selectable && is_link {
        warn(
            "ActionIcon: a link keeps the `selected` look but not `aria-pressed`, which `<a>` has no use for.",
        );
    }

    let has_variant_styling = props.variant.as_ref().is_some() || props.color.as_ref().is_some();
    let variant = props.variant.copied_or(use_theme().action_icon.variant);
    let variables: Input<Variables> =
        action_icon_variables(&props, variant, has_variant_styling).into();
    let gradient = use_gradient_style(
        props.gradient.as_ref(),
        props.color.as_ref(),
        has_variant_styling && variant == Variant::Gradient,
        false,
    );

    use_name_warning(
        !props.aria_label.trim().is_empty() || names_itself(&props.attributes),
        "ActionIcon: an empty `aria_label`, so it is announced as just \"button\".",
    );

    let ripple = use_ripple();
    let showing = ripple.showing();

    let states = props
        .states
        .unwrap_or_default()
        .with("disabled", disabled)
        .with("loading", loading)
        .with("checked", selected)
        .with(variant.state_name(), has_variant_styling);
    let states: Input<States> = match showing.as_ref() {
        Some(ripple) => states.with(ripple.state(), true),
        None => states,
    }
    .into();

    let handle_click = move |event: Event<MouseData>| {
        // Else a busy `type="submit"` still submits, as on `Button`.
        if loading || soft_disabled {
            event.prevent_default();
            return;
        }
        ripple.press(&event);
        if let Some(onclick) = &props.onclick {
            onclick.call(event);
        }
    };

    // One hook for every path, above the branch - see `Button`.
    let boxed = use_box()
        .framework_sx(&ACTION_ICON_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        // Only a button that has been clicked pays for the ripple.
        .style(match showing {
            Some(ripple) => Some(ripple.with_point(gradient.unwrap_or_default())),
            None => gradient,
        })
        .prepare();

    if let Some(to) = props.to.as_ref().cloned() {
        // `<a>` has no `disabled`: drop `href`; without it `<a>` is `generic`, so restore the role.
        if disabled {
            return boxed
                .attr_default("role", "link")
                .attr("aria-label", props.aria_label)
                .attr("aria-disabled", "true")
                .attr("tabindex", if soft_disabled { "0" } else { "-1" })
                .render(HtmlTag::A, props.attributes, glyph);
        }

        return rsx! {
            InternalAnchor {
                to,
                target: props.target,
                class: props.class,
                sx: props.sx,
                framework_sx: &ACTION_ICON_BASE_SX,
                states,
                variables,
                "aria-label": props.aria_label,
                attributes: props.attributes,
                {glyph}
            }
        };
    }

    let children = if loading {
        rsx! {
            span { {glyph} }
            Loader { color: "currentColor" }
        }
    } else {
        glyph
    };

    boxed
        .event("onclick", handle_click)
        .attr("aria-label", props.aria_label)
        .attr("disabled", disabled && !soft_disabled)
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
    use crate::tokens::{Color, Size};

    fn action_icon_props(color: Input<ThemeAwareValue>) -> ActionIconProps {
        ActionIconProps {
            gradient: None,
            class: Default::default(),
            sx: Default::default(),
            states: Input::None,
            attributes: Vec::new(),
            variant: Input::None,
            color,
            size: Size::Md.into(),
            radius: Input::None,
            aria_label: "test".to_string(),
            disabled: None,
            focusable_when_disabled: None,
            selected: None,
            loading: None,
            onclick: None,
            to: Input::None,
            target: None,
            icon: None,
            children: None,
        }
    }

    /// Sizing is always ActionIcon's; the color vars appear only when it
    /// isn't deferring to a wrapped `Icon`.
    #[test]
    fn without_variant_styling_only_the_size_variables_are_set() {
        let props = action_icon_props(Color::Primary.into());
        let variables = action_icon_variables(&props, Variant::Filled, false).to_string();

        assert!(variables.contains(ACTION_ICON_SIZE.override_var().name()));
        assert!(!variables.contains(ACTION_ICON_COLOR_VAR.name()));
    }

    /// Todo 1069: a size word is `Button`'s height round `Icon`'s glyph; a length fills.
    #[test]
    fn a_size_word_takes_the_button_height_and_a_length_fills() {
        let word = action_icon_variables(&action_icon_props(Input::None), Variant::Filled, false)
            .to_string();
        assert!(word.contains(&BUTTON_HEIGHT.value(Size::Md)), "{word}");
        assert!(word.contains(&ICON_SIZE.value(Size::Md)), "{word}");

        let length = ActionIconProps {
            size: "20px".into(),
            ..action_icon_props(Input::None)
        };
        let length = action_icon_variables(&length, Variant::Filled, false).to_string();
        assert!(length.contains("-size-override:20px;"), "{length}");
        assert!(length.contains("-glyph-override:100%;"), "{length}");
    }

    #[test]
    fn with_variant_styling_the_colour_variables_are_added() {
        let props = action_icon_props(Color::Primary.into());
        let variables = action_icon_variables(&props, Variant::Filled, true).to_string();

        assert!(variables.contains(ACTION_ICON_SIZE.override_var().name()));
        assert!(variables.contains(ACTION_ICON_COLOR_VAR.name()));
    }
}
