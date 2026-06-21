use dioxus::prelude::*;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::LiberoContext;

use super::{SxDyn, optimize_styles};

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
                .map(|rule| format!("{} {{ {} }}", rule.selector, rule.sx))
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
