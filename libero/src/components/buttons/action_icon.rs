use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables, Variant,
        common::base_props,
        common::{
            VariantVars, base_color, borderless_on_state_sx, contrast_color, disabled_look_sx,
            fill_color, focus_ring_sx, interactive_variant_sx, names_itself, text_color,
            use_name_warning, variant_colors, variant_selected_sx,
        },
        feedback::Loader,
        layout::{InternalAnchor, use_box},
        variables,
    },
    hooks::{clipped_ripple_sx, use_ripple, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{ACTION_ICON_RADIUS, ACTION_ICON_SIZE, CssVar, ICON_SIZE, LOADER_SIZE, SizeCss},
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

// `Icon`'s sizing plus the resets a native `<button>` needs that its `<span>`
// never did.
//
// The variant chrome sits behind a data-state token that `ActionIcon` sets
// only when the caller asks for it via `variant`/`color`. A caller with its
// own `sx` (`Code`'s copy button) needs background/color untouched: layering
// means a plain declaration here would beat a `:hover` rule from their
// lower-priority `sx`.
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
        // WCAG 2.5.8: an invisible 24x24 press target round a smaller icon,
        // the drawn box unchanged (todos 505, 566). Nothing at 24px and up.
        .selector(
            "::before",
            sx().content("\"\"")
                .position("absolute")
                .inset(HIT_AREA_INSET),
        )
        // A chromeless toggle's only pressed look; a variant's own block below
        // repeats it over its fill.
        .when(
            "checked",
            borderless_on_state_sx().focus_visible(focus_ring_sx()),
        );

    let base = Variant::ALL.iter().fold(base, |base, &variant| {
        base.when(
            variant.state_name(),
            // The ungated base is `border: none` for a caller with its own
            // `sx`, so the width the variants' `border-color` needs joins
            // here rather than out there.
            interactive_variant_sx(
                variant,
                &ACTION_ICON_VARS,
                &ACTION_ICON_HOVER_VAR,
                &ACTION_ICON_ON_STATE_VAR,
            )
            .border_style("solid")
            .border_width("1px")
            // After the variant's `:hover`, which it ties on specificity -
            // as on `Button`.
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

    // The variants' `:hover` skips a disabled control, so no `pointer-events:
    // none`. `:disabled` too: a disabled `Fieldset` disables the `<button>` (todo 499).
    let disabled = || disabled_look_sx("not-allowed");
    base.when("disabled", disabled())
        .selector("&:disabled", disabled())
        .when("loading", loading_sx())
});

/// `Button`'s loading shape: the caller's icon stays in the tree, hidden,
/// because it holds the box and its `aria-label` is the name; the loader sits
/// on top, centred. An icon fills its whole box, so the loader takes a share
/// of the box rather than of a line height.
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
    // Only a toggle reads it, and every other icon would pay for the
    // declaration in its `style` attribute.
    let selectable = props.selected.is_some();
    let result = variables()
        .with(
            ACTION_ICON_SIZE.override_var(),
            props.size.resolve(Some(ICON_SIZE)),
        )
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
            contrast.and_then(|c| c.resolve(None)),
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

base_props! {
    pub struct ActionIconProps {
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// `"sm"` is 20x20 and `"xs"` 16x16. Below 24px the button takes presses
        /// in an invisible 24x24 box centred on it (WCAG 2.5.8), so keep 2px
        /// (`sm`) or 4px (`xs`) clear of any other target.
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Required: an icon-only button has no text to announce.
        aria_label: String,
        #[props(default)]
        disabled: Option<bool>,
        /// Toggle button: renders `aria-pressed`, and the selected look when a
        /// `variant` or `color` turns the chrome on. `None` leaves it a plain
        /// action.
        #[props(default)]
        selected: Option<bool>,
        /// Overlays a `Loader` on the icon and swallows clicks, while leaving
        /// the button focusable - a busy control is still one the reader can
        /// find. Renders `aria-busy` and `aria-disabled` rather than native
        /// `disabled`, which would drop focus mid-wait. Ignored in link mode.
        #[props(default)]
        loading: Option<bool>,
        /// `Option`, not a bare `EventHandler` - see `Button`.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router-aware link instead of a `<button>`. No
        /// ripple/`onclick`/`onmouseleave` then - real navigation happens
        /// instead.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        children: Element,
    }
}

/// `Icon`'s badge as a real `<button>`, with `Button`'s click handling and
/// a11y - for icon-only actions like copy or close.
#[component]
pub fn ActionIcon(props: ActionIconProps) -> Element {
    let disabled = props.disabled.unwrap_or(false);
    let selectable = props.selected.is_some();
    let selected = props.selected.unwrap_or(false);
    let is_link = props.to.as_ref().is_some();
    let loading = props.loading.unwrap_or(false) && !is_link;

    if is_link && props.loading == Some(true) {
        warn("ActionIcon: `loading` is ignored on a link - an `<a>` has nothing to wait for.");
    }
    if selectable && is_link {
        warn(
            "ActionIcon: a link keeps the `selected` look but not `aria-pressed`, which `<a>` has no use for.",
        );
    }

    let has_variant_styling = props.variant.as_ref().is_some() || props.color.as_ref().is_some();
    let variant = props.variant.copied_or(use_theme().action_icon.variant);
    let variables: Input<Variables> =
        action_icon_variables(&props, variant, has_variant_styling).into();

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
        // `prevent_default` as well as returning, as on `Button`: a busy
        // `type="submit"` would otherwise still submit its form.
        if loading {
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
        // Only a button that has been clicked pays for this.
        .style(showing.map(|ripple| ripple.with_point(String::new())))
        .prepare();

    if let Some(to) = props.to.as_ref().cloned() {
        // `<a>` has no native `disabled`: dropping `to` stops navigation,
        // `aria-disabled`/`tabindex` handle the a11y tree and tab order.
        // `InternalAnchor` can't do this - it always resolves a real link.
        // An `<a>` without `href` is `generic`, so the role comes back by hand.
        if disabled {
            return boxed
                .attr_default("role", "link")
                .attr("aria-label", props.aria_label)
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
                framework_sx: &ACTION_ICON_BASE_SX,
                states,
                variables,
                "aria-label": props.aria_label,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    // The loader is `aria-hidden` - the button already has its name, and
    // `aria-busy` on it is what says it is waiting.
    let children = if loading {
        rsx! {
            span { {props.children} }
            Loader { color: "currentColor" }
        }
    } else {
        props.children
    };

    boxed
        .event("onclick", handle_click)
        .attr("aria-label", props.aria_label)
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
    use crate::tokens::{Color, Size};

    fn action_icon_props(color: Input<ThemeAwareValue>) -> ActionIconProps {
        ActionIconProps {
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
            selected: None,
            loading: None,
            onclick: None,
            to: Input::None,
            target: None,
            children: rsx! {},
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

    #[test]
    fn with_variant_styling_the_colour_variables_are_added() {
        let props = action_icon_props(Color::Primary.into());
        let variables = action_icon_variables(&props, Variant::Filled, true).to_string();

        assert!(variables.contains(ACTION_ICON_SIZE.override_var().name()));
        assert!(variables.contains(ACTION_ICON_COLOR_VAR.name()));
    }
}
