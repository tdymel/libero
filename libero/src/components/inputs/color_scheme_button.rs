use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, Variant,
        common::{MoonIcon, SunIcon, base_props},
        inputs::ActionIcon,
        layout::use_box,
    },
    hooks::{use_color_scheme, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::ColorScheme,
};

/// The glyph's box. `ActionIcon` stretches any `svg` to its whole box, so an
/// unsized glyph would be a 22px moon in a 24px button; this leaves the
/// margin Mantine's own scheme toggle has.
static GLYPH_SX: StaticSx =
    StaticSx::new(|| sx().display("inline-flex").width("55%").height("55%"));

base_props! {
    pub struct ColorSchemeButtonProps {
        /// Unset, the theme's
        /// [`ColorSchemeButtonDefaults::variant`](crate::theme::ColorSchemeButtonDefaults).
        #[props(default, into)]
        variant: Input<Variant>,
        /// Unset, the theme's
        /// [`ColorSchemeButtonDefaults::color`](crate::theme::ColorSchemeButtonDefaults).
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Replaces the theme's two labels. Given the scheme on screen, it
        /// names what a press does. Runs during render, so it can read a live
        /// locale.
        #[props(default)]
        label: Option<Callback<ColorScheme, String>>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// An icon button that flips the app between its light and dark theme: an
/// [`ActionIcon`] over [`use_color_scheme`], showing a moon in the light
/// scheme and a sun in the dark one.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::ColorSchemeButton;
/// # fn app() -> Element {
/// rsx! { ColorSchemeButton {} }
/// # }
/// ```
///
/// A press pins the other scheme only while it differs from the platform's,
/// so flipping back hands the choice to the platform again - an OS switch or
/// a devtools emulation is followed from then on. An app that wants an
/// explicit "follow the system" choice builds it from `use_color_scheme()`,
/// which is also the hook to reach for when a button is the wrong control.
#[component]
pub fn ColorSchemeButton(props: ColorSchemeButtonProps) -> Element {
    let theme = use_theme();
    let scheme = use_color_scheme();
    let showing = scheme.resolved();

    let aria_label = match props.label {
        Some(label) => label.call(showing),
        None => match showing {
            ColorScheme::Dark => theme.color_scheme_button.labels.to_light.to_string(),
            ColorScheme::Light => theme.color_scheme_button.labels.to_dark.to_string(),
        },
    };
    let variant = props.variant.copied_or(theme.color_scheme_button.variant);
    let color = props
        .color
        .into_option()
        .unwrap_or_else(|| theme.color_scheme_button.color.into());

    let glyph = use_box().framework_sx(&GLYPH_SX).prepare().render(
        HtmlTag::Span,
        Vec::new(),
        match showing {
            ColorScheme::Dark => rsx! { SunIcon {} },
            ColorScheme::Light => rsx! { MoonIcon {} },
        },
    );

    rsx! {
        ActionIcon {
            aria_label,
            onclick: move |_| scheme.toggle(),
            variant: Input::Value(variant),
            color,
            size: props.size,
            radius: props.radius,
            disabled: props.disabled,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
            {glyph}
        }
    }
}
