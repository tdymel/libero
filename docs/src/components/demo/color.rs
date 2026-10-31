use dioxus::prelude::*;
use libero::components::Pictogram;
use libero::{
    components::{
        Box, Button, ColorCode, ColorPicker, ColorSwatch, Flex, SliderChangeEvent, States,
        VisuallyHidden,
    },
    hooks::{use_direction, use_element, use_id},
    platform::ElementApi,
    sx::sx,
};
use pictogram_icons_lucide as lucide;

use super::{Control, UNSET, demo::border};

/// What the custom swatch opens on before anything was picked.
const FIRST_CUSTOM: u32 = 0x0ca678;

/// Every hue once around, so the closed custom swatch reads as "pick any".
const HUE_WHEEL: &str = "conic-gradient(red, yellow, lime, aqua, blue, magenta, red)";

/// Fill and tick color for one swatch. The unset swatch shows the component without the prop:
/// `Control::unset_swatch` names that color where it isn't the surface.
fn swatch(control: &Control, option: &str) -> (String, String) {
    match option {
        UNSET => {
            let fill = match control.unset_swatch.as_deref() {
                // currentColor inside the swatch button is the tick's own ink: paint it ink.
                Some("currentColor") => "ink".to_string(),
                Some(color) => color.to_string(),
                None => "surface".to_string(),
            };
            // A palette shade takes its own contrast tick, as that swatch does.
            let name = fill.split_once('.').map_or(fill.as_str(), |(name, _)| name);
            if control.options.iter().any(|option| option == name) {
                return swatch(control, &fill);
            }
            // An ink swatch needs a tick that reads on it.
            let tick = if fill == "ink" { "surface" } else { "ink" };
            (fill, tick.to_string())
        }
        // The palette's own contrast color, so the tick reads on every swatch.
        // A shade keeps its step: `muted.1` pairs with `muted-contrast.1`.
        color => match color.split_once('.') {
            Some((name, shade)) => (color.to_string(), format!("{name}-contrast.{shade}")),
            None => (color.to_string(), format!("{color}-contrast")),
        },
    }
}

/// The swatch fill: reaches the button's border, since the swatch is the
/// whole button.
fn fill() -> libero::sx::Sx {
    sx().width("100%")
        .height("100%")
        .border_radius("inherit")
        // A pale swatch needs an edge to read as a swatch at all.
        .border(border())
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .selector("& svg", sx().width("18px").height("18px"))
}

/// The swatch button. The variant's hover tint would show as a halo around
/// the fill.
fn swatch_button() -> libero::sx::Sx {
    sx().flex("1 1 0")
        .padding("0")
        // An invisible 24px press target (WCAG 2.5.8) for a swatch the row squeezes below it;
        // sideways it grows by at most half the row gap, so neighbouring pads never overlap.
        .position("relative")
        .overflow("visible")
        .selector(
            "&::before",
            sx().content("\"\"").position("absolute").inset(
                "min(0px, calc((100% - 24px) / 2)) \
                 max(calc(var(--lsx-spacing-xs) / -2), min(0px, calc((100% - 24px) / 2)))",
            ),
        )
        .hover(sx().background("transparent"))
}

/// The swatch an arrow key lands on, wrapping; `rtl` swaps left and right.
fn next_swatch(from: usize, count: usize, key: &Key, rtl: bool) -> Option<usize> {
    let (forward, back) = match rtl {
        false => (NamedKey::ArrowRight, NamedKey::ArrowLeft),
        true => (NamedKey::ArrowLeft, NamedKey::ArrowRight),
    };
    let last = count.checked_sub(1)?;
    match key {
        Key::Named(named) if *named == forward || *named == NamedKey::ArrowDown => {
            Some(if from >= last { 0 } else { from + 1 })
        }
        Key::Named(named) if *named == back || *named == NamedKey::ArrowUp => {
            Some(from.checked_sub(1).unwrap_or(last))
        }
        Key::Named(NamedKey::Home) => Some(0),
        Key::Named(NamedKey::End) => Some(last),
        _ => None,
    }
}

/// A swatch per theme color, then one opening a `ColorPicker` for any other. A swatch sets its
/// name, the picker a hex, so the code block prints what the caller would type.
#[component]
pub fn ColorControl(
    control: Control,
    /// Names the swatch row, for the group role.
    label: String,
    value: String,
    onchange: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    // Kept, not re-parsed from the emitted hex: a grey's hex loses the hue, and the thumb jumps to red.
    let mut custom = use_signal(|| {
        value
            .parse::<ColorCode>()
            .unwrap_or(ColorCode::hex(FIRST_CUSTOM))
    });
    let trigger = use_element();
    let row = use_element();
    let rtl = use_direction().is_rtl();
    let hint_id = use_id();

    let is_custom = !control.options.contains(&value);
    let swatches: Vec<(String, String, String)> = control
        .options
        .iter()
        .map(|option| {
            let (fill, tick) = swatch(&control, option);
            (option.clone(), fill, tick)
        })
        .collect();
    let has_custom = control.custom;
    // A custom value is a literal color, the one thing `ColorSwatch` draws.
    let custom_label = if is_custom {
        format!("Custom color, {value}")
    } else {
        "Custom color".to_string()
    };
    // One Tab stop: the checked swatch, else the first. The custom swatch comes last.
    let count = control.options.len() + usize::from(has_custom);
    let tab_stop = control
        .options
        .iter()
        .position(|option| *option == value)
        .or(is_custom.then_some(control.options.len()))
        .filter(|&at| at < count)
        .unwrap_or(0);
    let arrows = {
        let options = control.options.clone();
        move |event: KeyboardEvent| {
            if event
                .modifiers()
                .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META)
            {
                return;
            }
            let Some(next) = next_swatch(tab_stop, count, &event.key(), rtl) else {
                return;
            };
            event.prevent_default();
            open.set(false);
            // Arrows move and check in one press (APG radio group).
            match options.get(next) {
                Some(option) => onchange.call(option.clone()),
                None => onchange.call(custom().to_hex()),
            }
            let _ = row
                .query_selector(&format!("[data-swatch=\"{next}\"]"))
                .and_then(|swatch| swatch.focus());
        }
    };

    rsx! {
        Flex {
            direction: "column",
            gap: "xs",
            sx: sx().width("100%"),
            Flex {
                // `Flex` defaults to a column, and the swatches are a row.
                direction: "row",
                gap: "xs",
                sx: sx().width("100%"),
                role: "radiogroup",
                aria_label: "{label}",
                onmounted: row.mount(),
                onkeydown: arrows,
                // Radio `Button`s, not a `SegmentedControl`: swatches read as separate chips.
                for (at, (option, fill_color, tick_color)) in swatches.iter().enumerate() {
                    Button {
                        key: "{option}",
                        size: "sm",
                        variant: "text",
                        role: "radio",
                        aria_label: "{option}",
                        aria_checked: value == *option,
                        tabindex: if at == tab_stop { "0" } else { "-1" },
                        "data-swatch": "{at}",
                        states: States::default().with("checked", value == *option),
                        onclick: {
                            let option = option.clone();
                            move |_| {
                                open.set(false);
                                onchange.call(option.clone());
                            }
                        },
                        sx: swatch_button(),
                        Box {
                            sx: fill()
                                .background(fill_color.clone())
                                .color(tick_color.clone()),
                            if value == *option {
                                Pictogram { icon: lucide::check::outlined }
                            }
                        }
                    }
                }
                if has_custom {
                    // `display: contents` keeps the button the flex item; the div is Escape's focus handle.
                    div {
                        style: "display: contents",
                        onmounted: trigger.mount(),
                        VisuallyHidden { id: "{hint_id}", "Opens a color picker" }
                        Button {
                            size: "sm",
                            variant: "text",
                            role: "radio",
                            aria_label: "{custom_label}",
                            aria_checked: is_custom,
                            // `aria-haspopup` is not valid on a radio: a description says it instead.
                            "aria-describedby": "{hint_id}",
                            tabindex: if control.options.len() == tab_stop { "0" } else { "-1" },
                            "data-swatch": "{control.options.len()}",
                            onclick: move |_| {
                                let opening = !open();
                                open.set(opening);
                                // Opening is also picking: the swatch stands
                                // for the color the picker holds.
                                if opening {
                                    onchange.call(custom().to_hex());
                                }
                            },
                            sx: swatch_button(),
                            if is_custom {
                                ColorSwatch {
                                    color: custom(),
                                    with_shadow: false,
                                    // The swatch's own box is a size step; this one fills the
                                    // button like the theme swatches.
                                    sx: fill().min_width("0"),
                                    Pictogram { icon: lucide::check::outlined }
                                }
                            } else {
                                Box { sx: fill().background(HUE_WHEEL) }
                            }
                        }
                    }
                }
            }
            if has_custom && open() {
                div {
                    // The picker lets Escape bubble: close and hand focus back to the swatch.
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Named(NamedKey::Escape) {
                            event.stop_propagation();
                            open.set(false);
                            let _ = trigger.query_selector("button").and_then(|button| button.focus());
                        }
                    },
                    ColorPicker {
                        value: custom(),
                        full_width: true,
                        // The thumbs carry no name of their own.
                        saturation_label: format!("{label}: custom saturation and brightness"),
                        hue_label: format!("{label}: custom hue"),
                        oninput: move |event: SliderChangeEvent<ColorCode>| {
                            let color = event.value();
                            custom.set(color);
                            onchange.call(color.to_hex());
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::DemoValues;

    #[test]
    fn arrows_walk_the_swatches_and_flip_under_rtl() {
        let right = Key::Named(NamedKey::ArrowRight);
        let left = Key::Named(NamedKey::ArrowLeft);
        assert_eq!(next_swatch(0, 3, &right, false), Some(1));
        assert_eq!(next_swatch(0, 3, &left, false), Some(2));
        assert_eq!(next_swatch(0, 3, &right, true), Some(2));
        assert_eq!(next_swatch(0, 3, &left, true), Some(1));
        assert_eq!(
            next_swatch(2, 3, &Key::Named(NamedKey::ArrowDown), true),
            Some(0)
        );
        assert_eq!(
            next_swatch(1, 3, &Key::Named(NamedKey::End), false),
            Some(2)
        );
        assert_eq!(
            next_swatch(1, 3, &Key::Named(NamedKey::Home), false),
            Some(0)
        );
        assert_eq!(next_swatch(0, 3, &Key::Named(NamedKey::Enter), false), None);
        assert_eq!(next_swatch(0, 0, &right, false), None);
    }

    /// The custom swatch is described as opening a picker (todo 5096).
    #[test]
    fn the_custom_swatch_says_it_opens_a_picker() {
        fn app() -> Element {
            rsx! {
                libero::context::LiberoProvider {
                    ColorControl {
                        control: Control::color("color"),
                        label: "Color",
                        value: "primary",
                        onchange: |_| {},
                    }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let described = html
            .split("<button")
            .find(|button| button.contains("Custom color"))
            .and_then(|button| button.split("aria-describedby=\"").nth(1))
            .and_then(|rest| rest.split('"').next())
            .expect(&html);
        assert!(
            html.contains(&format!("id=\"{described}\"")) && html.contains("Opens a color picker"),
            "{html}"
        );
    }

    /// Text's unset swatch follows its gradient switch (todo 5035).
    #[test]
    fn the_unset_swatch_follows_another_control() {
        let color = Control::color("color")
            .with_unset()
            .unset_swatch("ink")
            .unset_swatch_when(|values| values.str("gradient") == "true", "primary.6");
        for (on, fill, tick) in [
            ("false", "ink", "surface"),
            ("true", "primary.6", "primary-contrast.6"),
        ] {
            let values =
                DemoValues::defaults(&[color.clone(), Control::switch("gradient").default(on)]);
            let shown = color.resolved(&values);
            assert_eq!(swatch(&shown, UNSET), (fill.to_string(), tick.to_string()));
        }
    }
}
