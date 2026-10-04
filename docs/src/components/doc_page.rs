use dioxus::prelude::*;
use libero::{
    components::{Anchor, Chip, Flex, Icon, OptionLabel, Options, Pictogram, Tabs, Title, Tldr},
    sx::{Sx, sx},
    theme::{HEADER_HEIGHT_VAR, PAPER_BORDER_COLOR},
};
use pictogram_icons_lucide as lucide;

use super::{A11yDoc, A11yPanel, PartsPanel, PropGroup, PropertyTable, doc_section::prose_sx};
use crate::{
    Route, heading_focus,
    nav::neighbours,
    site::{SITE, TLDR_PROMPT, github_tree},
};
use pictogram_icons_simple as simple;

/// The tabs a docs page can show. `Usage` is the page's own sections.
#[derive(Clone, PartialEq, Options)]
enum DocTab {
    Usage,
    /// The page's own `extra_tab`, labelled by it.
    Extra,
    Properties,
    #[option(label = "Style API")]
    StyleApi,
    Accessibility,
}

/// A page's own tab after "Usage", such as the Pictogram page's icon catalogue.
#[derive(Clone, PartialEq)]
pub struct ExtraTab {
    pub label: &'static str,
    /// Its section's id: a `#id` URL or a `SectionLink` to it opens the tab.
    pub id: &'static str,
    pub content: Element,
}

/// A docs page: heading, lead paragraph, `DocSection`s. With `properties` or `accessibility`
/// set, the sections move into a "Usage" tab beside those tabs. A `PropGroup` with
/// `.parts(...)` adds a "Style API" tab.
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
    #[props(default)] extra_tab: Option<ExtraTab>,
    /// The TLDR chip; off for a page with nothing to summarize, such as the not-found page.
    #[props(default = true)]
    tldr: bool,
    children: Element,
) -> Element {
    #[cfg(test)]
    use_hook(|| {
        crate::snippets::record_page(crate::snippets::RecordedPage {
            title: title.clone(),
            markdown: markdown.clone(),
            properties: properties.clone(),
        })
    });
    // Assistants read the markdown mirror, else the route.
    let route = use_route::<Route>().to_string();
    let tldr_path = markdown.clone().unwrap_or(route);
    let pending = try_use_context::<heading_focus::PendingSection>().map(|p| p.0);
    let extra_id = extra_tab.as_ref().map(|extra| extra.id);
    let landing = move |id: Option<String>| id.is_some() && id.as_deref() == extra_id;
    let mut tab = use_signal(|| {
        let pending = pending.and_then(|p| p.peek().clone());
        match landing(heading_focus::load_fragment()) || landing(pending) {
            true => DocTab::Extra,
            false => DocTab::Usage,
        }
    });
    // A search action on this same page sets the section without a route change.
    use_effect(move || {
        if let Some(pending) = pending
            && landing(pending())
        {
            tab.set(DocTab::Extra);
        }
    });
    // No route change lands it either: once its panel is in the page, land here and clear it.
    // Not on the first run, where a navigation's own landing still needs the id.
    let shell = try_use_context::<heading_focus::SectionLanding>();
    let mut first = use_hook(|| CopyValue::new(true));
    let mut extra_mounted = use_signal(|| false);
    use_effect(move || {
        let (Some(mut pending), Some(shell)) = (pending, shell) else {
            return;
        };
        let (ready, id) = (extra_mounted(), pending());
        if first.replace(false) || !ready || !landing(id.clone()) {
            return;
        }
        pending.set(None);
        shell.land(id.as_deref().unwrap_or_default());
    });
    // The tabs shown so far: their panels stay mounted, hidden (todo 2150).
    let mut visited = use_hook(|| CopyValue::new(Vec::<DocTab>::new()));
    let current = tab();
    if !visited.peek().contains(&current) {
        visited.write().push(current);
    }
    let tabs: Vec<DocTab> = DocTab::options()
        .iter()
        .filter(|tab| match tab {
            DocTab::Usage => true,
            DocTab::Extra => extra_tab.is_some(),
            DocTab::Properties => !properties.is_empty(),
            DocTab::Accessibility => accessibility.is_some(),
            DocTab::StyleApi => properties.iter().any(PropGroup::has_parts),
        })
        .cloned()
        .collect();
    let extra_label = extra_tab.as_ref().map(|extra| extra.label);
    // Equal tabs that never shrink below icon and label: the tablist scrolls instead.
    // Child combinators, or the Tabs page's demo strip matches too.
    let tabs_sx = sx()
        .selector(
            "& > [role=tablist] > [role=tab]",
            sx().flex("1 0 0")
                .min_width("max-content")
                .white_space("nowrap"),
        )
        .media(
            "(max-width: 30rem)",
            sx().selector("& > [role=tablist] > [role=tab]", sx().padding_inline("sm")),
        );

    rsx! {
        // Every route is a `DocPage`, so this names each one; `App`'s bare
        // "Libero" only shows before the first page renders.
        document::Title { "{title} - Libero" }
        Flex {
            direction: "column",
            gap: "xxl",
            sx: prose_sx(),
            Flex {
                direction: "column",
                gap: "lg",
                sx: prose_sx(),
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
                                to: github_tree(&source),
                                target: "_blank",
                                size: "sm",
                                // Filled would outrank the page title.
                                variant: "outlined",
                                color: "neutral",
                                icon: rsx! { Icon { variant: "standard", size: "sm", color: "inherit", svg: simple::github::regular } },
                                "Source"
                            }
                        }
                        if let Some(markdown) = markdown.clone() {
                            Chip {
                                // `External`: a static file, not a route. Blitz drops a root-relative
                                // URL (no `public/` served), so native links the repo copy.
                                to: NavigationTarget::External(
                                    if cfg!(any(feature = "native", feature = "native-cpu")) {
                                        github_tree(&format!("docs/public{markdown}"))
                                    } else {
                                        markdown.clone()
                                    },
                                ),
                                target: "_blank",
                                size: "sm",
                                variant: "outlined",
                                color: "neutral",
                                icon: rsx! { Icon { variant: "standard", size: "sm", color: "inherit", svg: simple::markdown::regular } },
                                "View as markdown"
                            }
                        }
                        if tldr {
                            Tldr { url: format!("{SITE}{tldr_path}"), prompt: TLDR_PROMPT, size: "sm" }
                        }
                    }
                }
                {lead}
            }
            if tabs.len() == 1 {
                {children}
            } else {
                Tabs {
                    aria_label: "Documentation sections",
                    options: tabs,
                    value: tab(),
                    onchange: move |next| {
                        if next != DocTab::Extra {
                            extra_mounted.set(false);
                        }
                        tab.set(next);
                    },
                    size: "lg",
                    full_width: true,
                    sx: tabs_sx,
                    option_label: move |selected: DocTab| OptionLabel::rich(
                        match (&selected, extra_label) {
                            (DocTab::Extra, Some(label)) => label.to_string(),
                            _ => selected.label(),
                        },
                        rsx! {
                            Icon { variant: "standard", size: "md",
                                match selected {
                                    DocTab::Usage => rsx! { Pictogram { icon: lucide::file::outlined } },
                                    DocTab::Extra => rsx! { Pictogram { icon: lucide::shapes::outlined } },
                                    DocTab::Properties => rsx! { Pictogram { icon: lucide::code::outlined } },
                                    DocTab::Accessibility => rsx! { Pictogram { icon: lucide::accessibility::outlined } },
                                    DocTab::StyleApi => rsx! { Pictogram { icon: lucide::palette::outlined } },
                                }
                            }
                            match (&selected, extra_label) {
                                (DocTab::Extra, Some(label)) => label.to_string(),
                                _ => selected.label(),
                            }
                        },
                    ),
                    // One template for every tab, so a switch flips `hidden` instead of remounting
                    // a panel it showed before (todo 2150). `Extra` still remounts: its landing waits on `onmounted`.
                    panel: move |selected: DocTab| {
                        let off = |tab: DocTab| (selected != tab).then_some(true);
                        let kept = |tab: DocTab| visited.peek().contains(&tab);
                        rsx! {
                            div { hidden: off(DocTab::Usage),
                                if kept(DocTab::Usage) {
                                    // The sections carry no spacing of their own - outside
                                    // the tabs the page's own column `Flex` gaps them.
                                    Flex { direction: "column", gap: "xxl", sx: prose_sx(), {children.clone()} }
                                }
                            }
                            // No heading: the tab already names it (1358).
                            if let (DocTab::Extra, Some(extra)) = (&selected, extra_tab.clone()) {
                                Flex {
                                    id: extra.id,
                                    direction: "column",
                                    gap: "sm",
                                    // The landing target, by script only, with no ring round the panel.
                                    tabindex: "-1",
                                    // Room to scroll its top up even before late content (the
                                    // catalogue's fetch) fills it.
                                    sx: prose_sx()
                                        .min_height(format!("calc(100vh - {})", HEADER_HEIGHT_VAR.value()))
                                        .selector("&:focus", sx().outline("none").box_shadow("none")),
                                    onmounted: move |_| extra_mounted.set(true),
                                    {extra.content}
                                }
                            }
                            div { hidden: off(DocTab::Properties),
                                if kept(DocTab::Properties) {
                                    PropertyTable { properties: properties.clone() }
                                }
                            }
                            div { hidden: off(DocTab::StyleApi),
                                if kept(DocTab::StyleApi) {
                                    PartsPanel { properties: properties.clone() }
                                }
                            }
                            div { hidden: off(DocTab::Accessibility),
                                if kept(DocTab::Accessibility) {
                                    A11yPanel { doc: accessibility.clone().unwrap_or_default() }
                                }
                            }
                        }
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
