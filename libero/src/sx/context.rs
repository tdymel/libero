use dioxus::prelude::*;
use std::collections::HashMap;

use crate::{LiberoContext, Sx};

#[derive(Clone, Copy)]
pub struct SxContext {
    registry: Signal<HashMap<String, String>>,
    next_id: Signal<usize>,
}

impl Default for SxContext {
    fn default() -> Self {
        Self {
            registry: Signal::new(HashMap::new()),
            next_id: Signal::new(0),
        }
    }
}

impl SxContext {
    pub fn register<const N: usize>(&self, sx: Sx<N>) -> String {
        let css_body = sx.to_string();
        let mut registry = self.registry;

        if let Some(existing_class_name) = registry.read().get(&css_body) {
            return existing_class_name.clone();
        }

        let mut next_id = self.next_id;
        let class_name = format!("libero-sx-{}", *next_id.read());
        next_id += 1;

        registry.write().insert(css_body, class_name.clone());

        class_name
    }

    pub fn stylesheet(&self) -> String {
        let registry = self.registry;
        registry
            .read()
            .iter()
            .map(|(css_body, class_name)| format!(".{} {{ {} }}", class_name, css_body))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn use_sx<const N: usize>(sx: Sx<N>) -> String {
    let context = use_context::<LiberoContext>();
    use_hook(move || context.sx.register(sx))
}
