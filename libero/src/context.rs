use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;

use crate::{css::Stylesheet, sx::Sx, theme::Theme};

#[derive(Clone)]
pub struct RegisteredStylesheet {
    pub sx: &'static Sx,
    pub css: Stylesheet,
    pub registrations: Rc<()>,
}

#[derive(Clone)]
struct RegistrationGuard {
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
    inner: Rc<RefCell<BTreeMap<u64, RegisteredStylesheet>>>,
}

impl SxRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn registration_token(&self, sx: &'static Sx) -> Rc<()> {
        let key = sx.hash();
        let mut registry = self.inner.borrow_mut();

        if let Some(entry) = registry.get(&key) {
            return entry.registrations.clone();
        }

        let registrations = Rc::new(());
        registry.insert(
            key,
            RegisteredStylesheet {
                sx,
                css: sx.to_css(),
                registrations: registrations.clone(),
            },
        );

        registrations
    }

    fn register(&self, sx: &'static Sx, sx_registry_version: Signal<u64>) -> RegistrationGuard {
        RegistrationGuard {
            registrations: self.registration_token(sx),
            sx_registry_version,
        }
    }

    pub fn stylesheets(&self) -> Vec<String> {
        let mut registry = self.inner.borrow_mut();
        registry.retain(|_, entry| Rc::strong_count(&entry.registrations) > 1);
        registry
            .values()
            .map(|entry| {
                debug_assert_eq!(
                    entry.sx.class_name(),
                    format!("lsx-{:016x}", entry.sx.hash())
                );
                entry.css.as_str().to_string()
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct LiberoContext {
    pub theme: &'static Theme,
    pub(crate) theme_css: String,
    pub(crate) sx_registry: SxRegistry,
    pub(crate) sx_registry_version: Signal<u64>,
}

impl LiberoContext {
    pub fn new(theme: &'static Theme, sx_registry_version: Signal<u64>) -> Self {
        Self {
            theme,
            theme_css: theme.to_css().as_str().to_string(),
            sx_registry: SxRegistry::new(),
            sx_registry_version,
        }
    }
}

#[component]
pub fn LiberoProvider(theme: &'static Theme, children: Element) -> Element {
    let sx_registry_version = use_signal(|| 0u64);
    let context = use_context_provider(|| LiberoContext::new(theme, sx_registry_version));
    let _registry_version = context.sx_registry_version.read();
    let active_stylesheets = context.sx_registry.stylesheets();

    rsx! {
        style {
            dangerous_inner_html: "{context.theme_css}"
        }
        for stylesheet in active_stylesheets {
            style {
                dangerous_inner_html: "{stylesheet}"
            }
        }
        {children}
    }
}

pub fn use_theme() -> &'static Theme {
    use_context::<LiberoContext>().theme
}

pub fn use_sx(sx: &'static crate::sx::Sx) {
    let mut context = use_context::<LiberoContext>();
    let _registration = use_hook(|| {
        *context.sx_registry_version.write() += 1;
        context
            .sx_registry
            .register(sx, context.sx_registry_version)
    });
}
