use dioxus::prelude::*;
use libero::components::{Chip, Flex, Icon, TabLabel, TabValue, Tabs, Title};

use super::{PropGroup, PropertyTable};
use crate::icons::{CodeIcon, FileIcon, GitHubIcon};

const REPO: &str = "https://github.com/tdymel/libero/tree/main/";

/// The tabs a docs page can show. `Usage` is the page's own sections.
#[derive(Clone, PartialEq, TabValue)]
enum DocTab {
    Usage,
    Properties,
}

/// A docs page: its heading, a lead paragraph, and its `DocSection`s.
///
/// `lead` is an `Element` rather than a `String` because most leads embed
/// `Code` spans in their prose. With `properties` set, the sections move into
/// a "Usage" tab beside the generated property table.
#[component]
pub fn DocPage(
    title: String,
    lead: Element,
    /// Repo-relative path to the component's source, linked beside the title.
    #[props(default)]
    source: Option<String>,
    #[props(default)] properties: Vec<PropGroup>,
    children: Element,
) -> Element {
    let mut tab = use_signal(|| DocTab::Usage);

    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Flex {
                    direction: "row",
                    align: "center",
                    gap: "md",
                    wrap: "wrap",
                    Title { size: "xxl", "{title}" }
                    if let Some(source) = source {
                        Chip {
                            to: format!("{REPO}{source}"),
                            target: "_blank",
                            size: "sm",
                            Icon { variant: "transparent", size: "sm", color: "inherit", GitHubIcon {} }
                            "Source"
                        }
                    }
                }
                {lead}
            }
            if properties.is_empty() {
                {children}
            } else {
                Tabs {
                    value: tab(),
                    onchange: move |next| tab.set(next),
                    size: "lg",
                    full_width: true,
                    label: |selected: DocTab| TabLabel::rich(
                        selected.label(),
                        rsx! {
                            Icon { variant: "transparent", size: "md",
                                match selected {
                                    DocTab::Usage => rsx! { FileIcon {} },
                                    DocTab::Properties => rsx! { CodeIcon {} },
                                }
                            }
                            "{selected.label()}"
                        },
                    ),
                    panel: move |selected| match selected {
                        DocTab::Usage => rsx! { {children.clone()} },
                        DocTab::Properties => rsx! {
                            PropertyTable { properties: properties.clone() }
                        },
                    },
                }
            }
        }
    }
}
