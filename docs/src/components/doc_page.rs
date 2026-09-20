use dioxus::prelude::*;
use libero::{
    components::{Anchor, Chip, Flex, Icon, OptionLabel, Options, Tabs, Title},
    sx::{Sx, sx},
    theme::PAPER_BORDER_COLOR,
};

use super::{A11yDoc, A11yPanel, PropGroup, PropertyTable};
use crate::{
    Route,
    icons::{AccessibilityIcon, CodeIcon, FileIcon, GitHubIcon, MarkdownIcon},
    nav::neighbours,
};

const REPO: &str = "https://github.com/tdymel/libero/tree/main/";

/// The tabs a docs page can show. `Usage` is the page's own sections.
#[derive(Clone, PartialEq, Options)]
enum DocTab {
    Usage,
    Properties,
    Accessibility,
}

/// A docs page: its heading, a lead paragraph, and its `DocSection`s.
///
/// `lead` is an `Element` rather than a `String` because most leads embed
/// `Code` spans in their prose. With `properties` or `accessibility` set, the
/// sections move into a "Usage" tab beside those tabs.
#[component]
pub fn DocPage(
    title: String,
    lead: Element,
    /// Repo-relative path to the component's source, linked beside the title.
    #[props(default)]
    source: Option<String>,
    /// URL of this page's markdown mirror under `public/md`: the web page plus
    /// complete, compiling examples, at a stable, guessable path.
    #[props(default)]
    markdown: Option<String>,
    #[props(default)] properties: Vec<PropGroup>,
    /// Fills an "Accessibility" tab; `None` shows no such tab.
    #[props(default)]
    accessibility: Option<A11yDoc>,
    children: Element,
) -> Element {
    let mut tab = use_signal(|| DocTab::Usage);
    let tabs: Vec<DocTab> = DocTab::options()
        .iter()
        .filter(|tab| match tab {
            DocTab::Usage => true,
            DocTab::Properties => !properties.is_empty(),
            DocTab::Accessibility => accessibility.is_some(),
        })
        .cloned()
        .collect();
    // Three equal tabs with icons break "Accessibility" mid-word on a phone:
    // drop the icons there and size each tab to its label.
    let crowded = tabs.len() > 2;
    let narrow = "(max-width: 30rem)";
    let icon_sx = match crowded {
        true => sx().media(narrow, sx().display("none")),
        false => sx(),
    };
    let tabs_sx = match crowded {
        true => sx().media(
            narrow,
            sx().selector(
                // Child combinators, or the Tabs page's own demo strip matches too.
                "& > [role=tablist] > [role=tab]",
                sx().flex("1 1 auto"),
            ),
        ),
        false => sx(),
    };

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
                    // Focused after a navigation (`AppShell`), without a ring round the heading:
                    // `Title` draws its keyboard ring as a `box-shadow`.
                    Title {
                        size: "xxl",
                        tabindex: "-1",
                        sx: sx().selector("&:focus", sx().outline("none").box_shadow("none")),
                        "{title}"
                    }
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
                                icon: rsx! { Icon { variant: "standard", size: "sm", color: "inherit", GitHubIcon {} } },
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
                                icon: rsx! { Icon { variant: "standard", size: "sm", color: "inherit", MarkdownIcon {} } },
                                "View as markdown"
                            }
                        }
                    }
                }
                {lead}
            }
            if tabs.len() == 1 {
                {children}
            } else {
                Tabs {
                    options: tabs,
                    value: tab(),
                    onchange: move |next| tab.set(next),
                    size: "lg",
                    full_width: true,
                    sx: tabs_sx,
                    option_label: move |selected: DocTab| OptionLabel::rich(
                        selected.label(),
                        rsx! {
                            Icon { variant: "standard", size: "md",
                                sx: icon_sx.clone(),
                                match selected {
                                    DocTab::Usage => rsx! { FileIcon {} },
                                    DocTab::Properties => rsx! { CodeIcon {} },
                                    DocTab::Accessibility => rsx! { AccessibilityIcon {} },
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
                        DocTab::Accessibility => rsx! {
                            A11yPanel { doc: accessibility.clone().unwrap_or_default() }
                        },
                    },
                }
            }
            Pager {}
        }
    }
}

fn pager_link_sx(end: bool) -> Sx {
    sx().flex("1")
        .min_width("0")
        .display("flex")
        .flex_direction("column")
        .gap("xs")
        .padding("md")
        .border(format!("1px solid {}", PAPER_BORDER_COLOR.value()))
        .border_radius("md")
        .text_align(if end { "end" } else { "start" })
        .overflow_wrap("anywhere")
        .hover(sx().border_color("primary"))
        .selector(
            "& > span:first-child",
            sx().font_size("0.875em").color("text-dimmed"),
        )
        .selector("& > span:last-child", sx().font_weight("600"))
}

/// Previous and next page in sidebar order, at the foot of every page.
#[component]
fn Pager() -> Element {
    let [previous, next] = neighbours(&use_route::<Route>());
    if previous.is_none() && next.is_none() {
        return rsx! {};
    }

    rsx! {
        nav { "aria-label": "Previous and next page",
            Flex { direction: "row", gap: "md",
                if let Some((route, label)) = previous {
                    Anchor { to: route, underline: "never", sx: pager_link_sx(false),
                        span { "Previous" }
                        span { "{label}" }
                    }
                } else {
                    div { flex: "1" }
                }
                if let Some((route, label)) = next {
                    Anchor { to: route, underline: "never", sx: pager_link_sx(true),
                        span { "Next" }
                        span { "{label}" }
                    }
                } else {
                    div { flex: "1" }
                }
            }
        }
    }
}
