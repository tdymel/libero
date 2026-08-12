use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;

use crate::{SxLayer, css::Stylesheet, sx::Sx};

#[derive(Clone)]
pub struct RegisteredStylesheet {
    pub key: u64,
    pub class_name: String,
    pub css: Stylesheet,
    pub registrations: Rc<()>,
}

#[derive(Clone)]
pub struct RegistrationGuard {
    registrations: Rc<()>,
    sx_registry_version: Signal<u64>,
}

impl Drop for RegistrationGuard {
    fn drop(&mut self) {
        let _ = &self.registrations;
        *self.sx_registry_version.write() += 1;
    }
}

#[derive(Clone, Default)]
pub struct SxRegistry {
    inner: Rc<RefCell<BTreeMap<(SxLayer, u64), RegisteredStylesheet>>>,
}

impl SxRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn registration_token(&self, sx: &Sx, layer: SxLayer) -> Rc<()> {
        let key = sx.hash();
        let mut registry = self.inner.borrow_mut();

        if let Some(entry) = registry.get(&(layer, key)) {
            return entry.registrations.clone();
        }

        let registrations = Rc::new(());
        registry.insert(
            (layer, key),
            RegisteredStylesheet {
                key,
                class_name: sx.class_name(),
                css: Stylesheet::from(format!(
                    "@layer {}{{{}}}",
                    layer.css_name(),
                    Stylesheet::from(sx).as_str()
                )),
                registrations: registrations.clone(),
            },
        );

        registrations
    }

    pub fn register(
        &self,
        sx: &Sx,
        layer: SxLayer,
        sx_registry_version: Signal<u64>,
    ) -> RegistrationGuard {
        RegistrationGuard {
            registrations: self.registration_token(sx, layer),
            sx_registry_version,
        }
    }

    pub fn stylesheets(&self) -> Vec<String> {
        let mut registry = self.inner.borrow_mut();
        registry.retain(|_, entry| Rc::strong_count(&entry.registrations) > 1);
        registry
            .values()
            .map(|entry| {
                debug_assert_eq!(entry.class_name, format!("lsx-{:016x}", entry.key));
                entry.css.as_str().to_string()
            })
            .collect()
    }
}
