use dioxus::prelude::*;

/// One thing a [`use_spotlight`](super::use_spotlight) palette can run.
#[derive(Clone, Default)]
pub struct SpotlightAction {
    pub label: String,
    pub description: Option<String>,
    /// Matched by [`spotlight_filter`], never drawn.
    pub keywords: Vec<String>,
    /// A section header. Groups are ordered by their first appearance.
    pub group: Option<String>,
    pub icon: Option<Element>,
    /// A hint, one `Kbd` per key split on spaces and `+` (spell a plus key
    /// "Plus"). Never bound.
    pub shortcut: Option<String>,
    pub onclick: Option<Callback<()>>,
}

impl SpotlightAction {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn keywords<K: Into<String>>(mut self, keywords: impl IntoIterator<Item = K>) -> Self {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn icon(mut self, icon: Element) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn onclick(mut self, onclick: impl FnMut(()) + 'static) -> Self {
        self.onclick = Some(Callback::new(onclick));
        self
    }
}

/// The default filter: a case-insensitive substring match on label, description
/// and keywords, label hits first. An empty query returns everything.
pub fn spotlight_filter(query: &str, actions: &[SpotlightAction]) -> Vec<SpotlightAction> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return actions.to_vec();
    }
    let hit = |text: &str| text.to_lowercase().contains(&query);

    let (by_label, rest): (Vec<_>, Vec<_>) = actions.iter().partition(|action| hit(&action.label));
    let by_other = rest.into_iter().filter(|action| {
        action.description.as_deref().is_some_and(hit)
            || action.keywords.iter().any(|keyword| hit(keyword))
    });
    by_label.into_iter().chain(by_other).cloned().collect()
}

/// Regrouped by first appearance of `group`, then cut to `limit` rows in total,
/// not per group.
pub(super) fn group_and_limit(
    actions: Vec<SpotlightAction>,
    limit: Option<usize>,
) -> Vec<(Option<String>, Vec<SpotlightAction>)> {
    let mut groups: Vec<(Option<String>, Vec<SpotlightAction>)> = Vec::new();
    for action in actions {
        match groups.iter_mut().find(|(group, _)| *group == action.group) {
            Some((_, members)) => members.push(action),
            None => groups.push((action.group.clone(), vec![action])),
        }
    }

    let mut budget = limit.unwrap_or(usize::MAX);
    groups
        .into_iter()
        .filter_map(|(group, mut members)| {
            members.truncate(budget);
            budget -= members.len();
            (!members.is_empty()).then_some((group, members))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(actions: &[SpotlightAction]) -> Vec<&str> {
        actions.iter().map(|action| action.label.as_str()).collect()
    }

    fn fixture() -> Vec<SpotlightAction> {
        vec![
            SpotlightAction::new("Home").description("The start page"),
            SpotlightAction::new("Settings").keywords(["preferences", "home screen"]),
            SpotlightAction::new("Homebrew"),
            SpotlightAction::new("Docs"),
        ]
    }

    #[test]
    fn an_empty_query_keeps_everything_in_order() {
        assert_eq!(
            labels(&spotlight_filter("  ", &fixture())),
            ["Home", "Settings", "Homebrew", "Docs"]
        );
    }

    #[test]
    fn label_hits_rank_above_description_and_keyword_hits() {
        assert_eq!(
            labels(&spotlight_filter("HOME", &fixture())),
            ["Home", "Homebrew", "Settings"]
        );
        assert_eq!(labels(&spotlight_filter("start", &fixture())), ["Home"]);
        assert!(spotlight_filter("nothing", &fixture()).is_empty());
    }

    #[test]
    fn groups_follow_first_appearance_and_the_limit_counts_through_them() {
        let actions = vec![
            SpotlightAction::new("a").group("B"),
            SpotlightAction::new("b").group("A"),
            SpotlightAction::new("c"),
            SpotlightAction::new("d").group("B"),
            SpotlightAction::new("e").group("A"),
        ];
        let grouped = group_and_limit(actions.clone(), None);
        let shape: Vec<(Option<&str>, Vec<&str>)> = grouped
            .iter()
            .map(|(group, members)| (group.as_deref(), labels(members)))
            .collect();
        assert_eq!(
            shape,
            [
                (Some("B"), vec!["a", "d"]),
                (Some("A"), vec!["b", "e"]),
                (None, vec!["c"]),
            ]
        );

        let limited = group_and_limit(actions, Some(3));
        let shape: Vec<(Option<&str>, Vec<&str>)> = limited
            .iter()
            .map(|(group, members)| (group.as_deref(), labels(members)))
            .collect();
        assert_eq!(shape, [(Some("B"), vec!["a", "d"]), (Some("A"), vec!["b"])]);
    }
}
