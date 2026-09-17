use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, Variant,
        buttons::ActionIcon,
        common::{ChevronDownIcon, MoonIcon, SunIcon, SystemSchemeIcon, base_props},
        layout::use_box,
        overlay::{Menu, MenuEntry, MenuItem, use_menu},
    },
    hooks::{Align, use_color_scheme, use_localization, use_theme, use_theme_set},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ACTION_ICON_SIZE, ColorSchemeSetting, ThemeSet},
};

/// The glyph's box. `ActionIcon` stretches any `svg` to its whole box, so an
/// unsized glyph would be a 22px moon in a 24px button; this leaves the
/// margin Mantine's own scheme toggle has.
static GLYPH_SX: StaticSx =
    StaticSx::new(|| sx().display("inline-flex").width("55%").height("55%"));

/// The pair drawn as one control: the two halves share the seam, and the
/// focused one is lifted so its ring is not painted under its neighbour.
/// The toggle is a direct child; the chevron sits inside the `div` `Menu`
/// wraps its trigger in. Logical sides, so the seam follows `dir="rtl"`.
static SPLIT_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("stretch")
        .selector("& button", sx().position("relative"))
        .selector("& button:focus-visible", sx().z_index("1"))
        .selector(
            "& > button",
            sx().border_start_end_radius("0").border_end_end_radius("0"),
        )
        // One seam, not two borders side by side.
        .selector("& > div", sx().display("flex").margin_inline_start("-1px"))
        .selector(
            "& > div > button",
            sx().border_start_start_radius("0")
                .border_end_start_radius("0")
                // Narrower than the toggle, as a split button's arrow is,
                // but never under WCAG 2.5.8's 24px.
                .width(format!(
                    "max(24px, calc({} * 0.75))",
                    ACTION_ICON_SIZE.overridable()
                )),
        )
});

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
        /// Opts into the theme picker: a second button beside the toggle, a
        /// chevron that opens a menu of these sets with the active one
        /// checked. Unset, it is the toggle alone.
        #[props(default)]
        themes: Option<&'static [&'static ThemeSet]>,
        /// Replaces the localization's three toggle names. Given the setting a
        /// press moves to, it names what the press does. Runs during render,
        /// so it can read a live locale.
        #[props(default)]
        label: Option<Callback<ColorSchemeSetting, String>>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// An icon button that steps the app's colour scheme: following the
/// platform, then the scheme the platform is not showing, then the one it is,
/// then back to following it. The glyph and the name both say where a press
/// goes: a sun switches to light, a moon to dark, a half-filled disc back to
/// following the platform.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::ColorSchemeButton;
/// # fn app() -> Element {
/// rsx! { ColorSchemeButton {} }
/// # }
/// ```
///
/// With `themes`, it becomes a split button: the toggle, and beside it a
/// chevron opening a menu of theme sets. Two buttons, not one with a second
/// gesture - each keeps one job and one name, and both are reachable from
/// the keyboard. The pair is a named `group`, and `class`, `sx` and the extra
/// attributes land on it rather than on either half.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::{components::ColorSchemeButton, theme::ThemeSet};
/// # fn app() -> Element {
/// rsx! { ColorSchemeButton { themes: ThemeSet::CATALOGUE } }
/// # }
/// ```
#[component]
pub fn ColorSchemeButton(props: ColorSchemeButtonProps) -> Element {
    let theme = use_theme();
    let scheme = use_color_scheme();
    let theme_set = use_theme_set();
    let menu = use_menu();
    let labels = use_localization().color_scheme_button;

    let next = scheme.next_in_cycle();
    let aria_label = match props.label {
        Some(label) => label.call(next),
        None => match next {
            ColorSchemeSetting::Light => labels.to_light,
            ColorSchemeSetting::Dark => labels.to_dark,
            ColorSchemeSetting::System => labels.to_system,
        }
        .to_string(),
    };
    let variant = Input::Value(props.variant.copied_or(theme.color_scheme_button.variant));
    let color = props
        .color
        .into_option()
        .unwrap_or_else(|| theme.color_scheme_button.color.into());

    let glyph = use_box().framework_sx(&GLYPH_SX).prepare().render(
        HtmlTag::Span,
        Vec::new(),
        // Where a press goes, the same as the name: a sun switches to light.
        match next {
            ColorSchemeSetting::System => rsx! { SystemSchemeIcon {} },
            ColorSchemeSetting::Light => rsx! { SunIcon {} },
            ColorSchemeSetting::Dark => rsx! { MoonIcon {} },
        },
    );
    let chevron = use_box().framework_sx(&GLYPH_SX).prepare().render(
        HtmlTag::Span,
        Vec::new(),
        rsx! { ChevronDownIcon {} },
    );
    // A hook, so above the branch: the picker's wrapper is built either way.
    let split = use_box()
        .framework_sx(&SPLIT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .attr("role", "group")
        .attr("aria-label", labels.group);

    let Some(sets) = props.themes else {
        return rsx! {
            ActionIcon {
                aria_label,
                onclick: move |_| scheme.cycle(),
                variant: variant.clone(),
                color: color.clone(),
                size: props.size.clone(),
                radius: props.radius.clone(),
                disabled: props.disabled,
                class: props.class.clone(),
                sx: props.sx.clone(),
                states: props.states.clone(),
                attributes: props.attributes.clone(),
                {glyph}
            }
        };
    };

    let active = theme_set.name();
    let items = vec![MenuEntry::Group {
        label: labels.themes.to_string(),
        items: sets
            .iter()
            .map(|&set| {
                let theme_set = theme_set.clone();
                MenuItem::new(set.name())
                    .radio(set.name() == active)
                    .onselect(move |_| theme_set.set(set.clone()))
                    .into()
            })
            .collect(),
    }];

    split.render(
        HtmlTag::Div,
        props.attributes.clone(),
        rsx! {
            ActionIcon {
                aria_label,
                onclick: move |_| scheme.cycle(),
                variant: variant.clone(),
                color: color.clone(),
                size: props.size.clone(),
                radius: props.radius.clone(),
                disabled: props.disabled,
                {glyph}
            }
            Menu {
                state: menu,
                items,
                align: Align::End,
                disabled: props.disabled.unwrap_or(false),
                ActionIcon {
                    aria_label: labels.picker,
                    variant,
                    color,
                    size: props.size.clone(),
                    radius: props.radius.clone(),
                    disabled: props.disabled,
                    attributes: menu.a11y_attributes(),
                    {chevron}
                }
            }
        },
    )
}
