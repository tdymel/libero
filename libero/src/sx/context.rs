use dioxus::prelude::*;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::LiberoContext;

use super::{NestedRule, SxDyn, optimize_styles};

#[derive(Clone, Copy)]
pub struct SxContext {
    registry: Signal<IndexMap<String, SxDyn>>,
    stylesheet: Memo<String>,
}

impl Default for SxContext {
    fn default() -> Self {
        let registry = Signal::new(IndexMap::new());
        let stylesheet = Memo::new(move || {
            optimize_styles(&registry.read())
                .into_iter()
                .flat_map(|rule| render_rule(&rule.selector, &rule.sx))
                .collect::<Vec<_>>()
                .join("\n")
        });

        Self {
            registry,
            stylesheet,
        }
    }
}

pub(crate) fn class_name_from_id(id: &str) -> String {
    format!("lsx-{}", id)
}

fn render_rule(selector: &str, sx: &SxDyn) -> Vec<String> {
    render_rule_with_media(selector, sx, &[])
}

fn render_rule_with_media(selector: &str, sx: &SxDyn, media_stack: &[&'static str]) -> Vec<String> {
    let mut rules = Vec::new();

    if !sx.declarations.is_empty()
        || sx
            .dynamic_declarations
            .as_ref()
            .is_some_and(|dynamic| !dynamic.is_empty())
    {
        let mut rule = format!("{} {{ {} }}", selector, sx);

        for query in media_stack.iter().rev() {
            rule = format!("@media {} {{ {} }}", query, rule);
        }

        rules.push(rule);
    }

    for nested_rule in &sx.nested_rules {
        match nested_rule {
            NestedRule::Selector { fragment, sx } => {
                rules.extend(render_rule_with_media(
                    &fragment.replace('&', selector),
                    sx,
                    media_stack,
                ));
            }
            NestedRule::Media { query, sx } => {
                let mut nested_media_stack = media_stack.to_vec();
                nested_media_stack.push(query);
                rules.extend(render_rule_with_media(selector, sx, &nested_media_stack));
            }
        }
    }

    rules
}

impl SxContext {
    pub fn upsert(&self, id: String, sx: SxDyn) {
        let mut registry = self.registry;
        let should_update = registry.read().get(&id) != Some(&sx);

        if should_update {
            registry.write().insert(id, sx);
        }
    }

    pub fn stylesheet(&self) -> String {
        self.stylesheet.read().clone()
    }
}

pub fn use_sx<S>(sx: S) -> String
where
    S: Into<SxDyn> + Clone + PartialEq + 'static,
{
    let context = use_context::<LiberoContext>();
    let id = use_hook(|| Uuid::new_v4().simple().to_string()[..8].to_string());
    let snapshot = use_memo(move || sx.clone().into());

    let id_value = id.clone();
    use_effect(move || {
        context.sx.upsert(id_value.clone(), snapshot.read().clone());
    });

    class_name_from_id(id.as_str())
}
