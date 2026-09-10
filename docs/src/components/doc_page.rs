use dioxus::prelude::*;
use libero::components::{Chip, Flex, Icon, OptionLabel, Options, Tabs, Title};

use super::{PropGroup, PropertyTable};
use crate::icons::{CodeIcon, FileIcon, GitHubIcon, MarkdownIcon};

const REPO: &str = "https://github.com/tdymel/libero/tree/main/";

/// The tabs a docs page can show. `Usage` is the page's own sections.
#[derive(Clone, PartialEq, Options)]
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
    /// URL of this page's markdown rendering - a verbatim copy under
    /// `public/md`, so the link stays a stable, guessable path.
    #[props(default)]
    markdown: Option<String>,
    #[props(default)] properties: Vec<PropGroup>,
    children: Element,
) -> Element {
    let mut tab = use_signal(|| DocTab::Usage);

    rsx! {
        // Every route is a `DocPage`, so this names each one; `App`'s bare
        // "Libero" only shows before the first page renders.
        document::Title { "{title} - Libero" }
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Flex {
                    direction: "row",
                    align: "center",
                    gap: "lg",
                    wrap: "wrap",
                    Title { size: "xxl", "{title}" }
                    Flex {
                        direction: "row",
                        align: "center",
                        gap: "sm",
                        wrap: "wrap",
                        if let Some(source) = source {
                            Chip {
                                to: format!("{REPO}{source}"),
                                target: "_blank",
                                size: "sm",
                                // Page furniture, not an accent: the filled
                                // default would outrank the page's own title.
                                variant: "outlined",
                                color: "neutral",
                                Icon { variant: "standard", size: "sm", color: "inherit", GitHubIcon {} }
                                "Source"
                            }
                        }
                        if let Some(markdown) = markdown {
                            Chip {
                                // `External`, or the router parses the path as
                                // a route and fails - it is a static file.
                                //
                                // Root-relative only works where something
                                // serves `public/`. A Blitz window resolves
                                // it against `dioxus://index.html` and drops
                                // it for having the wrong scheme, so point at
                                // the repo copy there instead - that opens a
                                // browser.
                                //
                                // Gated on the renderer, not the target: the
                                // fullstack server and the Android WebView are
                                // both "not wasm" and both serve `public/`.
                                to: NavigationTarget::External(
                                    if cfg!(any(feature = "native", feature = "native-cpu")) {
                                        format!("{REPO}docs/public{markdown}")
                                    } else {
                                        markdown.clone()
                                    },
                                ),
                                target: "_blank",
                                size: "sm",
                                variant: "outlined",
                                color: "neutral",
                                Icon { variant: "standard", size: "sm", color: "inherit", MarkdownIcon {} }
                                "View as markdown"
                            }
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
                    option_label: |selected: DocTab| OptionLabel::rich(
                        selected.label(),
                        rsx! {
                            Icon { variant: "standard", size: "md",
                                match selected {
                                    DocTab::Usage => rsx! { FileIcon {} },
                                    DocTab::Properties => rsx! { CodeIcon {} },
                                }
                            }
                            "{selected.label()}"
                        },
                    ),
                    panel: move |selected| match selected {
                        // The sections carry no spacing of their own - outside
                        // the tabs the page's own column `Flex` gaps them.
                        DocTab::Usage => rsx! {
                            Flex { direction: "column", gap: "xxl", {children.clone()} }
                        },
                        DocTab::Properties => rsx! {
                            PropertyTable { properties: properties.clone() }
                        },
                    },
                }
            }
        }
    }
}
