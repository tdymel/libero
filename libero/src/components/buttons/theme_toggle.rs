use dioxus::prelude::*;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            ChevronDownIcon, HtmlTag, Input, MoonIcon, SunIcon, SystemSchemeIcon, Variant,
            base_props,
        },
        layout::use_box,
        overlay::{Menu, MenuEntry, MenuItem, use_menu},
    },
    hooks::{Align, use_color_scheme, use_localization, use_theme, use_theme_set},
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
                // Narrower than the toggle, never under WCAG 2.5.8's 24px.
                .width(format!(
                    "max(24px, calc({} * 0.75))",
                    ACTION_ICON_SIZE.overridable()
                )),
        )
});

base_props! {
    pub struct ThemeToggleProps {
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
        /// Names the press, given the setting it moves to. Replaces the localized names.
        #[props(default)]
        label: Option<Callback<ColorSchemeSetting, String>>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// An icon button that cycles the app's colour scheme, optionally with a theme-set picker.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::{components::ThemeToggle, theme::ThemeSet};
/// # fn app() -> Element {
/// rsx! {
///     ThemeToggle {}
///     ThemeToggle { themes: ThemeSet::CATALOGUE }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/theme-toggle>
#[component]
pub fn ThemeToggle(props: ThemeToggleProps) -> Element {
    let theme = use_theme();
    let scheme = use_color_scheme();
    let theme_set = use_theme_set();
    let menu = use_menu();
    let labels = use_localization().theme_toggle;

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
    let variant = Input::Value(props.variant.copied_or(theme.theme_toggle.variant));
    let color = props
        .color
        .into_option()
        .unwrap_or_else(|| theme.theme_toggle.color.into());

    let glyph = use_box().framework_sx(&GLYPH_SX).prepare().render(
        HtmlTag::Span,
        Vec::new(),
        // Where a press goes: a sun switches to light.
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
