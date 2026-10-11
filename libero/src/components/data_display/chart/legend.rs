use dioxus::prelude::*;

use super::ChartPart;
use crate::{
    components::common::{Part, focus_ring_sx},
    sx::{Sx, sx},
    tokens::NamedColorCss,
    utils::warn,
};

/// The toggles' rules, under the legend.
pub(super) fn legend_sx() -> Sx {
    sx().selector(
        "& button",
        sx().display("inline-flex")
            .align_items("center")
            .gap("xs")
            // The target size floor for a pointer.
            .min_height("24px")
            .padding("2px 4px")
            .margin("0")
            .border("0")
            .border_radius("sm")
            .background_color("transparent")
            .color("inherit")
            .with("font", "inherit")
            .cursor("pointer")
            .focus_visible(focus_ring_sx()),
    )
    // Struck through, not only dimmed: the state reads without colour.
    .selector(
        "& button[aria-pressed='false']",
        sx().color(NamedColorCss::TEXT_DIMMED.value())
            .with("text-decoration", "line-through"),
    )
    .selector("& button[aria-disabled='true']", sx().cursor("default"))
}

/// The series the legend hides, by name: `hidden` when set, else its own, seeded by
/// `default_hidden`. A toggle asks `onhiddenchange` either way.
#[derive(Clone)]
pub(super) struct HiddenSeries {
    own: Signal<Vec<String>>,
    controlled: Option<Vec<String>>,
    onchange: Option<EventHandler<Vec<String>>>,
}

impl HiddenSeries {
    pub(super) fn get(&self) -> Vec<String> {
        self.controlled
            .clone()
            .unwrap_or_else(|| self.own.read().clone())
    }

    /// Shows or hides `name`; the last shown series stays.
    pub(super) fn toggle(&self, name: &str, names: &[String]) {
        let mut next = self.get();
        match next.iter().position(|hidden| hidden == name) {
            Some(at) => {
                next.remove(at);
            }
            None if shown(names, &next) > 1 => next.push(name.to_string()),
            None => return,
        }
        if self.controlled.is_none() {
            let mut own = self.own;
            own.set(next.clone());
        }
        if let Some(onchange) = self.onchange {
            onchange.call(next);
        }
    }
}

pub(super) fn shown(names: &[String], hidden: &[String]) -> usize {
    names.iter().filter(|name| !hidden.contains(name)).count()
}

pub(super) fn use_hidden_series(
    hidden: &Option<Vec<String>>,
    default_hidden: &[String],
    onchange: Option<EventHandler<Vec<String>>>,
    names: &[String],
) -> HiddenSeries {
    let own = use_signal(|| default_hidden.to_vec());
    let names = names.to_vec();
    use_effect(use_reactive!(|names| {
        let mut seen = std::collections::HashSet::new();
        if let Some(twice) = names.iter().find(|name| !seen.insert(*name)) {
            warn(&format!(
                "chart series `{twice}` is named twice: the legend hides series by name"
            ));
        }
    }));
    HiddenSeries {
        own,
        controlled: hidden.clone(),
        onchange,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct LegendEntry {
    pub(super) name: String,
    pub(super) color: String,
    pub(super) hidden: bool,
}

/// The series as toggle buttons, pressed while shown. The last shown one is
/// `aria-disabled`, so the chart never goes blank.
#[component]
pub(super) fn ChartLegend(
    entries: Vec<LegendEntry>,
    label: String,
    ontoggle: EventHandler<String>,
) -> Element {
    let last = entries.iter().filter(|entry| !entry.hidden).count() <= 1;
    rsx! {
        ul { "data-slot": ChartPart::Legend.slot(), "aria-label": "{label}",
            for entry in entries {
                li { key: "{entry.name}", "data-slot": ChartPart::LegendItem.slot(),
                    button {
                        r#type: "button",
                        "aria-pressed": if entry.hidden { "false" } else { "true" },
                        "aria-disabled": (last && !entry.hidden).then_some("true"),
                        onclick: {
                            let name = entry.name.clone();
                            move |_| ontoggle.call(name.clone())
                        },
                        span {
                            "data-chart-swatch": true,
                            "data-hidden": entry.hidden.then_some("true"),
                            style: "--lsx-chart-swatch:{entry.color}",
                        }
                        "{entry.name}"
                    }
                }
            }
        }
    }
}
