use dioxus::prelude::*;
use libero::components::Pictogram;
use libero::{
    components::{Box, Button, ColorCode, ColorPicker, ColorSwatch, Flex, SliderChangeEvent},
    hooks::use_element,
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
        .hover(sx().background("transparent"))
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
                role: "group",
                aria_label: "{label}",
                // Toggle `Button`s, not a `SegmentedControl`: swatches read as separate chips.
                for (option, fill_color, tick_color) in swatches.iter() {
                    Button {
                        key: "{option}",
                        size: "sm",
                        variant: "text",
                        aria_label: "{option}",
                        selected: value == *option,
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
                        Button {
                            size: "sm",
                            variant: "text",
                            aria_label: "{custom_label}",
                            aria_expanded: open(),
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
                        if event.key() == Key::Escape {
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
