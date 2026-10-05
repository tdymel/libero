use dioxus::prelude::*;
use libero::components::SpotlightAction;

use super::{aliases::aliases, tree::pages};
use crate::Route;

/// One palette action per page, grouped under its sidebar section - what the
/// docs search opens on until a real search index exists. `section` lands a
/// section action on its `DocSection` or `ExtraTab`.
pub fn page_actions(section: Signal<Option<String>>) -> Vec<SpotlightAction> {
    let sections = SECTIONS
        .iter()
        .map(move |&(label, route, id, group, keywords)| {
            let mut action = SpotlightAction::new(label).onclick(move |_| {
                let mut section = section;
                section.set(Some(id.to_string()));
                navigator().push(route());
            });
            action.keywords = keywords.iter().map(|k| k.to_string()).collect();
            action.group = Some(group.to_string());
            action
        });
    pages()
        .into_iter()
        .map(|(path, label, group)| {
            let mut action = SpotlightAction::new(label).onclick(move |_| {
                if let Ok(route) = path.parse::<Route>() {
                    navigator().push(route);
                }
            });
            // The section is a keyword too: "Accessibility" finds its "Overview".
            action.keywords = group
                .into_iter()
                .chain(aliases(group, label).iter().copied())
                .map(str::to_string)
                .collect();
            action.group = group.map(str::to_string);
            action
        })
        .chain(sections)
        .collect()
}

/// Search actions for a page's section: label, page, section id, group, keywords.
type SectionAction = (
    &'static str,
    fn() -> Route,
    &'static str,
    &'static str,
    &'static [&'static str],
);

const SECTIONS: &[SectionAction] = &[(
    "Icon catalogue",
    || Route::PictogramPage {},
    "icons",
    "Data display",
    &["icons", "icon search", "lucide", "browse icons"],
)];

#[cfg(test)]
mod tests {
    use libero::components::spotlight_filter;

    use super::*;

    #[test]
    fn a_section_name_finds_its_overview() {
        // `onclick` makes a `Callback`, which needs a runtime.
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        let hits = spotlight_filter("Accessibility", &actions);
        let hit = |label: &str| {
            hits.iter().any(|action| {
                action.label == label && action.group.as_deref() == Some("Accessibility")
            })
        };
        assert!(hit("Overview") && hit("FocusTrap") && hit("Focus return"));
    }

    #[test]
    fn the_icon_catalogue_lands_on_its_tab() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        let hits = spotlight_filter("browse icons", &actions);
        assert_eq!(hits[0].label, "Icon catalogue");
    }

    #[test]
    fn icon_and_icons_list_the_icon_providers() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        for query in ["Icon", "Icons"] {
            let hits = spotlight_filter(query, &actions);
            for label in ["Icon", "Pictogram", "IconProvider"] {
                assert!(
                    hits.iter().any(|hit| hit.label == label),
                    "{query}: {label}"
                );
            }
        }
    }

    #[test]
    fn a_shared_label_takes_its_sections_words() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        let hits = spotlight_filter("a11y", &actions);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].group.as_deref(), Some("Accessibility"));
    }

    #[test]
    fn an_alias_finds_its_page_after_label_matches() {
        let dom = VirtualDom::new(VNode::empty);
        let actions = dom.in_scope(ScopeId::ROOT, || page_actions(Signal::new(None)));
        let labels = |query| {
            spotlight_filter(query, &actions)
                .into_iter()
                .map(|action| action.label)
                .collect::<Vec<_>>()
        };
        assert!(labels("datepicker").contains(&"ChronoField".to_string()));
        assert_eq!(labels("select")[0], "Select");
        let popup = labels("popup");
        assert!(popup.contains(&"Modal".to_string()) && popup.contains(&"Popover".to_string()));
        // "Menu" lists "Menu" and "Menubar" (labels) before "Burger" (alias "menu button").
        let menu = labels("menu");
        let at = |label: &str| menu.iter().position(|found| found == label).unwrap();
        assert!(at("Menubar") < at("Burger"));
    }
}
