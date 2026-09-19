use dioxus::prelude::*;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{HtmlTag, Input, TextDirectionIcon, Variant, base_props},
        layout::use_box,
    },
    hooks::{use_direction, use_localization, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    tokens::Direction,
};

/// The glyph's box, `ThemeToggle`'s share of the button.
static GLYPH_SX: StaticSx =
    StaticSx::new(|| sx().display("inline-flex").width("55%").height("55%"));

base_props! {
    pub struct DirectionToggleProps {
        /// Unset, the theme's
        /// [`DirectionToggleDefaults::variant`](crate::theme::DirectionToggleDefaults).
        #[props(default, into)]
        variant: Input<Variant>,
        /// Unset, the theme's
        /// [`DirectionToggleDefaults::color`](crate::theme::DirectionToggleDefaults).
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Replaces the localization's two names. Given the direction a press
        /// turns the text to, it names what the press does.
        #[props(default)]
        label: Option<Callback<Direction, String>>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// An icon button that turns the app's text between left to right and right
/// to left, through [`use_direction`](crate::hooks::use_direction). The glyph
/// and the name both say where a press goes: the arrow points the way the
/// text will run.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::DirectionToggle;
/// # fn app() -> Element {
/// rsx! { DirectionToggle {} }
/// # }
/// ```
#[component]
pub fn DirectionToggle(props: DirectionToggleProps) -> Element {
    let theme = use_theme();
    let direction = use_direction();
    let labels = use_localization().direction_toggle;

    let next = direction.get().flipped();
    let aria_label = match props.label {
        Some(label) => label.call(next),
        None => match next {
            Direction::Rtl => labels.to_rtl,
            Direction::Ltr => labels.to_ltr,
        }
        .to_string(),
    };
    let variant = Input::Value(props.variant.copied_or(theme.direction_toggle.variant));
    let color = props
        .color
        .into_option()
        .unwrap_or_else(|| theme.direction_toggle.color.into());
    let glyph = use_box().framework_sx(&GLYPH_SX).prepare().render(
        HtmlTag::Span,
        Vec::new(),
        rsx! { TextDirectionIcon { to_rtl: next == Direction::Rtl } },
    );

    rsx! {
        ActionIcon {
            aria_label,
            onclick: move |_| direction.toggle(),
            variant,
            color,
            size: props.size.clone(),
            radius: props.radius.clone(),
            disabled: props.disabled,
            class: props.class.clone(),
            sx: props.sx.clone(),
            states: props.states.clone(),
            attributes: props.attributes.clone(),
            {glyph}
        }
    }
}
