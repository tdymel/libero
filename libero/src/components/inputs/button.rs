use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        Box, Input, States,
        common::{base_props, focus_ring_sx},
        navigation::InternalAnchor,
    },
    hooks::{use_css, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{BUTTON_RIPPLE_ANIMATION, ButtonDefaults, Color, ColorShade, ColorValue, Size},
};

// `Input<NavigationTarget>` rather than a bare required field (like
// `Anchor`'s `to`), since a Button is only a link when this is actually set -
// costs the ergonomic direct `to: Route::Foo {}` Anchor gets (needs
// `NavigationTarget::from(Route::Foo {})` instead), since Dioxus's
// `#[props(into)]` can't chain a foreign conversion through an `Option`/our
// own wrapper at once.
impl From<&str> for Input<NavigationTarget> {
    fn from(value: &str) -> Self {
        Input::Value(NavigationTarget::from(value))
    }
}

impl From<String> for Input<NavigationTarget> {
    fn from(value: String) -> Self {
        Input::Value(NavigationTarget::from(value))
    }
}

impl From<NavigationTarget> for Input<NavigationTarget> {
    fn from(value: NavigationTarget) -> Self {
        Input::Value(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ripple {
    id: u64,
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Filled,
    Outlined,
    Text,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Outlined
    }
}

impl From<&str> for ButtonVariant {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "filled" => Self::Filled,
            "text" => Self::Text,
            "outlined" | "outline" => Self::Outlined,
            _ => Self::Outlined,
        }
    }
}

impl From<String> for ButtonVariant {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for Input<ButtonVariant> {
    fn from(value: &str) -> Self {
        Input::Value(ButtonVariant::from(value))
    }
}

impl From<String> for Input<ButtonVariant> {
    fn from(value: String) -> Self {
        Input::Value(ButtonVariant::from(value))
    }
}

// The default shade used for a bare color (e.g. "primary") when no explicit
// shade is given. Kept a step darker than the library-wide default shade (5)
// since buttons need to stand out more than plain text/borders do.
const BUTTON_DEFAULT_SHADE: ColorShade = ColorShade::S6;
// Light tint used as the hover background for the outlined/text variants,
// mirroring Mantine's "subtle" hover treatment. `pub(crate)` since
// `ActionIcon` reuses this for its own outlined/transparent hover.
pub(crate) const BUTTON_HOVER_TINT_SHADE: ColorShade = ColorShade::S1;

// A bare theme color name (e.g. "primary") has no shade of its own, so it's
// resolved to our own default shade here rather than the sx pipeline's
// generic default (5). Anything else - an explicit shade/contrast, or a
// literal value like "red"/#123456/rgb(...) - passes through unchanged and
// is resolved by the normal sx-to-css pipeline. Only a genuinely unset
// `color` falls back to the library's default color.
fn button_base_color(value: Option<&ThemeAwareValue>) -> ThemeAwareValue {
    match value {
        None => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, BUTTON_DEFAULT_SHADE))
        }
        Some(ThemeAwareValue::Color(color)) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, BUTTON_DEFAULT_SHADE))
        }
        Some(other) => other.clone(),
    }
}

// A hover darken/tint and an auto-contrast text color both need a resolved
// theme shade to compute against; a literal color has neither available.
fn button_contrast_color(base: &ThemeAwareValue) -> Option<ThemeAwareValue> {
    match base {
        ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) => Some(
            ThemeAwareValue::ColorValue(ColorValue::Contrast(*color, *shade)),
        ),
        _ => None,
    }
}

// `shade_fn` picks the hover shade relative to the base's own shade (e.g.
// darker for `Filled`, or a fixed light tint for `Outlined`/`Text`).
// `pub(crate)` - `ActionIcon` reuses this for the same reason.
pub(crate) fn button_hover_sx(
    base: &ThemeAwareValue,
    shade_fn: impl Fn(ColorShade) -> ColorShade,
) -> Option<Sx> {
    match base {
        ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) => Some(sx().background(
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, shade_fn(*shade))),
        )),
        _ => None,
    }
}

static BUTTON_BASE_SX: StaticSx = StaticSx::new(|| {
    ButtonDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .position("relative")
        .overflow("hidden")
        .border_style("solid")
        .border_width("1px")
        .font_weight("600")
        .cursor("pointer")
        .user_select("none")
        .white_space("nowrap")
        .text_decoration("none")
        .outline("none")
        .when(
            "disabled",
            // pointer-events: none also stops the variant's :hover styles from
            // triggering, since a disabled button no longer receives pointer events.
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
        // Only shown for keyboard focus (not on mouse click), since the base
        // outline is suppressed above and re-added here just for :focus-visible.
        .focus_visible(focus_ring_sx())
});

fn button_variant_sx(variant: ButtonVariant, base: ThemeAwareValue) -> Sx {
    match variant {
        ButtonVariant::Filled => {
            let contrast = button_contrast_color(&base);
            let hover = button_hover_sx(&base, ColorShade::darker);
            let result = sx().background(base.clone()).border_color(base);
            let result = match contrast {
                Some(contrast) => result.color(contrast),
                None => result,
            };
            match hover {
                Some(hover) => result.hover(hover),
                None => result,
            }
        }
        ButtonVariant::Outlined => {
            let hover = button_hover_sx(&base, |_| BUTTON_HOVER_TINT_SHADE);
            let result = sx()
                .background("transparent")
                .border_color(base.clone())
                .color(base);
            match hover {
                Some(hover) => result.hover(hover),
                None => result,
            }
        }
        ButtonVariant::Text => {
            let hover = button_hover_sx(&base, |_| BUTTON_HOVER_TINT_SHADE);
            let result = sx()
                .background("transparent")
                .border_color("transparent")
                .color(base);
            match hover {
                Some(hover) => result.hover(hover),
                None => result,
            }
        }
    }
}

base_props! {
    pub struct ButtonProps {
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        variant: Input<ButtonVariant>,
        /// Corner radius - `theme.button.radius` by default, independent
        /// of `size`.
        #[props(default, into)]
        radius: Input<Size>,
        #[props(default, into)]
        size: Input<Size>,
        #[props(default)]
        full_width: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        #[props(default)]
        onclick: EventHandler<MouseEvent>,
        /// Renders as a link (router-aware, like `Anchor`) instead of a
        /// `<button>` when set. No ripple/`onclick` in that case - see the note
        /// above `is_link` below.
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
    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let color = button_base_color(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);
    let full_width = props.full_width.unwrap_or(false);

    // Only one ripple is shown at a time; a new click simply overrides the last one.
    let mut ripple_signal = use_signal(|| None::<Ripple>);
    let mut next_ripple_id = use_signal(|| 0u64);

    let size = props.size.as_ref().copied().unwrap_or(theme.button.size);
    let radius = props
        .radius
        .as_ref()
        .copied()
        .unwrap_or(theme.button.radius);

    let dynamic_sx = button_variant_sx(variant, color)
        .apply_if(full_width.then_some(()), |sx, ()| sx.width("100%"));
    let dynamic_class = use_css(&dynamic_sx, CssLayer::UserDynamic);
    let class = props.class.unwrap_or_default().with(dynamic_class);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("disabled", disabled)
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true);

    let handle_click = move |event: Event<MouseData>| {
        let point = event.element_coordinates();
        let id = next_ripple_id();
        next_ripple_id += 1;
        ripple_signal.set(Some(Ripple {
            id,
            x: point.x,
            y: point.y,
        }));
        props.onclick.call(event);
    };

    let ripple_span = rsx! {
        if let Some(ripple) = ripple_signal() {
            Box {
                component: "span",
                key: "{ripple.id}",
                style: "position:absolute;left:{ripple.x}px;top:{ripple.y}px;width:300%;height:300%;border-radius:50%;background:currentColor;opacity:0.3;transform:translate(-50%, -50%) scale(0);animation:{BUTTON_RIPPLE_ANIMATION} 550ms ease-out forwards;pointer-events:none;",
                onanimationend: move |_| ripple_signal.set(None),
            }
        }
    };

    // `InternalAnchor` has no `onclick`, since ripple only makes sense for a
    // real <button> - a link-mode Button loses the ripple, and `onclick`
    // itself is never called, only real navigation happens.
    if let Some(to) = props.to.as_ref().cloned() {
        // A disabled link keeps looking/behaving like a disabled control (it
        // just doesn't natively support the `disabled` attribute like
        // <button> does): no `to` at all stops navigation entirely,
        // `aria-disabled`/`tabindex` keep it out of the a11y tree and tab
        // order. Can't go through `InternalAnchor` for this - it always
        // resolves to a real, working link.
        if disabled {
            return rsx! {
                Box {
                    component: "a",
                    class,
                    sx: props.sx,
                    states,
                    framework_sx: &BUTTON_BASE_SX,
                    "aria-disabled": "true",
                    tabindex: "-1",
                    attributes: props.attributes,
                    {props.children}
                }
            };
        }

        return rsx! {
            InternalAnchor {
                to,
                target: props.target,
                class,
                sx: props.sx,
                framework_sx: &BUTTON_BASE_SX,
                states,
                attributes: props.attributes,
                {props.children}
            }
        };
    }

    rsx! {
        Box {
            component: "button",
            class,
            sx: props.sx,
            states,
            framework_sx: &BUTTON_BASE_SX,
            onclick: handle_click,
            disabled,
            r#type: "button",
            attributes: props.attributes,
            {ripple_span}
            {props.children}
        }
    }
}
