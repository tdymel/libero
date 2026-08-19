use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::CssLayer;
use crate::css::Stylesheet;

/// Only [`StylesheetRegistry::acquire`] mints one, so nothing can be released
/// under a key the registry never issued.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StylesheetKey {
    layer: CssLayer,
    hash: u64,
}

/// Built once at `acquire`. `Rc<str>` so
/// [`StylesheetRegistry::stylesheets`], called on every style-outlet render,
/// bumps a refcount instead of re-allocating the whole CSS corpus.
#[derive(Clone)]
struct RegisteredStylesheet {
    node_key: Rc<str>,
    css: Rc<str>,
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
            // Not a `Stylesheet`: that re-hashes text for a key we have.
            node_key: Rc::from(format!("{}-{:x}", layer.css_name(), key.hash)),
            css: Rc::from(format!(
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
            entry.ref_count = entry.ref_count.saturating_sub(1);
            if entry.ref_count == 0 {
                registry.remove(&key);
            }
        }
    }

    /// Every registered sheet as `(node key, CSS)`. Cheap - both halves are
    /// refcounted handles.
    pub fn stylesheets(&self) -> Vec<(Rc<str>, Rc<str>)> {
        self.inner
            .borrow()
            .values()
            .map(|entry| (entry.node_key.clone(), entry.css.clone()))
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

    #[test]
    fn an_entry_carries_its_layer_in_both_its_node_key_and_its_css() {
        let registry = StylesheetRegistry::new();
        registry.acquire(Stylesheet::from(&sx().padding("lg")), CssLayer::Framework);

        let (node_key, css) = registry.stylesheets().remove(0);

        assert!(node_key.starts_with("lsx-framework-"));
        assert!(css.starts_with("@layer lsx-framework{"));
    }
}
