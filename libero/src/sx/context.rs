use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::LiberoContext;

use super::{Sx, SxDyn};

#[derive(Clone, Copy)]
pub struct SxContext {
    registry: Signal<HashMap<String, SxDyn>>,
}

impl Default for SxContext {
    fn default() -> Self {
        Self {
            registry: Signal::new(HashMap::new()),
        }
    }
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
        let registry = self.registry;
        registry
            .read()
            .iter()
            .map(|(id, sx)| format!(".{} {{ {} }}", class_name_from_id(id), sx))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn class_name_from_id(id: &str) -> String {
    format!("libero-sx-{}", id)
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
