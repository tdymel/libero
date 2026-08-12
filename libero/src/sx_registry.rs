use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{SxLayer, css::Stylesheet, sx::Sx};

#[derive(Clone)]
pub struct RegisteredStylesheet {
    pub key: (SxLayer, u64),
    pub class_name: String,
    pub css: Stylesheet,
    pub ref_count: usize,
}

#[derive(Clone, Default)]
pub struct SxRegistry {
    inner: Rc<RefCell<BTreeMap<(SxLayer, u64), RegisteredStylesheet>>>,
}

impl SxRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self, sx: &Sx, layer: SxLayer) -> String {
        let key = (layer, sx.hash());
        let mut registry = self.inner.borrow_mut();

        let entry = registry.entry(key).or_insert_with(|| RegisteredStylesheet {
            key,
            class_name: sx.class_name(),
            css: Stylesheet::from(format!(
                "@layer {}{{{}}}",
                layer.css_name(),
                Stylesheet::from(sx).as_str()
            )),
            ref_count: 0,
        });

        entry.ref_count += 1;
        entry.class_name.clone()
    }

    pub fn release(&self, key: (SxLayer, u64)) {
        let mut registry = self.inner.borrow_mut();

        if let Some(entry) = registry.get_mut(&key) {
            entry.ref_count -= 1;
            if entry.ref_count == 0 {
                registry.remove(&key);
            }
        }
    }

    pub fn stylesheets(&self) -> Vec<String> {
        self.inner
            .borrow()
            .values()
            .map(|entry| {
                debug_assert_eq!(entry.class_name, format!("lsx-{:016x}", entry.key.1));
                entry.css.as_str().to_string()
            })
            .collect()
    }
}
