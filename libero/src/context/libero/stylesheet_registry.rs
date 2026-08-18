use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::CssLayer;
use crate::css::Stylesheet;

/// Identifies a registry entry. Only [`StylesheetRegistry::acquire`] mints
/// one, so a caller can't release under a key the registry never used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StylesheetKey {
    layer: CssLayer,
    hash: u64,
}

#[derive(Clone)]
struct RegisteredStylesheet {
    css: Stylesheet,
    ref_count: usize,
}

#[derive(Clone, Default)]
pub struct StylesheetRegistry {
    inner: Rc<RefCell<BTreeMap<StylesheetKey, RegisteredStylesheet>>>,
}

impl StylesheetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(&self, stylesheet: impl Into<Stylesheet>, layer: CssLayer) -> StylesheetKey {
        let stylesheet = stylesheet.into();
        let key = StylesheetKey {
            layer,
            hash: stylesheet.hash(),
        };
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
        key
    }

    pub fn release(&self, key: StylesheetKey) {
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
            .map(|(key, entry)| {
                (
                    format!("{}-{:x}", key.layer.css_name(), key.hash),
                    entry.css.as_str().to_string(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sx::sx;

    #[test]
    fn acquire_release_round_trip_removes_the_entry() {
        let registry = StylesheetRegistry::new();
        let stylesheet = Stylesheet::from(&sx().padding("lg"));

        let key = registry.acquire(stylesheet.clone(), CssLayer::UserCustom);
        assert_eq!(registry.stylesheets().len(), 1);

        let second_key = registry.acquire(stylesheet, CssLayer::UserCustom);
        assert_eq!(key, second_key);
        assert_eq!(registry.stylesheets().len(), 1);

        registry.release(key);
        assert_eq!(registry.stylesheets().len(), 1);

        registry.release(second_key);
        assert!(registry.stylesheets().is_empty());
    }

    #[test]
    fn the_same_css_on_two_layers_is_two_entries() {
        let registry = StylesheetRegistry::new();
        let stylesheet = Stylesheet::from(&sx().padding("lg"));

        let framework = registry.acquire(stylesheet.clone(), CssLayer::Framework);
        let custom = registry.acquire(stylesheet, CssLayer::UserCustom);

        assert_ne!(framework, custom);
        assert_eq!(registry.stylesheets().len(), 2);

        registry.release(framework);
        assert_eq!(registry.stylesheets().len(), 1);
    }
}
