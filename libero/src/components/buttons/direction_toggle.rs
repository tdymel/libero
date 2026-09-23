use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, HtmlTag, Input, Variant, base_props, use_button_group},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{use_direction, use_localization, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    tokens::Direction,
};

/// The glyph's box, `ThemeToggle`'s share of the button.
static GLYPH_SX: StaticSx =
    StaticSx::new(|| sx().display("inline-flex").width("55%").height("55%"));

base_props! {
    pub struct DirectionToggleProps {
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Names the press, given the direction it turns to. Replaces the localized names.
        #[props(default)]
        label: Option<Callback<Direction, String>>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// An icon button that flips the app's text direction between LTR and RTL.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::DirectionToggle;
/// # fn app() -> Element {
/// rsx! { DirectionToggle {} }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/direction-toggle>
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
    let group = use_button_group();
    // Unset, `Button`'s `sm`: a toolbar control, not a page action (1069).
    let size = props
        .size
        .clone()
        .into_option()
        .or(group.size.map(ThemeAwareValue::Size))
        .unwrap_or(ThemeAwareValue::Size(crate::theme::Size::Sm));
    let variant = Input::Value(
        props
            .variant
            .copied_or(group.variant.unwrap_or(theme.direction_toggle.variant)),
    );
    let color = props
        .color
        .into_option()
        .or(group.color)
        .unwrap_or_else(|| theme.direction_toggle.color.into());
    // A pilcrow over an arrow pointing where a press turns the text.
    let (slot, icon) = match next {
        Direction::Rtl => (IconSlot::TextDirectionRtl, lucide::pilcrow_left::outlined),
        Direction::Ltr => (IconSlot::TextDirectionLtr, lucide::pilcrow_right::outlined),
    };
    let glyph = use_box().framework_sx(&GLYPH_SX).prepare().render(
        HtmlTag::Span,
        Vec::new(),
        rsx! { Glyph { slot, icon } },
    );

    rsx! {
        ActionIcon {
            aria_label,
            onclick: move |_| direction.toggle(),
            variant,
            color,
            size: size.clone(),
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
