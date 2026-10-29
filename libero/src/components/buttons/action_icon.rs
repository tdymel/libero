use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        common::{ButtonGroupContext, Glyph, use_button_group},
        common::{
            COARSE_POINTER, COARSE_TARGET, VariantVars, base_color, borderless_on_state_sx,
            coarse_hit_area_sx, contrast_color, disabled_look_sx, fill_color, focus_ring_sx,
            interactive_variant_sx, literal_contrast, min_hit_area_sx, names_itself, text_color,
            use_name_warning, variant_colors, variant_selected_sx,
        },
        common::{HtmlTag, Input, States, Variables, Variant, base_props, variables},
        common::{TOOLBAR_ITEM, ToolbarItem, use_toolbar_item},
        data_display::{Pictogram, SvgData},
        feedback::Loader,
        form::use_fieldset_disabled,
        layout::{InternalAnchor, use_box},
        navigation::wants_new_tab_hint,
        overlay::Tooltip,
        typography::{chord_kbd, chord_words},
    },
    context::IconSlot,
    hooks::{
        Chord, aria_keyshortcuts, child_ripple_sx, current_localization, use_gradient_style,
        use_ripple, use_theme,
    },
    platform::mod_is_meta,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        ACTION_ICON_GLYPH, ACTION_ICON_RADIUS, ACTION_ICON_SIZE, BUTTON_HEIGHT, CssVar, Gradient,
        ICON_SIZE, LOADER_SIZE, Size, SizeCss,
    },
    utils::use_warn_once,
};

const ACTION_ICON_COLOR_VAR: CssVar = CssVar::new("--lsx-action-icon-color");
const ACTION_ICON_FILL_VAR: CssVar = CssVar::new("--lsx-action-icon-fill");
const ACTION_ICON_CONTRAST_VAR: CssVar = CssVar::new("--lsx-action-icon-contrast");
const ACTION_ICON_HOVER_VAR: CssVar = CssVar::new("--lsx-action-icon-hover");
const ACTION_ICON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-action-icon-container");
const ACTION_ICON_ON_CONTAINER_VAR: CssVar = CssVar::new("--lsx-action-icon-on-container");
const ACTION_ICON_SELECTED_VAR: CssVar = CssVar::new("--lsx-action-icon-selected");
const ACTION_ICON_ON_STATE_VAR: CssVar = CssVar::new("--lsx-action-icon-on-state");

/// Set on the `xs` and `sm` icons, which grow a 44px touch box: a `Toolbar` or `ButtonGroup` keeps
/// it off a neighbour, a loose row needs a 10px (`xs`) or 7px (`sm`) gap.
pub(crate) const ACTION_ICON_COMPACT: &str = "compact";

/// Set on the `md` and larger icons, whose own box grows to 44px under a finger, as `Button`'s does.
pub(crate) const ACTION_ICON_GROWS: &str = "grows";

/// A `px`, `rem` or `em` length (16px) in pixels; a `calc` or a variable is `None`.
fn length_px(text: &str) -> Option<f32> {
    let text = text.trim();
    let (number, scale) = match text.strip_suffix("rem").or_else(|| text.strip_suffix("em")) {
        Some(number) => (number, 16.0),
        None => (text.trim_end_matches("px"), 1.0),
    };
    number
        .trim()
        .parse::<f32>()
        .ok()
        .map(|length| length * scale)
}

/// An `xs` or `sm` size word, or a length up to `sm`'s 30px (`rem` and `em` at 16px).
pub(crate) fn is_compact(size: Option<&ThemeAwareValue>) -> bool {
    match size {
        Some(ThemeAwareValue::Size(size)) => matches!(size, Size::Xs | Size::Sm),
        Some(ThemeAwareValue::String(text) | ThemeAwareValue::Number(text)) => {
            length_px(text).is_some_and(|length| length <= 30.0)
        }
        _ => false,
    }
}

/// An `md` or larger size word (`themed` when unset), or a length past 30px. A size the
/// icon cannot read, a `calc`, is neither: a wrapper that knows it sets the state itself.
pub(crate) fn grows_on_touch(size: Option<&ThemeAwareValue>, themed: Size) -> bool {
    match size {
        None => !matches!(themed, Size::Xs | Size::Sm),
        Some(ThemeAwareValue::Size(size)) => !matches!(size, Size::Xs | Size::Sm),
        Some(ThemeAwareValue::String(text) | ThemeAwareValue::Number(text)) => {
            length_px(text).is_some_and(|length| length > 30.0)
        }
        _ => false,
    }
}

/// The new-tab cue's side, from the box's height, not its width, which a `Repository` pill grows.
pub(crate) fn new_tab_cue_side() -> String {
    format!("max(10px, calc({} * 0.34))", ACTION_ICON_SIZE.overridable())
}

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
    let base = child_ripple_sx(sx())
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
        // A `_blank` link's corner cue, in the glyph's colour, so it keeps the glyph's contrast.
        .selector(
            "& > [data-slot='new-tab']",
            sx().position("absolute")
                .inset_inline_end("0")
                .bottom("0")
                .display("inline-flex")
                .width(new_tab_cue_side())
                .height(new_tab_cue_side())
                .pointer_events("none"),
        )
        // WCAG 2.5.8: an invisible 24x24 press target round a smaller icon (todos 505, 566),
        // 44x44 on a coarse pointer.
        .and(min_hit_area_sx("::before"))
        .when(ACTION_ICON_COMPACT, coarse_hit_area_sx("::before"))
        .when(
            ACTION_ICON_GROWS,
            sx().media(
                COARSE_POINTER,
                sx().min_width(COARSE_TARGET).min_height(COARSE_TARGET),
            ),
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
                &ACTION_ICON_SELECTED_VAR,
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

    // Last, as on `Button`: the elevated variant's shadow ties the ring's on specificity.
    base.when("disabled", disabled_look_sx("not-allowed"))
        .when("loading", loading_sx())
        .focus_visible(focus_ring_sx())
});

/// `Button`'s loading shape; the loader takes a share of the box, not of a line height.
/// By slot, not position: a press appends a ripple span after the loader.
fn loading_sx() -> Sx {
    sx().cursor("progress")
        .selector(
            "& > [data-slot='glyph']",
            sx().display("inline-flex")
                .width("100%")
                .height("100%")
                .opacity("0"),
        )
        .selector(
            "& > [data-slot='spinner']",
            // Centred in flex too, for a custom loader that fills the box.
            sx().position("absolute")
                .inset("0")
                .margin("auto")
                .display("inline-flex")
                .align_items("center")
                .justify_content("center")
                .var(
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
        .with(ACTION_ICON_SELECTED_VAR, colors.selected)
}

/// The enclosing `ButtonGroup`'s defaults, where the caller left a prop unset.
fn fill_from_group(props: &mut ActionIconProps, group: &ButtonGroupContext) {
    fn fill<T>(prop: &mut Input<T>, group: Option<T>) {
        if let (Input::None, Some(value)) = (&*prop, group) {
            *prop = Input::Value(value);
        }
    }
    fill(&mut props.size, group.size.map(ThemeAwareValue::Size));
    fill(&mut props.radius, group.radius.clone());
    fill(&mut props.variant, group.variant);
    fill(&mut props.color, group.color.clone());
    props.selected = props
        .selected
        .or_else(|| group.pressed(props.value.as_deref()));
    props.disabled = props.disabled.or(group.disabled);
    props.focusable_when_disabled = props
        .focusable_when_disabled
        .or(group.focusable_when_disabled);
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
        /// sizes both. Below 24px the press target stays 24x24 (WCAG 2.5.8), 44x44 under a finger at `xs` and `sm`
        /// (in a `Toolbar`, up to half its gap along the bar).
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Required: an icon-only button has no text to announce.
        aria_label: String,
        /// Shows `aria_label` in a `Tooltip` on hover and keyboard focus.
        #[props(default)]
        tooltip: Option<bool>,
        /// The chord that runs this action, as `use_hotkeys` takes it (`"mod+b"`): shown
        /// in the tooltip and set as `aria-keyshortcuts`.
        #[props(default, into)]
        shortcut: Option<String>,
        #[props(default)]
        disabled: Option<bool>,
        /// With `disabled`: keeps the button in the Tab order, as on `Button`. Either way it keeps a focus it has.
        /// Unset, `false`; inside a `Toolbar`, `true`.
        #[props(default)]
        focusable_when_disabled: Option<bool>,
        /// Toggle button: renders `aria-pressed`. `None` is a plain action.
        #[props(default)]
        selected: Option<bool>,
        /// Overlays a `Loader` and swallows clicks; stays focusable. Ignored in link mode.
        /// Sets only `aria-busy`: announce the progress in your own `role="status"` region.
        #[props(default)]
        loading: Option<bool>,
        /// Drawn in place of the `Loader` while `loading`. Size it yourself.
        #[props(default)]
        loader: Option<Element>,
        /// The press ripple. `false` drops it.
        #[props(default = true)]
        ripple: bool,
        /// Its key in the enclosing `ButtonGroup`'s `value`, which makes it a toggle.
        #[props(default, into)]
        value: Option<String>,
        // `Option`, not a bare `EventHandler`: see `Button`.
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        /// Renders a router link instead of a `<button>`.
        #[props(default, into)]
        to: Input<NavigationTarget>,
        #[props(default)]
        target: Option<String>,
        /// With `target: "_blank"`, "(opens in a new tab)" in the name and tooltip. `false` drops it and the icon.
        #[props(default = true)]
        new_tab_hint: bool,
        /// With the hint, draws a corner icon. Unset, the theme's `anchor.new_tab_icon`, off.
        #[props(default)]
        new_tab_icon: Option<bool>,
        /// The glyph, drawn as a [`Pictogram`]; `children` for anything else.
        #[props(default, into)]
        icon: Option<SvgData>,
        #[props(default)]
        children: Option<Element>,
    }
}

/// Provided above `ActionIcon`s whose tooltips open late: the media controls' (todo 1439).
#[derive(Clone, Copy)]
pub(crate) struct TooltipOpenDelay(pub(crate) u32);

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
pub fn ActionIcon(props: ActionIconProps) -> Element {
    let group = use_button_group();
    let apple = mod_is_meta();
    let warned = use_warn_once();
    let chord = props.shortcut.as_deref().and_then(|text| {
        let chord = Chord::parse(text, apple);
        if chord.is_none() {
            warned.once(text, || format!(
                "ActionIcon: `shortcut` {text:?} is not a chord `use_hotkeys` takes, so it is not announced or shown."
            ));
        }
        chord
    });
    let keyshortcuts = chord.as_ref().map(aria_keyshortcuts);
    if !props.tooltip.unwrap_or(false) {
        return action_icon(props, keyshortcuts);
    }
    let keys = chord
        .as_ref()
        .map(|chord| chord_words(chord, apple, &current_localization().shortcut_help));
    let mut label = props.aria_label.clone();
    if !label.trim().is_empty() && opens_new_tab(&props, &group) {
        label = format!("{label} {}", current_localization().anchor.new_tab);
    }
    let open_delay = try_consume_context::<TooltipOpenDelay>().map(|delay| delay.0);
    rsx! {
        Tooltip {
            open_delay,
            label: rsx! {
                "{label}"
                if let Some(keys) = keys {
                    " "
                    {chord_kbd(keys)}
                }
            },
            {action_icon(props, keyshortcuts)}
        }
    }
}

/// A live `_blank` link that says so: not a disabled one, which drops its `href`.
fn opens_new_tab(props: &ActionIconProps, group: &ButtonGroupContext) -> bool {
    let disabled = props.disabled.or(group.disabled).unwrap_or(false);
    props.to.as_ref().is_some()
        && !disabled
        && wants_new_tab_hint(props.target.as_deref(), props.new_tab_hint)
}

fn action_icon(mut props: ActionIconProps, keyshortcuts: Option<String>) -> Element {
    let group = use_button_group();
    let own_selected = props.selected.is_some();
    fill_from_group(&mut props, &group);
    let item = use_toolbar_item();
    let is_link = props.to.as_ref().is_some();
    // A disabled `Fieldset` reaches no link, as a native one (todo 3873).
    let in_disabled_group = use_fieldset_disabled() && !is_link;
    let disabled = props.disabled.unwrap_or(false) || in_disabled_group;
    let selectable = props.selected.is_some();
    let selected = props.selected.unwrap_or(false);
    let loading = props.loading.unwrap_or(false) && !is_link;
    // A toolbar keeps its disabled items in the arrow order.
    let soft_disabled = disabled && props.focusable_when_disabled.unwrap_or(item.is_some());
    // An explicit `false` takes a toolbar item out of the arrow order too.
    let item = item.filter(|_| !disabled || soft_disabled);

    // Once per instance, as the bad-chord warning.
    let warned = use_warn_once();
    let warn_once = |when: bool, message: &'static str| {
        if when {
            warned.once(message, || message.to_string());
        }
    };
    warn_once(
        is_link && props.loading == Some(true),
        "ActionIcon: `loading` is ignored on a link - an `<a>` has nothing to wait for.",
    );
    let glyph = match (props.icon, props.children.take()) {
        (Some(icon), children) => {
            warn_once(
                children.is_some(),
                "ActionIcon: both `icon` and `children` set; `children` is ignored.",
            );
            rsx! { Pictogram { icon } }
        }
        (None, children) => children.unwrap_or_else(VNode::empty),
    };
    warn_once(
        selectable && is_link,
        "ActionIcon: a link keeps the `selected` look but not `aria-pressed`, which `<a>` has no use for.",
    );

    let has_variant_styling = props.variant.as_ref().is_some() || props.color.as_ref().is_some();
    let theme = use_theme();
    let variant = props.variant.copied_or(theme.action_icon.variant);
    let new_tab_icon = props.new_tab_icon.unwrap_or(theme.anchor.new_tab_icon);
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
    // A blank own label would override a spread `aria-label` (todo 2336).
    let aria_label = (!props.aria_label.trim().is_empty()).then(|| props.aria_label.clone());
    let ripple = use_ripple();
    let showing = ripple.showing();

    let states = props
        .states
        .unwrap_or_default()
        .with("disabled", disabled)
        .with("loading", loading)
        .with("checked", selected)
        .with(variant.state_name(), has_variant_styling);
    // Unless a wrapper (`Burger`) set it for a size it computes.
    let states = if is_compact(props.size.as_ref()) {
        states.with(ACTION_ICON_COMPACT, true)
    } else {
        states
    };
    let states = if grows_on_touch(props.size.as_ref(), theme.action_icon.size) {
        states.with(ACTION_ICON_GROWS, true)
    } else {
        states
    };
    let states: Input<States> = match showing.as_ref() {
        Some(ripple) => states.with(ripple.state(), true),
        None => states,
    }
    .into();

    // `value` is also the `<button>`'s own attribute, which the prop shadows.
    let native_value = props.value.clone();
    let handle_click = move |event: Event<MouseData>| {
        if let Some(item) = item {
            item.take_stop();
        }
        // Else a busy `type="submit"` still submits, as on `Button`.
        // Android's WebView sends a tap on a disabled button's child a click (990).
        if loading || disabled {
            event.prevent_default();
            return;
        }
        if props.ripple {
            ripple.press(&event);
        }
        if let Some(onclick) = &props.onclick {
            onclick.call(event);
        }
        // A button with its own `selected` is not the group's to toggle.
        if let (false, Some(value)) = (own_selected, &props.value) {
            group.toggle(value);
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
            let tabindex = match item {
                Some(item) => item.tabindex(),
                None if soft_disabled => "0",
                None => "-1",
            };
            return boxed
                .attr_default("role", "link")
                .attr("aria-label", aria_label)
                .attr("aria-disabled", "true")
                .attr("aria-keyshortcuts", keyshortcuts)
                .attr("tabindex", tabindex)
                .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
                .render(HtmlTag::A, props.attributes, glyph);
        }

        // The name replaces the content, so the new-tab cue goes into it.
        let hint = wants_new_tab_hint(props.target.as_deref(), props.new_tab_hint);
        let aria_label = if hint {
            let new_tab = current_localization().anchor.new_tab;
            aria_label.map(|label| format!("{label} {new_tab}"))
        } else {
            aria_label
        };
        let mut attributes = props.attributes;
        attributes.extend(item.into_iter().flat_map(ToolbarItem::attributes));
        // An explicit `None` would strip a spread `aria-label` in the DOM (a blank own label).
        attributes.extend(aria_label.map(|label| Attribute::new("aria-label", label, None, false)));
        return rsx! {
            InternalAnchor {
                to,
                target: props.target,
                class: props.class,
                sx: props.sx,
                framework_sx: &ACTION_ICON_BASE_SX,
                states,
                variables,
                "aria-keyshortcuts": keyshortcuts,
                attributes,
                {glyph}
                if hint && new_tab_icon {
                    span { "data-slot": "new-tab", "aria-hidden": "true",
                        Glyph { slot: IconSlot::ExternalLink, icon: lucide::external_link::outlined }
                    }
                }
            }
        };
    }

    let children = if loading {
        rsx! {
            span { "data-slot": "glyph", {glyph} }
            if let Some(loader) = props.loader {
                span { "data-slot": "spinner", {loader} }
            } else {
                Loader { "data-slot": "spinner", color: "currentColor" }
            }
        }
    } else {
        glyph
    };
    // Mounted by the first press, so an untouched icon's markup stays as it was.
    let children = rsx! {
        {children}
        if showing.is_some() {
            span { "data-ripple": "", aria_hidden: "true" }
        }
    };

    // No native `disabled`, as on `Button`: it would drop a focused icon's focus.
    let tabindex = match item {
        Some(item) => Some(item.tabindex()),
        None => (disabled && !soft_disabled).then_some("-1"),
    };
    boxed
        .event("onclick", handle_click)
        .attr("aria-label", aria_label)
        .attr("aria-keyshortcuts", keyshortcuts)
        .attr("tabindex", tabindex)
        .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
        .attr("aria-busy", loading.then_some("true"))
        .attr("aria-disabled", (loading || disabled).then_some("true"))
        .attr("value", native_value)
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
            tooltip: None,
            shortcut: None,
            disabled: None,
            focusable_when_disabled: None,
            selected: None,
            loading: None,
            loader: None,
            ripple: true,
            value: None,
            onclick: None,
            to: Input::None,
            target: None,
            new_tab_hint: true,
            new_tab_icon: None,
            icon: None,
            children: None,
        }
    }

    /// Todo 4648: a re-render with the same bad chord does not warn again.
    #[test]
    fn a_bad_shortcut_warns_once_across_renders() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        use crate::{LiberoProvider, utils::take_warnings};

        static RENDERS: AtomicUsize = AtomicUsize::new(0);
        fn app() -> Element {
            let aria_label = format!("Bold {}", RENDERS.fetch_add(1, Ordering::Relaxed));
            rsx! {
                LiberoProvider {
                    ActionIcon { aria_label, shortcut: "hyper+b", tooltip: true, "B" }
                }
            }
        }
        take_warnings();
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        for _ in 0..2 {
            dom.mark_dirty(ScopeId::APP);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        let bad: Vec<_> = take_warnings()
            .into_iter()
            .filter(|w| w.contains("hyper+b"))
            .collect();
        assert!(RENDERS.load(Ordering::Relaxed) >= 3);
        assert_eq!(bad.len(), 1, "{bad:?}");
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

    /// Todo 3611: a `rem` or `em` length is compact up to `sm`'s 30px, as a `px` one is.
    #[test]
    fn a_rem_or_em_length_is_compact_up_to_the_sm_box() {
        let compact = |length: &str| is_compact(Some(&ThemeAwareValue::String(length.into())));
        assert!(compact("24px") && compact("1.5rem") && compact("1.5em") && compact("1.875rem"));
        assert!(!compact("31px") && !compact("2rem") && !compact("auto"));
    }
}
