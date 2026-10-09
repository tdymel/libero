use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, Part, Parts, Variant, attr, base_props, parts_enum,
            parts_under_sx, use_button_group,
        },
        layout::use_box,
        overlay::{Menu, MenuEntry, MenuItem, MenuPart, use_menu},
    },
    context::IconSlot,
    hooks::{
        Align, ColorSchemeHandle, use_color_scheme, use_localization, use_theme, use_theme_set,
    },
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ACTION_ICON_SIZE, ColorSchemeSetting, ThemeSet},
};

/// `ActionIcon` stretches any `svg` to its whole box; this leaves a margin around the glyph.
static GLYPH_SX: StaticSx =
    StaticSx::new(|| sx().display("inline-flex").width("55%").height("55%"));

/// The pair as one control; the focused half is lifted so its ring stays on top.
/// The chevron sits inside `Menu`'s trigger `div`.
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
                // Never under 24px (WCAG 2.5.8), so at the smallest size it matches the toggle.
                .width(format!(
                    "max(24px, calc({} * 0.75))",
                    ACTION_ICON_SIZE.overridable()
                )),
        )
});

parts_enum! {
    /// [`ThemeSwitcher`]'s inner parts. Without `themes` the toggle is the root itself,
    /// styled by `sx`; `Toggle`, `Picker` and `Chevron` exist only with `themes`.
    pub enum ThemeSwitcherPart {
        /// The toggle's sun, moon or system glyph.
        Icon = "icon" => "& > [data-slot='icon'], & > [data-slot='toggle'] > [data-slot='icon']",
        /// The scheme button beside the picker.
        Toggle = "toggle" => "& > [data-slot='toggle']",
        /// The button that opens the theme-set menu.
        Picker = "picker" => "& > div > [data-slot='picker']",
        Chevron = "chevron" => "& > div > [data-slot='picker'] > [data-slot='chevron']",
    }
}

base_props! {
    parts(ThemeSwitcherPart);
    pub struct ThemeSwitcherProps {
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Adds a chevron beside the toggle that opens a menu of these theme sets.
        #[props(default)]
        themes: Option<&'static [&'static ThemeSet]>,
        /// Adds following the system to the cycle. Off, a press flips between light and dark,
        /// and flipping to the system's own scheme follows the system again.
        #[props(default)]
        with_system: Option<bool>,
        /// Names the press, given the scheme it shows next: the next cycle step with
        /// `with_system`, else always `Light` or `Dark`, even when the press stores `System`.
        /// Replaces the localized names.
        #[props(default)]
        label: Option<Callback<ColorSchemeSetting, String>>,
        #[props(default)]
        disabled: Option<bool>,
        /// The theme-set menu's `parts`: the menu is portaled, out of `parts`' reach.
        #[props(default, into)]
        menu_parts: Input<Parts<MenuPart>>,
    }
}

fn press(scheme: &ColorSchemeHandle, with_system: bool) {
    if with_system {
        scheme.cycle();
    } else {
        scheme.toggle();
    }
}

/// An icon button that switches the app's colour scheme, optionally with a theme-set picker.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::{components::ThemeSwitcher, theme::ThemeSet};
/// # fn app() -> Element {
/// rsx! {
///     ThemeSwitcher {}
///     ThemeSwitcher { with_system: true }
///     ThemeSwitcher { themes: ThemeSet::CATALOGUE }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/theme-switcher>
#[component]
pub fn ThemeSwitcher(props: ThemeSwitcherProps) -> Element {
    let theme = use_theme();
    let scheme = use_color_scheme();
    let theme_set = use_theme_set();
    let menu = use_menu();
    let labels = use_localization().theme_switcher;

    let with_system = props.with_system.unwrap_or(false);
    let next = if with_system {
        scheme.next_in_cycle()
    } else {
        scheme.resolved().flipped().into()
    };
    let aria_label = match props.label {
        Some(label) => label.call(next),
        None => match next {
            ColorSchemeSetting::Light => labels.to_light,
            ColorSchemeSetting::Dark => labels.to_dark,
            ColorSchemeSetting::System => labels.to_system,
        }
        .to_string(),
    };
    let group = use_button_group();
    let variant = Input::Value(
        props
            .variant
            .copied_or(group.variant.unwrap_or(theme.theme_switcher.variant)),
    );
    let color = props
        .color
        .into_option()
        .or(group.color)
        .unwrap_or_else(|| theme.theme_switcher.color.into());
    let disabled = props.disabled.or(group.disabled);

    let glyph = use_box()
        .framework_sx(&GLYPH_SX)
        .prepare()
        .attr("data-slot", ThemeSwitcherPart::Icon.slot())
        .render(
        HtmlTag::Span,
        Vec::new(),
        // Where a press goes: a sun switches to light.
        match next {
            ColorSchemeSetting::System => {
                rsx! { Glyph { slot: IconSlot::SystemScheme, icon: lucide::contrast::outlined } }
            }
            ColorSchemeSetting::Light => {
                rsx! { Glyph { slot: IconSlot::Sun, icon: lucide::sun::outlined } }
            }
            ColorSchemeSetting::Dark => {
                rsx! { Glyph { slot: IconSlot::Moon, icon: lucide::moon::outlined } }
            }
        },
    );
    let chevron = use_box()
        .framework_sx(&GLYPH_SX)
        .prepare()
        .attr("data-slot", ThemeSwitcherPart::Chevron.slot())
        .render(
            HtmlTag::Span,
            Vec::new(),
            rsx! { Glyph { slot: IconSlot::ChevronDown, icon: lucide::chevron_down::outlined } },
        );
    // A hook, so above the branch: the picker's wrapper is built either way.
    let split = use_box()
        .framework_sx(&SPLIT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .prepare()
        .attr("role", "group")
        .attr("aria-label", labels.group);

    let Some(sets) = props.themes else {
        return rsx! {
            ActionIcon {
                aria_label,
                onclick: move |_| press(&scheme, with_system),
                variant: variant.clone(),
                color: color.clone(),
                size: props.size.clone(),
                radius: props.radius.clone(),
                disabled: props.disabled,
                class: props.class.clone(),
                sx: parts_under_sx(&props.parts, props.sx.clone()),
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

    let mut picker_attributes = menu.a11y_attributes();
    picker_attributes.push(attr("data-slot", ThemeSwitcherPart::Picker.slot()));

    split.render(
        HtmlTag::Div,
        props.attributes.clone(),
        rsx! {
            ActionIcon {
                "data-slot": ThemeSwitcherPart::Toggle.slot(),
                aria_label,
                onclick: move |_| press(&scheme, with_system),
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
                disabled: disabled.unwrap_or(false),
                parts: props.menu_parts.clone(),
                ActionIcon {
                    aria_label: labels.picker,
                    variant,
                    color,
                    size: props.size.clone(),
                    radius: props.radius.clone(),
                    disabled: props.disabled,
                    attributes: picker_attributes,
                    {chevron}
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<ThemeSwitcherPart>(),
            [
                (
                    "icon",
                    "& > [data-slot='icon'], & > [data-slot='toggle'] > [data-slot='icon']"
                ),
                ("toggle", "& > [data-slot='toggle']"),
                ("picker", "& > div > [data-slot='picker']"),
                (
                    "chevron",
                    "& > div > [data-slot='picker'] > [data-slot='chevron']"
                ),
            ]
        );
    }
}
