use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

use super::CssLayer;
use crate::css::Stylesheet;

/// Unused sheets kept mounted: Chromium restyles and relays out the whole page per `@layer` sheet added or removed.
const RETAINED: usize = 256;

/// Where a sheet sorts in its layer: without a rank, source order between equal
/// specificity rules is the CSS hash's, i.e. arbitrary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SheetRank {
    /// First in the layer, so any other sheet overrides it at equal specificity.
    /// Only `Box`'s focus ring, which a component's own `:focus-visible` must beat.
    Default,
    /// Hash order.
    Component,
}

/// Minted only by [`StylesheetRegistry::acquire`], so no foreign key can be released.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StylesheetKey {
    layer: CssLayer,
    rank: SheetRank,
    hash: u64,
}

/// Built once at `acquire`. `Rc<str>`: the outlet calls
/// [`StylesheetRegistry::stylesheets`] every render, so it must not re-allocate.
#[derive(Clone)]
struct RegisteredStylesheet {
    node_key: Rc<str>,
    css: Rc<str>,
    ref_count: usize,
    /// When it last fell to no users, matched against its `retired` queue entry.
    retired_at: u64,
    /// Only a sheet scoped to its own class matches nothing once unused; a global
    /// (`:root`, raw CSS) one must leave the cascade at its last release.
    retainable: bool,
}

impl RegisteredStylesheet {
    /// Whether this entry holds `stylesheet`'s CSS. Debug-only: it re-renders
    /// the layered text, the allocation `css` exists to avoid.
    #[cfg(debug_assertions)]
    fn holds(&self, layer: CssLayer, stylesheet: &Stylesheet) -> bool {
        *self.css == *layered_css(layer, stylesheet)
    }
}

fn layered_css(layer: CssLayer, stylesheet: &Stylesheet) -> String {
    format!("@layer {}{{{}}}", layer.css_name(), stylesheet.as_str())
}

#[derive(Default)]
struct Sheets {
    by_key: BTreeMap<StylesheetKey, RegisteredStylesheet>,
    /// Sheets that fell to no users, oldest first; an entry is stale once its sheet was used again.
    retired: VecDeque<(StylesheetKey, u64)>,
    clock: u64,
}

/// The sheets mounted components use, refcounted by layer, rank and CSS hash, so one
/// `<style>` serves every instance. The last [`RETAINED`] unused ones stay. Clones share it.
#[derive(Clone, Default)]
pub struct StylesheetRegistry {
    inner: Rc<RefCell<Sheets>>,
}

impl StylesheetRegistry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Counts one more user of `stylesheet`, registering it on the first; pair with [`release`](Self::release).
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

        let entry = registry
            .by_key
            .entry(key)
            .or_insert_with(|| RegisteredStylesheet {
                // Not a `Stylesheet`, which would re-hash. The rank is in the key,
                // so the same CSS on both ranks needs two node keys.
                node_key: Rc::from(match rank {
                    SheetRank::Default => format!("{}-default-{:x}", layer.css_name(), key.hash),
                    SheetRank::Component => format!("{}-{:x}", layer.css_name(), key.hash),
                }),
                css: Rc::from(layered_css(layer, &stylesheet)),
                ref_count: 0,
                retired_at: 0,
                retainable: stylesheet.class_name().is_some(),
            });

        // A 64-bit hash collision would silently render one sheet with another's
        // CSS and share its refcount: too rare to design around, too quiet to ignore.
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

    /// Counts one user less. A class-scoped sheet without users stays until [`RETAINED`] newer
    /// ones push it out, so a class toggled back costs no `<style>` write; a global one goes at once.
    pub fn release(&self, key: StylesheetKey) {
        let mut registry = self.inner.borrow_mut();
        let registry = &mut *registry;

        let Some(entry) = registry.by_key.get_mut(&key) else {
            return;
        };
        if entry.ref_count == 0 {
            return;
        }
        entry.ref_count -= 1;
        if entry.ref_count > 0 {
            return;
        }
        if !entry.retainable {
            registry.by_key.remove(&key);
            return;
        }
        registry.clock += 1;
        entry.retired_at = registry.clock;
        registry.retired.push_back((key, registry.clock));

        while registry.retired.len() > RETAINED {
            let Some((old, at)) = registry.retired.pop_front() else {
                break;
            };
            if registry
                .by_key
                .get(&old)
                .is_some_and(|entry| entry.ref_count == 0 && entry.retired_at == at)
            {
                registry.by_key.remove(&old);
            }
        }
    }

    /// Every registered sheet as `(node key, CSS)`. Cheap: both are refcounted.
    pub fn stylesheets(&self) -> Vec<(Rc<str>, Rc<str>)> {
        self.inner
            .borrow()
            .by_key
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
    fn acquire_release_round_trip_keeps_one_entry_for_reuse() {
        let registry = StylesheetRegistry::new();
        let stylesheet = Stylesheet::from(&sx().padding("lg"));

        let key = registry.acquire(
            stylesheet.clone(),
            CssLayer::UserCustom,
            SheetRank::Component,
        );
        assert_eq!(registry.stylesheets().len(), 1);

        let second_key = registry.acquire(
            stylesheet.clone(),
            CssLayer::UserCustom,
            SheetRank::Component,
        );
        assert_eq!(key, second_key);
        assert_eq!(registry.stylesheets().len(), 1);

        registry.release(key);
        registry.release(second_key);
        assert_eq!(registry.stylesheets().len(), 1, "kept, unused");

        registry.acquire(stylesheet, CssLayer::UserCustom, SheetRank::Component);
        assert_eq!(registry.stylesheets().len(), 1);
    }

    fn padding(registry: &StylesheetRegistry, px: usize) -> StylesheetKey {
        registry.acquire(
            Stylesheet::from(&sx().padding(format!("{px}px"))),
            CssLayer::UserCustom,
            SheetRank::Component,
        )
    }

    #[test]
    fn unused_sheets_past_the_cap_go_oldest_first() {
        let registry = StylesheetRegistry::new();
        let keys: Vec<_> = (0..=RETAINED).map(|px| padding(&registry, px)).collect();
        for &key in &keys {
            registry.release(key);
        }

        let sheets = registry.inner.borrow();
        assert_eq!(sheets.by_key.len(), RETAINED);
        assert!(!sheets.by_key.contains_key(&keys[0]));
        assert!(sheets.by_key.contains_key(&keys[RETAINED]));
    }

    #[test]
    fn a_sheet_used_again_is_not_evicted_by_its_older_retirement() {
        let registry = StylesheetRegistry::new();
        let first = padding(&registry, 0);
        registry.release(first);
        padding(&registry, 0);

        for px in 1..=RETAINED {
            let key = padding(&registry, px);
            registry.release(key);
        }

        assert!(
            registry.inner.borrow().by_key.contains_key(&first),
            "in use"
        );
        registry.release(first);
        assert!(
            registry.inner.borrow().by_key.contains_key(&first),
            "newest unused"
        );
    }

    /// A kept `:root` sheet would stay in the cascade (todo 2195).
    #[test]
    fn a_released_root_sheet_is_removed_and_a_class_scoped_one_stays() {
        let registry = StylesheetRegistry::new();
        let root = registry.acquire(
            ":root{--height:64px;}",
            CssLayer::Framework,
            SheetRank::Component,
        );
        let scoped = padding(&registry, 0);

        registry.release(root);
        registry.release(scoped);

        let sheets = registry.inner.borrow();
        assert!(!sheets.by_key.contains_key(&root));
        assert!(sheets.by_key.contains_key(&scoped));
        assert_eq!(sheets.retired.len(), 1, "only the scoped one queued");
    }

    #[test]
    fn a_release_past_zero_changes_nothing() {
        let registry = StylesheetRegistry::new();
        let key = padding(&registry, 0);
        registry.release(key);
        registry.release(key);

        assert_eq!(registry.inner.borrow().retired.len(), 1);
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
    }

    /// Stands in for a real hash collision, which can't be constructed to order.
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
        let entry = registry.by_key.get(&key).expect("just acquired");

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

    /// The ring must come first whatever it hashes to. Several sheets, so some
    /// hash below the default one and some above.
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
