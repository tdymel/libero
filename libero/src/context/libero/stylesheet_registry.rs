use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

use super::CssLayer;
use crate::{css::Stylesheet, platform::StyleRulesApi};

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
    /// Its rules live in the outlet's layer blocks, not in a `<style>` of its own.
    inserted: bool,
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

/// `css`'s top-level rules, each with its braces; quoted braces don't count.
fn top_level_rules(css: &str) -> impl Iterator<Item = &str> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut start = 0;
    css.char_indices().filter_map(move |(at, c)| {
        if escaped {
            escaped = false;
            return None;
        }
        match (quote, c) {
            (_, '\\') => escaped = true,
            (Some(open), c) if c == open => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '{') => depth += 1,
            (None, '}') if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    let rule = css[start..=at].trim();
                    start = at + 1;
                    return Some(rule);
                }
            }
            _ => {}
        }
        None
    })
}

/// `css`'s rules for a layer block, `@media P{.a{x}}` nested as `.a{@media P{x}}`: inserting
/// a grouping rule costs Chromium the whole-page restyle, a nested one 1-2 ms (todo 2231).
/// `None` for any other at-rule shape, or without `nests`: the sheet keeps its `<style>`.
fn block_rules(css: &str, nests: bool) -> Option<Vec<Cow<'_, str>>> {
    const GROUPING: [&str; 3] = ["@media", "@container", "@supports"];
    let mut rules = Vec::new();
    for rule in top_level_rules(css) {
        if !rule.starts_with('@') {
            rules.push(Cow::Borrowed(rule));
            continue;
        }
        let (prelude, inner) = rule.split_once('{')?;
        let inner = inner.strip_suffix('}')?;
        // A forced reduced motion rewrites a motion query's text in the outlet.
        if !nests
            || !GROUPING.iter().any(|name| prelude.starts_with(name))
            || prelude.contains("prefers-reduced-motion")
            || inner.contains('@')
        {
            return None;
        }
        for style in top_level_rules(inner) {
            let (selector, body) = style.split_once('{')?;
            let body = body.strip_suffix('}')?;
            let pseudo_element = selector.contains("::")
                || [":before", ":after", ":first-line", ":first-letter"]
                    .iter()
                    .any(|name| selector.contains(name));
            if pseudo_element
                || body.contains('{')
                || selector.matches(['"', '\'']).count() % 2 == 1
            {
                return None;
            }
            rules.push(Cow::Owned(format!("{selector}{{{prelude}{{{body}}}}}")));
        }
    }
    Some(rules)
}

#[derive(Default)]
struct Sheets {
    by_key: BTreeMap<StylesheetKey, RegisteredStylesheet>,
    /// Sheets that fell to no users, oldest first; an entry is stale once its sheet was used again.
    retired: VecDeque<(StylesheetKey, u64)>,
    clock: u64,
    /// The outlet's mounted layer blocks: a new layer sheet costs Chromium a whole-page
    /// restyle, a rule inserted into a block about 1 ms (todo 2186). Web only.
    blocks: Option<Box<dyn StyleRulesApi>>,
    /// The blocks' browser nests rules, so a sheet's at-rules can go in as nested rules.
    nests: bool,
    /// Per layer, the inserted sheets in rule order with how many rules each holds.
    inserted: [Vec<(StylesheetKey, usize)>; 4],
    /// A `<style>` entry came or went since the outlet last heard.
    outlet_changed: bool,
}

impl Sheets {
    /// Puts a class-scoped sheet's rules at the end of its layer's block, see [`block_rules`].
    /// Global sheets stay `<style>`s.
    fn insert(&mut self, key: StylesheetKey, stylesheet: &Stylesheet) -> bool {
        let Some(blocks) = &self.blocks else {
            return false;
        };
        if key.rank != SheetRank::Component || stylesheet.class_name().is_none() {
            return false;
        }
        let Some(rules) = block_rules(stylesheet.as_str(), self.nests) else {
            return false;
        };
        let order = &mut self.inserted[key.layer.index()];
        let start: usize = order.iter().map(|(_, count)| count).sum();
        let mut end = start;
        for rule in &rules {
            // A rule the browser rejects is dropped, as a `<style>` would drop it.
            if blocks.insert(key.layer.index(), end, rule) {
                end += 1;
            }
        }
        if end == start {
            return false;
        }
        order.push((key, end - start));
        true
    }

    fn remove(&mut self, key: &StylesheetKey) {
        let Some(entry) = self.by_key.remove(key) else {
            return;
        };
        if !entry.inserted {
            self.outlet_changed = true;
            return;
        }
        let order = &mut self.inserted[key.layer.index()];
        let Some(at) = order.iter().position(|(inserted, _)| inserted == key) else {
            return;
        };
        let (_, count) = order.remove(at);
        let index = order[..at].iter().map(|(_, count)| count).sum();
        if let Some(blocks) = &self.blocks {
            for _ in 0..count {
                blocks.delete(key.layer.index(), index);
            }
        }
    }
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
        let registry = &mut *registry;

        if !registry.by_key.contains_key(&key) {
            let inserted = registry.insert(key, &stylesheet);
            registry.outlet_changed |= !inserted;
            registry.by_key.insert(
                key,
                RegisteredStylesheet {
                    // Not a `Stylesheet`, which would re-hash. The rank is in the key,
                    // so the same CSS on both ranks needs two node keys.
                    node_key: Rc::from(match rank {
                        SheetRank::Default => {
                            format!("{}-default-{:x}", layer.css_name(), key.hash)
                        }
                        SheetRank::Component => format!("{}-{:x}", layer.css_name(), key.hash),
                    }),
                    css: Rc::from(layered_css(layer, &stylesheet)),
                    ref_count: 0,
                    retired_at: 0,
                    retainable: stylesheet.class_name().is_some(),
                    inserted,
                },
            );
        }
        let entry = registry.by_key.get_mut(&key).expect("registered above");

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
            registry.remove(&key);
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
                registry.remove(&old);
            }
        }
    }

    /// Every sheet on `rank` that needs its own `<style>`, as `(node key, CSS)`.
    /// Cheap: both are refcounted.
    pub fn stylesheets(&self, rank: SheetRank) -> Vec<(Rc<str>, Rc<str>)> {
        self.inner
            .borrow()
            .by_key
            .iter()
            .filter(|(key, entry)| key.rank == rank && !entry.inserted)
            .map(|(_, entry)| (entry.node_key.clone(), entry.css.clone()))
            .collect()
    }

    /// Sends later plain class-scoped sheets into `blocks`, one per [`CssLayer`]; the
    /// sheets registered so far keep their `<style>`, as a hydrating client must.
    pub(crate) fn attach(&self, blocks: Box<dyn StyleRulesApi>) {
        let mut sheets = self.inner.borrow_mut();
        sheets.nests = blocks.nests();
        sheets.blocks = Some(blocks);
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.inner.borrow().by_key.len()
    }

    /// Whether the outlet's `<style>` list changed since the last call.
    pub(crate) fn take_outlet_change(&self) -> bool {
        std::mem::take(&mut self.inner.borrow_mut().outlet_changed)
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
        assert_eq!(registry.len(), 1);

        let second_key = registry.acquire(
            stylesheet.clone(),
            CssLayer::UserCustom,
            SheetRank::Component,
        );
        assert_eq!(key, second_key);
        assert_eq!(registry.len(), 1);

        registry.release(key);
        registry.release(second_key);
        assert_eq!(registry.len(), 1, "kept, unused");

        registry.acquire(stylesheet, CssLayer::UserCustom, SheetRank::Component);
        assert_eq!(registry.len(), 1);
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
        assert_eq!(registry.len(), 2);
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

        let (node_key, css) = registry.stylesheets(SheetRank::Component).remove(0);

        assert!(node_key.starts_with("lsx-framework-"));
        assert!(css.starts_with("@layer lsx-framework{"));
    }

    /// The outlet renders the ring's list first, whatever it hashes to.
    #[test]
    fn a_default_ranked_sheet_is_listed_apart_from_the_component_ones() {
        let registry = StylesheetRegistry::new();
        for color in ["red", "blue", "green"] {
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

        let defaults = registry.stylesheets(SheetRank::Default);

        assert_eq!(defaults.len(), 1);
        assert!(defaults[0].0.starts_with("lsx-framework-default-"));
        assert_eq!(registry.stylesheets(SheetRank::Component).len(), 3);
    }

    #[test]
    fn the_same_css_on_both_ranks_is_two_node_keys() {
        let registry = StylesheetRegistry::new();
        let stylesheet = Stylesheet::from(&sx().padding("lg"));
        registry.acquire(stylesheet.clone(), CssLayer::Framework, SheetRank::Default);
        registry.acquire(stylesheet, CssLayer::Framework, SheetRank::Component);

        let default = registry.stylesheets(SheetRank::Default);
        let component = registry.stylesheets(SheetRank::Component);

        assert_ne!(default[0].0, component[0].0);
    }

    #[test]
    fn top_level_rules_split_at_depth_zero_and_skip_quoted_braces() {
        let css = r#".a{color:red}  .a:hover{content:"}{"}.a{&:focus{color:blue}}"#;

        let rules: Vec<_> = top_level_rules(css).collect();

        assert_eq!(
            rules,
            [
                ".a{color:red}",
                r#".a:hover{content:"}{"}"#,
                ".a{&:focus{color:blue}}"
            ]
        );
    }

    /// Records each block edit and keeps the blocks' rules, as the CSSOM would.
    #[derive(Clone, Default)]
    struct FakeRules {
        blocks: Rc<RefCell<[Vec<String>; 4]>>,
        reject: Option<&'static str>,
        nests: bool,
    }

    impl StyleRulesApi for FakeRules {
        fn insert(&self, block: usize, index: usize, rule: &str) -> bool {
            if self.reject.is_some_and(|bad| rule.contains(bad)) {
                return false;
            }
            self.blocks.borrow_mut()[block].insert(index, rule.to_string());
            true
        }

        fn delete(&self, block: usize, index: usize) {
            self.blocks.borrow_mut()[block].remove(index);
        }

        fn nests(&self) -> bool {
            self.nests
        }
    }

    fn attached() -> (StylesheetRegistry, FakeRules) {
        let registry = StylesheetRegistry::new();
        let rules = FakeRules {
            nests: true,
            ..FakeRules::default()
        };
        registry.attach(Box::new(rules.clone()));
        (registry, rules)
    }

    fn block(rules: &FakeRules, layer: CssLayer) -> Vec<String> {
        rules.blocks.borrow()[layer.index()].clone()
    }

    #[test]
    fn a_plain_scoped_sheet_goes_into_its_layer_block_not_a_style() {
        let (registry, rules) = attached();
        let stylesheet = Stylesheet::from(&sx().padding("lg"));

        registry.acquire(
            stylesheet.clone(),
            CssLayer::UserStatic,
            SheetRank::Component,
        );

        assert!(registry.stylesheets(SheetRank::Component).is_empty());
        assert_eq!(
            block(&rules, CssLayer::UserStatic).concat(),
            stylesheet.as_str()
        );
        assert!(!registry.take_outlet_change(), "the outlet has nothing new");
    }

    /// Sheets registered before the blocks mounted keep the `<style>` the server rendered.
    #[test]
    fn a_sheet_registered_before_attach_keeps_its_style() {
        let registry = StylesheetRegistry::new();
        registry.acquire(
            Stylesheet::from(&sx().padding("lg")),
            CssLayer::UserStatic,
            SheetRank::Component,
        );
        let rules = FakeRules::default();
        registry.attach(Box::new(rules.clone()));
        registry.acquire(
            Stylesheet::from(&sx().padding("lg")),
            CssLayer::UserStatic,
            SheetRank::Component,
        );

        assert_eq!(registry.stylesheets(SheetRank::Component).len(), 1);
        assert!(block(&rules, CssLayer::UserStatic).is_empty());
    }

    #[test]
    fn global_ranked_and_motion_query_sheets_keep_their_style() {
        let (registry, rules) = attached();
        registry.acquire(":root{--x:1px;}", CssLayer::Framework, SheetRank::Component);
        registry.acquire(
            Stylesheet::from(&sx().padding("lg")),
            CssLayer::Framework,
            SheetRank::Default,
        );
        registry.acquire(
            Stylesheet::from(&sx().media("(prefers-reduced-motion: reduce)", sx().padding("lg"))),
            CssLayer::Framework,
            SheetRank::Component,
        );

        assert!(block(&rules, CssLayer::Framework).is_empty());
        assert_eq!(registry.len(), 3);
        assert!(registry.take_outlet_change());
    }

    #[test]
    fn a_media_sheet_goes_in_nested_in_its_own_order() {
        let (registry, rules) = attached();
        let stylesheet = Stylesheet::from(
            &sx()
                .padding("lg")
                .media("(forced-colors: active)", sx().color("red")),
        );
        let class = stylesheet.class_name().unwrap().to_string();

        registry.acquire(stylesheet, CssLayer::UserStatic, SheetRank::Component);

        assert!(registry.stylesheets(SheetRank::Component).is_empty());
        let block = block(&rules, CssLayer::UserStatic);
        assert_eq!(block.len(), 2);
        assert!(block[0].contains("padding"));
        assert_eq!(
            block[1],
            format!(".{class}{{@media (forced-colors: active){{color:red;}}}}")
        );
    }

    /// Without nesting, and for any shape nesting cannot carry, the sheet keeps its `<style>`.
    #[test]
    fn block_rules_nest_only_the_simple_shape() {
        let nested = block_rules(".a{x:1}@container c (min-width: 1px){.a > b{y:2}}", true);
        assert_eq!(
            nested.unwrap(),
            [".a{x:1}", ".a > b{@container c (min-width: 1px){y:2}}"]
        );
        assert!(block_rules("@media x{.a{y:2}}", false).is_none());
        assert!(block_rules("@keyframes k{to{opacity:0}}", true).is_none());
        assert!(block_rules("@media x{.a::before{y:2}}", true).is_none());
        assert!(block_rules("@media x{@supports y{.a{z:3}}}", true).is_none());
        assert!(block_rules("@media (prefers-reduced-motion: reduce){.a{y:2}}", true).is_none());
        assert_eq!(block_rules(".a{x:1}", false).unwrap(), [".a{x:1}"]);
    }

    /// Eviction finds the sheet's rules by the ones inserted before it, whatever came after.
    #[test]
    fn an_evicted_sheet_deletes_exactly_its_own_rules() {
        let (registry, rules) = attached();
        let keys: Vec<_> = (0..=RETAINED).map(|px| padding(&registry, px)).collect();
        assert_eq!(block(&rules, CssLayer::UserCustom).len(), RETAINED + 1);
        for &key in &keys {
            registry.release(key);
        }

        let left = block(&rules, CssLayer::UserCustom);
        assert_eq!(left.len(), RETAINED);
        let first = Stylesheet::from(&sx().padding("0px"));
        let second = Stylesheet::from(&sx().padding("1px"));
        assert!(!left.iter().any(|rule| rule == first.as_str()));
        assert_eq!(left[0], second.as_str());
    }

    #[test]
    fn a_rule_the_browser_rejects_is_dropped_and_the_rest_counted() {
        let rules = FakeRules {
            reject: Some(":hover"),
            ..FakeRules::default()
        };
        let registry = StylesheetRegistry::new();
        registry.attach(Box::new(rules.clone()));
        let stylesheet = Stylesheet::from(&sx().padding("lg").hover(sx().color("red")));
        let key = registry.acquire(stylesheet, CssLayer::UserCustom, SheetRank::Component);
        assert_eq!(
            block(&rules, CssLayer::UserCustom).len(),
            1,
            "the hover rule rejected"
        );
        padding(&registry, 3);

        registry.inner.borrow_mut().remove(&key);

        assert_eq!(
            block(&rules, CssLayer::UserCustom),
            [Stylesheet::from(&sx().padding("3px")).as_str()]
        );
    }
}
