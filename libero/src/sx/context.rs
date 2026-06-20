use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{LiberoContext, Sx};

use super::{DynamicDeclaration, StaticDeclaration};

#[derive(Clone, PartialEq, Eq)]
pub struct SxSnapshot {
    declarations: Vec<StaticDeclaration>,
    dynamic_declarations: Option<Vec<DynamicDeclaration>>,
}

impl<const N: usize> From<&Sx<N>> for SxSnapshot {
    fn from(sx: &Sx<N>) -> Self {
        Self {
            declarations: sx.declarations.iter().cloned().collect(),
            dynamic_declarations: sx.dynamic_declarations.clone(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SxContext {
    registry: Signal<HashMap<String, SxSnapshot>>,
}

impl Default for SxContext {
    fn default() -> Self {
        Self {
            registry: Signal::new(HashMap::new()),
        }
    }
}

impl SxContext {
    pub fn upsert(&self, id: String, sx: SxSnapshot) {
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

impl std::fmt::Display for SxSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for declaration in &self.declarations {
            write!(f, "{} ", declaration)?;
        }

        if let Some(dynamic_declarations) = &self.dynamic_declarations {
            for declaration in dynamic_declarations {
                write!(f, "{} ", declaration)?;
            }
        }

        Ok(())
    }
}

fn class_name_from_id(id: &str) -> String {
    format!("libero-sx-{}", id)
}

pub fn use_sx<const N: usize>(sx: Sx<N>) -> String {
    let context = use_context::<LiberoContext>();
    let id = use_hook(|| Uuid::new_v4().simple().to_string()[..8].to_string());
    let snapshot = use_memo(move || SxSnapshot::from(&sx));

    let id_value = id.clone();
    use_effect(move || {
        context.sx.upsert(id_value.clone(), snapshot.read().clone());
    });

    class_name_from_id(id.as_str())
}
