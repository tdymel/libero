use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::CssLayer;
use crate::css::Stylesheet;

/// Where a sheet sorts among the others in its layer. Two rules of equal
/// specificity in one layer are decided by source order, and without a rank
/// that order is the CSS hash's - arbitrary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SheetRank {
    /// Ahead of every other sheet in the layer, so any of them overrides it
    /// at equal specificity. Only `Box`'s focus ring: a component's own
    /// `:focus-visible` has the same specificity and has to win.
    Default,
    /// Hash order.
    Component,
}

/// Only [`StylesheetRegistry::acquire`] mints one, so nothing can be released
/// under a key the registry never issued.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StylesheetKey {
    layer: CssLayer,
    rank: SheetRank,
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

impl RegisteredStylesheet {
    /// Whether this entry really holds `stylesheet`'s CSS. Debug-only: it
    /// re-renders the layered text, which is the allocation `css` exists to
    /// avoid - see [`StylesheetRegistry::acquire`].
    #[cfg(debug_assertions)]
    fn holds(&self, layer: CssLayer, stylesheet: &Stylesheet) -> bool {
        *self.css == *layered_css(layer, stylesheet)
    }
}

fn layered_css(layer: CssLayer, stylesheet: &Stylesheet) -> String {
    format!("@layer {}{{{}}}", layer.css_name(), stylesheet.as_str())
}

#[derive(Clone, Default)]
pub struct StylesheetRegistry {
    inner: Rc<RefCell<BTreeMap<StylesheetKey, RegisteredStylesheet>>>,
}

impl StylesheetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire(
        &self,
        stylesheet: impl Into<Stylesheet>,
        layer: CssLayer,
        rank: SheetRank,
    ) -> StylesheetKey {
        let stylesheet = stylesheet.into();
        let key = StylesheetKey {
            layer,
            rank,
            hash: stylesheet.hash(),
        };
        let mut registry = self.inner.borrow_mut();

        let entry = registry.entry(key).or_insert_with(|| RegisteredStylesheet {
            // Not a `Stylesheet`: that re-hashes text for a key we have.
            // The rank is part of the key, so the same CSS on both ranks is
            // two entries and needs two node keys.
            node_key: Rc::from(match rank {
                SheetRank::Default => format!("{}-default-{:x}", layer.css_name(), key.hash),
                SheetRank::Component => format!("{}-{:x}", layer.css_name(), key.hash),
            }),
            css: Rc::from(layered_css(layer, &stylesheet)),
            ref_count: 0,
        });

        // The key is a 64-bit hash of the CSS text, and the same number mints
        // the class name, so a collision would render one sheet with
        // another's CSS *and* share its refcount - silently, and identically
        // on every reload. Too unlikely to design around, far too quiet to
        // leave undetected.
        #[cfg(debug_assertions)]
        if !entry.holds(layer, &stylesheet) {
            crate::utils::warn(&format!(
                "StylesheetRegistry: CSS hash collision on {}-{:x} - two different \
                 stylesheets share one class name, and one will render with the \
                 other's CSS.",
                layer.css_name(),
                key.hash,
            ));
        }

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

        let key = registry.acquire(
            stylesheet.clone(),
            CssLayer::UserCustom,
            SheetRank::Component,
        );
        assert_eq!(registry.stylesheets().len(), 1);

        let second_key = registry.acquire(stylesheet, CssLayer::UserCustom, SheetRank::Component);
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

        let framework = registry.acquire(
            stylesheet.clone(),
            CssLayer::Framework,
            SheetRank::Component,
        );
        let custom = registry.acquire(stylesheet, CssLayer::UserCustom, SheetRank::Component);

        assert_ne!(framework, custom);
        assert_eq!(registry.stylesheets().len(), 2);

        registry.release(framework);
        assert_eq!(registry.stylesheets().len(), 1);
    }

    /// Stands in for a real hash collision, which can't be constructed to
    /// order: the check is what turns one into a visible warning instead of
    /// silently wrong styles.
    #[cfg(debug_assertions)]
    #[test]
    fn an_entry_does_not_hold_a_different_sheets_css() {
        let registry = StylesheetRegistry::new();
        let key = registry.acquire(
            Stylesheet::from(&sx().padding("lg")),
            CssLayer::Framework,
            SheetRank::Component,
        );
        let registry = registry.inner.borrow();
        let entry = registry.get(&key).expect("just acquired");

        assert!(entry.holds(CssLayer::Framework, &Stylesheet::from(&sx().padding("lg"))));
        assert!(!entry.holds(CssLayer::Framework, &Stylesheet::from(&sx().color("red"))));
        assert!(!entry.holds(CssLayer::UserCustom, &Stylesheet::from(&sx().padding("lg"))));
    }

    #[test]
    fn an_entry_carries_its_layer_in_both_its_node_key_and_its_css() {
        let registry = StylesheetRegistry::new();
        registry.acquire(
            Stylesheet::from(&sx().padding("lg")),
            CssLayer::Framework,
            SheetRank::Component,
        );

        let (node_key, css) = registry.stylesheets().remove(0);

        assert!(node_key.starts_with("lsx-framework-"));
        assert!(css.starts_with("@layer lsx-framework{"));
    }

    /// `Box`'s ring and a component's own `:focus-visible` tie on
    /// specificity, so the ring has to come first whatever the two hash to.
    /// Several sheets, so some hash below the default one and some above.
    #[test]
    fn a_default_ranked_sheet_comes_before_every_other_in_its_layer() {
        let registry = StylesheetRegistry::new();
        for color in ["red", "blue", "green", "black", "white", "gray"] {
            registry.acquire(
                Stylesheet::from(&sx().color(color)),
                CssLayer::Framework,
                SheetRank::Component,
            );
        }
        registry.acquire(
            Stylesheet::from(&sx().padding("lg")),
            CssLayer::Framework,
            SheetRank::Default,
        );

        let (node_key, css) = registry.stylesheets().remove(0);

        assert!(node_key.starts_with("lsx-framework-default-"));
        assert!(css.starts_with("@layer lsx-framework{"));
    }

    #[test]
    fn the_same_css_on_both_ranks_is_two_node_keys() {
        let registry = StylesheetRegistry::new();
        let stylesheet = Stylesheet::from(&sx().padding("lg"));
        registry.acquire(stylesheet.clone(), CssLayer::Framework, SheetRank::Default);
        registry.acquire(stylesheet, CssLayer::Framework, SheetRank::Component);

        let keys = registry.stylesheets();

        assert_eq!(keys.len(), 2);
        assert_ne!(keys[0].0, keys[1].0);
    }
}
