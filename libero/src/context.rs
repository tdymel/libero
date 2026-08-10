use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use dioxus::prelude::*;

use crate::{css::Stylesheet, theme::Theme};

#[derive(Clone)]
pub struct RegisteredStylesheet {
    pub id: Rc<String>,
    pub css: String,
}

#[derive(Clone, Default)]
pub struct SxRegistry {
    // TODO: Store shared stylesheet handles (e.g. Rc<Stylesheet>) to avoid copying
    // whole Stylesheet values when collecting active entries for rendering.
    inner: Rc<RefCell<BTreeMap<String, RegisteredStylesheet>>>,
}

impl SxRegistry {
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
}

impl LiberoContext {
    pub fn new(theme: &'static Theme) -> Self {
        Self {
            theme,
            theme_css: theme.to_css().as_str().to_string(),
            sx_registry: SxRegistry::default(),
        }
    }
}

#[component]
pub fn LiberoProvider(theme: &'static Theme, children: Element) -> Element {
    let context = use_context_provider(|| LiberoContext::new(theme));
    // let active_stylesheets = context.sx_registry.stylesheets();

    rsx! {
        style {
            dangerous_inner_html: "{context.theme_css}"
        }
        // for stylesheet in active_stylesheets {
        //     style {
        //         dangerous_inner_html: "{stylesheet}"
        //     }
        // }
        {children}
    }
}

pub fn use_theme() -> &'static Theme {
    use_context::<LiberoContext>().theme
}

pub fn use_sx(sx: &'static crate::sx::Sx) -> String {
    let context = use_context::<LiberoContext>();
    let stylesheet = sx.to_css();
    let id = use_hook(|| context.sx_registry.register(stylesheet));

    match id.as_ref() {
        Some(id) => id.as_str().to_string(),
        None => String::new(),
    }
}
