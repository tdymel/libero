use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;

use crate::{css::Stylesheet, theme::Theme};

#[derive(Clone)]
pub struct RegisteredStylesheet {
    pub id: Rc<String>,
    pub css: String,
}

#[derive(Clone)]
pub struct RegisteredSxHandle {
    id: Option<Rc<String>>,
    sx_registry_version: Signal<u64>,
}

impl RegisteredSxHandle {
    fn class_name(&self) -> String {
        match &self.id {
            Some(id) => id.as_str().to_string(),
            None => String::new(),
        }
    }
}

impl Drop for RegisteredSxHandle {
    fn drop(&mut self) {
        self.id.take();
        *self.sx_registry_version.write() += 1;
    }
}

#[derive(Clone, Default)]
pub struct SxRegistry {
    inner: Rc<RefCell<BTreeMap<String, RegisteredStylesheet>>>,
}

impl SxRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, stylesheet: Stylesheet) -> Option<Rc<String>> {
        let Some(class_name) = stylesheet.class_name() else {
            return None;
        };

        let key = class_name.to_string();
        let mut registry = self.inner.borrow_mut();

        if let Some(entry) = registry.get(&key) {
            return Some(entry.id.clone());
        }

        let id = Rc::new(class_name.to_string());
        registry.insert(
            key,
            RegisteredStylesheet {
                id: id.clone(),
                css: stylesheet.as_str().to_string(),
            },
        );

        Some(id)
    }

    pub fn stylesheets(&self) -> Vec<String> {
        let mut registry = self.inner.borrow_mut();
        registry.retain(|_, entry| Rc::strong_count(&entry.id) > 1);
        registry.values().map(|entry| entry.css.clone()).collect()
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

pub fn use_sx(sx: &'static crate::sx::Sx) -> String {
    let mut context = use_context::<LiberoContext>();
    let stylesheet = sx.to_css();
    let registration = use_hook(|| {
        let id = context.sx_registry.register(stylesheet);
        *context.sx_registry_version.write() += 1;
        RegisteredSxHandle {
            id,
            sx_registry_version: context.sx_registry_version,
        }
    });

    registration.class_name()
}
