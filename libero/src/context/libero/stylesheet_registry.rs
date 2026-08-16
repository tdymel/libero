use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::CssLayer;
use crate::css::Stylesheet;

#[derive(Clone)]
struct RegisteredStylesheet {
    css: Stylesheet,
    ref_count: usize,
}

#[derive(Clone, Default)]
pub struct StylesheetRegistry {
    inner: Rc<RefCell<BTreeMap<(CssLayer, u64), RegisteredStylesheet>>>,
}

impl StylesheetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self, stylesheet: impl Into<Stylesheet>, layer: CssLayer) {
        let stylesheet = stylesheet.into();
        let key = (layer, stylesheet.hash());
        let mut registry = self.inner.borrow_mut();

        let entry = registry.entry(key).or_insert_with(|| RegisteredStylesheet {
            css: Stylesheet::from(format!(
                "@layer {}{{{}}}",
                layer.css_name(),
                stylesheet.as_str()
            )),
            ref_count: 0,
        });

        entry.ref_count += 1;
    }

    pub fn release(&self, key: (CssLayer, u64)) {
        let mut registry = self.inner.borrow_mut();

        if let Some(entry) = registry.get_mut(&key) {
            entry.ref_count -= 1;
            if entry.ref_count == 0 {
                registry.remove(&key);
            }
        }
    }

    pub fn stylesheets(&self) -> Vec<(String, String)> {
        self.inner
            .borrow()
            .iter()
            .map(|((layer, hash), entry)| {
                (
                    format!("{}-{hash:x}", layer.css_name()),
                    entry.css.as_str().to_string(),
                )
            })
            .collect()
    }
}
