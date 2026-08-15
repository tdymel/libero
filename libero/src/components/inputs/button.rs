use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::class_list},
    context::use_theme,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{BUTTON_RIPPLE_ANIMATION, ButtonDefaults, Color, ColorShade, ColorValue, Size},
};

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
// mirroring Mantine's "subtle" hover treatment.
const BUTTON_HOVER_TINT_SHADE: ColorShade = ColorShade::S1;

fn button_color_parts(value: Option<&ThemeAwareValue>) -> (Color, ColorShade) {
    match value {
        Some(ThemeAwareValue::Color(color)) => (*color, BUTTON_DEFAULT_SHADE),
        Some(ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade))) => (*color, *shade),
        Some(ThemeAwareValue::ColorValue(ColorValue::Contrast(color, shade))) => (*color, *shade),
        _ => (Color::Primary, BUTTON_DEFAULT_SHADE),
    }
}

static BUTTON_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
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
        .and(ButtonDefaults::radius_sx())
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
        .focus_visible(
            sx().outline("2px solid var(--lsx-primary-6)")
                .outline_offset("2px"),
        )
});

fn button_variant_sx(variant: ButtonVariant, color: Color, shade: ColorShade) -> Sx {
    let base = ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade));

    match variant {
        ButtonVariant::Filled => {
            let contrast = ThemeAwareValue::ColorValue(ColorValue::Contrast(color, shade));
            let hover_bg = ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade.darker()));
            sx().background(base.clone())
                .border_color(base)
                .color(contrast)
                .hover(sx().background(hover_bg))
        }
        ButtonVariant::Outlined => {
            let tint =
                ThemeAwareValue::ColorValue(ColorValue::Shade(color, BUTTON_HOVER_TINT_SHADE));
            sx().background("transparent")
                .border_color(base.clone())
                .color(base)
                .hover(sx().background(tint))
        }
        ButtonVariant::Text => {
            let tint =
                ThemeAwareValue::ColorValue(ColorValue::Shade(color, BUTTON_HOVER_TINT_SHADE));
            sx().background("transparent")
                .border_color("transparent")
                .color(base)
                .hover(sx().background(tint))
        }
    }
}

fn get_size_sx(size: Size) -> &'static Sx {
    static XS: StaticSx = StaticSx::new(ButtonDefaults::xs_sx);
    static SM: StaticSx = StaticSx::new(ButtonDefaults::sm_sx);
    static MD: StaticSx = StaticSx::new(ButtonDefaults::md_sx);
    static LG: StaticSx = StaticSx::new(ButtonDefaults::lg_sx);
    static XL: StaticSx = StaticSx::new(ButtonDefaults::xl_sx);

    match size {
        Size::Xs => &XS,
        Size::Sm => &SM,
        Size::Md => &MD,
        Size::Lg => &LG,
        Size::Xl => &XL,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    #[props(default)]
    class: Option<String>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
    #[props(default, into)]
    color: Input<ThemeAwareValue>,
    #[props(default, into)]
    variant: Input<ButtonVariant>,
    #[props(default, into)]
    radius: Input<ThemeAwareValue>,
    #[props(default, into)]
    size: Input<ThemeAwareValue>,
    #[props(default)]
    full_width: Option<bool>,
    #[props(default)]
    disabled: Option<bool>,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
    #[props(default)]
    href: Option<String>,
    #[props(default)]
    target: Option<String>,
    children: Element,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let theme = use_theme();
    let variant = props.variant.as_ref().copied().unwrap_or_default();
    let (color, shade) = button_color_parts(props.color.as_ref());
    let disabled = props.disabled.unwrap_or(false);
    let full_width = props.full_width.unwrap_or(false);

    // Only one ripple is shown at a time; a new click simply overrides the last one.
    let mut ripple_signal = use_signal(|| None::<Ripple>);
    let mut next_ripple_id = use_signal(|| 0u64);

    // The default radius/size are baked into the framework layer as theme CSS
    // vars (see ButtonDefaults::radius_sx / get_size_sx), so a plain `sx` prop
    // override still works. Only an explicit prop should win over that at the
    // (higher-priority) dynamic layer.
    let explicit_radius = match props.radius.as_ref() {
        Some(ThemeAwareValue::Size(size)) => Some(*size),
        _ => None,
    };
    let size = match props.size.as_ref() {
        Some(ThemeAwareValue::Size(size)) => *size,
        _ => theme.button.size,
    };

    let size_class = crate::context::use_sx(get_size_sx(size), crate::SxLayer::Framework);

    let dynamic_sx = button_variant_sx(variant, color, shade)
        .apply_if(explicit_radius, |sx, radius| {
            sx.border_radius(ThemeAwareValue::Size(radius))
        })
        .apply_if(full_width.then_some(()), |sx, ()| sx.width("100%"));
    let dynamic_class = crate::context::use_sx(&dynamic_sx, crate::SxLayer::UserDynamic);

    let class = class_list([props.class, size_class, dynamic_class]);

    let states = props
        .states
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("disabled", disabled);

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

    // A disabled link keeps looking/behaving like a disabled control (it just
    // doesn't natively support the `disabled` attribute like <button> does):
    // dropping `href` stops navigation, `aria-disabled`/`tabindex` keep it out
    // of the a11y tree and tab order.
    let is_link = props.href.is_some();
    let component = if is_link { "a" } else { "button" };
    let href = is_link
        .then(|| (!disabled).then_some(props.href.unwrap()))
        .flatten();
    let target = is_link.then_some(props.target).flatten();
    let aria_disabled = is_link.then(|| disabled.then_some("true")).flatten();
    let tabindex = is_link.then(|| disabled.then_some("-1")).flatten();
    let button_disabled = (!is_link).then_some(disabled);
    let button_type = (!is_link).then_some("button".to_string());

    rsx! {
        Box {
            component: component,
            class: class,
            sx: props.sx,
            states: states,
            framework_sx: &BUTTON_BASE_SX,
            onclick: handle_click,
            href: href,
            target: target,
            "aria-disabled": aria_disabled,
            tabindex: tabindex,
            disabled: button_disabled,
            r#type: button_type,
            attributes: props.attributes,
            {ripple_span}
            {props.children}
        }
    }
}
