//! The label against the fill it sits on in every interactive state, measured
//! on the resolved `:root` rather than on the vars' names (todo 452).

use std::collections::BTreeMap;

use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Button},
    theme::Theme,
};

const PALETTE: [&str; 8] = [
    "primary",
    "secondary",
    "error",
    "warning",
    "info",
    "success",
    "neutral",
    "muted",
];
const VARIANTS: [&str; 5] = ["filled", "tonal", "elevated", "outlined", "standard"];

#[component]
fn Matrix() -> Element {
    rsx! {
        for color in PALETTE {
            for variant in VARIANTS {
                Button { color, variant, selected: false, "{color}" }
                ActionIcon { color, variant, aria_label: "{color}", selected: false, "x" }
            }
        }
    }
}

fn light() -> Element {
    // The default pair: the light theme is the `:root` block read below.
    rsx! { LiberoProvider { Matrix {} } }
}

fn dark() -> Element {
    rsx! { LiberoProvider { themes: &Theme::DARK, Matrix {} } }
}

/// `name:value;` pairs, as `:root` and a `style` attribute both write them.
fn declarations(block: &str) -> BTreeMap<String, String> {
    block
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        .collect()
}

/// Follows `var(--x, fallback)` through the element's own vars, then `:root`.
fn resolve(value: &str, vars: &[&BTreeMap<String, String>]) -> String {
    let Some(inner) = value.strip_prefix("var(").and_then(|v| v.strip_suffix(')')) else {
        return value.to_string();
    };
    let (name, fallback) = match inner.split_once(',') {
        Some((name, fallback)) => (name.trim(), Some(fallback.trim())),
        None => (inner.trim(), None),
    };
    match vars.iter().find_map(|vars| vars.get(name)) {
        Some(value) => resolve(value, vars),
        None => resolve(fallback.unwrap_or_else(|| panic!("{name} is unset")), vars),
    }
}

fn luminance(hex: &str) -> f32 {
    let hex = hex
        .strip_prefix('#')
        .unwrap_or_else(|| panic!("{hex} is no hex"));
    let channel = |i: usize| {
        let value = u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f32 / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4)
}

/// `fill` with `PRESSED_LAYER` percent of `label` mixed in, as `color-mix(in srgb, ..)` does.
fn layered(fill: &str, label: &str) -> String {
    let channel = |hex: &str, i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f32;
    let mixed: String = [1, 3, 5]
        .map(|i| channel(fill, i) * 0.9 + channel(label, i) * 0.1)
        .map(|value| format!("{:02x}", value.round() as u8))
        .concat();
    format!("#{mixed}")
}

fn ratio(a: &str, b: &str) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// Every hover, selected and pressed pair below 4.5:1, as `"<tag> <color> <variant>
/// <state> <ratio>"`.
fn short_pairs(app: fn() -> Element) -> Vec<String> {
    let html = render(app);
    let root = html
        .split_once(":root{")
        .and_then(|(_, rest)| rest.split_once('}'))
        .map(|(block, _)| declarations(block))
        .expect("a :root block");

    let mut short = Vec::new();
    for (tag, prefix) in [
        ("Button", "--lsx-button"),
        ("ActionIcon", "--lsx-action-icon"),
    ] {
        let mut rest = html.as_str();
        for color in PALETTE {
            for variant in VARIANTS {
                let marker = format!("{prefix}-color:");
                let at = rest.find(&marker).expect("one element per pair");
                let start = rest[..at].rfind("<button").expect("a button");
                let own = declarations(&attributes_of(&rest[start..], "button")["style"]);
                rest = &rest[at + marker.len()..];

                for state in ["hover", "selected", "pressed"] {
                    // Any other pressed fill is the selected one, measured above.
                    if state == "pressed" && variant != "tonal" {
                        continue;
                    }
                    // The var the CSS labels this state with.
                    let label = match (variant, state) {
                        ("tonal", "hover" | "pressed") => "on-container",
                        _ => "on-state",
                    };
                    let label = resolve(&own[&format!("{prefix}-{label}")], &[&own, &root]);
                    // A tonal pressed fill is the hover's with a layer of the label.
                    let fill = match state {
                        "pressed" => {
                            let hover = resolve(&own[&format!("{prefix}-hover")], &[&own, &root]);
                            layered(&hover, &label)
                        }
                        _ => resolve(&own[&format!("{prefix}-{state}")], &[&own, &root]),
                    };
                    let measured = ratio(&label, &fill);
                    if measured < 4.5 {
                        short.push(format!("{tag} {color} {variant} {state} {measured:.2}"));
                    }
                }
            }
        }
    }
    short
}

#[test]
fn every_hover_selected_and_pressed_label_reads_on_its_fill_in_the_light_theme() {
    assert_eq!(short_pairs(light), Vec::<String>::new());
}

/// Recorded, not asserted away: a dark theme's `fill-3` of yellow and green is
/// a mid tone that neither end of the page reads on, so no label can fix it.
#[test]
fn only_the_tonal_selected_yellow_and_green_fall_short_in_the_dark_theme() {
    assert_eq!(
        short_pairs(dark),
        [
            "Button warning tonal selected 3.99",
            "Button success tonal selected 4.34",
            "ActionIcon warning tonal selected 3.99",
            "ActionIcon success tonal selected 4.34",
        ]
    );
}
