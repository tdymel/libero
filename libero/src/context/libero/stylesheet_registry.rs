use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

use super::CssLayer;
use crate::{
    css::Stylesheet,
    platform::{A11yAnswers, StyleRulesApi, answer_a11y_media, current_a11y_answers},
};

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

/// `css`'s rules for a layer block, `@media P{.a{x}}` nested as `.a{@media P{x}}` and
/// `@media P{.a::before{x}}` as `.a{@media P{&::before{x}}}`: inserting a grouping rule costs
/// Chromium the whole-page restyle, a nested one 1-2 ms (todos 2231, 2249, 2287). `None` for any
/// other at-rule shape, a query `answers` rewrite, or without `nests`: the sheet keeps its `<style>`.
fn block_rules<'a>(css: &'a str, nests: bool, answers: &A11yAnswers) -> Option<Vec<Cow<'a, str>>> {
    let mut rules = Vec::new();
    for rule in top_level_rules(css) {
        if !rule.starts_with('@') {
            rules.push(Cow::Borrowed(rule));
            continue;
        }
        // An answered query (a forced reduced motion) is rewritten in the outlet's text.
        if !nests || answer_a11y_media(rule, answers) != rule {
            return None;
        }
        nest_grouping(rule, &mut Vec::new(), &mut rules)?;
    }
    Some(rules)
}

/// Pushes `rule`'s style rules nested under `preludes` plus its own; `@supports` in `@media` too.
fn nest_grouping<'a>(
    rule: &'a str,
    preludes: &mut Vec<&'a str>,
    rules: &mut Vec<Cow<'a, str>>,
) -> Option<()> {
    const GROUPING: [&str; 3] = ["@media", "@container", "@supports"];
    let (prelude, inner) = rule.split_once('{')?;
    if !GROUPING.iter().any(|name| prelude.starts_with(name)) {
        return None;
    }
    preludes.push(prelude);
    for style in top_level_rules(inner.strip_suffix('}')?) {
        if style.starts_with('@') {
            nest_grouping(style, preludes, rules)?;
            continue;
        }
        let (selector, body) = style.split_once('{')?;
        let body = body.strip_suffix('}')?;
        if body.contains('{') || selector.matches(['"', '\'']).count() % 2 == 1 {
            return None;
        }
        let open: String = preludes
            .iter()
            .map(|prelude| format!("{prelude}{{"))
            .collect();
        let close = "}".repeat(preludes.len());
        // `&` cannot stand for a list holding a pseudo-element: one rule per selector.
        let parts: Vec<&str> = if selector.contains("::") {
            selector_list(selector).collect()
        } else {
            vec![selector]
        };
        for selector in parts {
            rules.push(Cow::Owned(match pseudo_element(selector)? {
                None => format!("{selector}{{{open}{body}{close}}}"),
                Some((base, pseudo)) => format!("{base}{{{open}&{pseudo}{{{body}}}{close}}}"),
            }));
        }
    }
    preludes.pop();
    Some(())
}

/// `selector`'s top-level comma-separated parts, trimmed.
fn selector_list(selector: &str) -> impl Iterator<Item = &str> {
    let mut depth = 0i32;
    selector
        .split(move |c| {
            depth += match c {
                '[' | '(' => 1,
                ']' | ')' => -1,
                _ => 0,
            };
            depth == 0 && c == ','
        })
        .map(str::trim)
}

/// `Some(None)` for a selector with no pseudo-element, `Some(Some((base, "::x")))` for one
/// complex selector ending in a single `::x`, nested as `base{&::x{..}}`. `None` for what the
/// nesting `&` cannot carry: a list, a legacy `:before`, `::part()`, a bare `::x`.
fn pseudo_element(selector: &str) -> Option<Option<(&str, &str)>> {
    const LEGACY: [&str; 4] = [":before", ":after", ":first-line", ":first-letter"];
    let selector = selector.trim();
    let Some(at) = selector.find("::") else {
        return (!LEGACY.iter().any(|name| selector.contains(name))).then_some(None);
    };
    let (base, pseudo) = selector.split_at(at);
    let name = &pseudo[2..];
    let named = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    let legacy = LEGACY.iter().any(|name| base.contains(name));
    let ends_compound =
        !base.ends_with(|c: char| c.is_whitespace() || matches!(c, '>' | '+' | '~'));
    (named && !base.is_empty() && ends_compound && !legacy).then_some(Some((base, pseudo)))
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
    /// The outlet's accessibility answers: a query they rewrite keeps its `<style>`.
    answers: A11yAnswers,
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
        let Some(rules) = block_rules(stylesheet.as_str(), self.nests, &self.answers) else {
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
        self.delete_rules(key);
    }

    /// Takes `key`'s inserted rules out of its layer block.
    fn delete_rules(&mut self, key: &StylesheetKey) {
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
        sheets.answers = current_a11y_answers();
    }

    /// New accessibility answers: an inserted sheet whose queries they rewrite goes back to
    /// its `<style>`, which the outlet writes answered. One whole-page restyle per change.
    pub(crate) fn reanswer(&self, answers: A11yAnswers) {
        let mut sheets = self.inner.borrow_mut();
        let sheets = &mut *sheets;
        sheets.answers = answers;
        let stale: Vec<StylesheetKey> = sheets
            .by_key
            .iter()
            .filter(|(_, entry)| {
                entry.inserted && answer_a11y_media(&entry.css, &answers) != *entry.css
            })
            .map(|(key, _)| *key)
            .collect();
        for key in &stale {
            sheets.delete_rules(key);
            if let Some(entry) = sheets.by_key.get_mut(key) {
                entry.inserted = false;
            }
        }
        sheets.outlet_changed |= !stale.is_empty();
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
    fn global_and_ranked_sheets_keep_their_style() {
        let (registry, rules) = attached();
        registry.acquire(":root{--x:1px;}", CssLayer::Framework, SheetRank::Component);
        registry.acquire(
            Stylesheet::from(&sx().padding("lg")),
            CssLayer::Framework,
            SheetRank::Default,
        );

        assert!(block(&rules, CssLayer::Framework).is_empty());
        assert_eq!(registry.len(), 2);
        assert!(registry.take_outlet_change());
    }

    fn forced_motion() -> A11yAnswers {
        A11yAnswers {
            reduced_motion: Some(true),
            ..A11yAnswers::default()
        }
    }

    fn motion_sheet() -> Stylesheet {
        Stylesheet::from(
            &sx()
                .padding("lg")
                .media("(prefers-reduced-motion: reduce)", sx().padding("sm")),
        )
    }

    /// Unanswered, a motion query nests like any other (todo 2249).
    #[test]
    fn an_unanswered_motion_query_goes_in_nested() {
        let (registry, rules) = attached();
        registry.reanswer(A11yAnswers::default());
        registry.acquire(motion_sheet(), CssLayer::Framework, SheetRank::Component);

        assert!(registry.stylesheets(SheetRank::Component).is_empty());
        assert!(
            block(&rules, CssLayer::Framework)[1]
                .contains("{@media (prefers-reduced-motion: reduce){")
        );
    }

    /// A forced motion rewrites the query, so the sheet is written answered by the outlet.
    #[test]
    fn a_forced_motion_keeps_new_motion_sheets_out_and_moves_inserted_ones_back() {
        let (registry, rules) = attached();
        registry.reanswer(A11yAnswers::default());
        let moved = registry.acquire(motion_sheet(), CssLayer::Framework, SheetRank::Component);
        let plain = padding(&registry, 3);
        assert!(!registry.take_outlet_change());

        registry.reanswer(forced_motion());
        assert!(registry.take_outlet_change());
        assert_eq!(registry.stylesheets(SheetRank::Component).len(), 1);
        assert!(
            block(&rules, CssLayer::Framework).is_empty(),
            "its rules left the block"
        );
        assert_eq!(
            block(&rules, CssLayer::UserCustom).len(),
            1,
            "a plain sheet stays"
        );

        registry.release(moved);
        registry.release(plain);
        let other =
            Stylesheet::from(&sx().media("(prefers-reduced-motion: reduce)", sx().padding("xs")));
        registry.acquire(other, CssLayer::Framework, SheetRank::Component);
        assert_eq!(registry.stylesheets(SheetRank::Component).len(), 2);
        assert!(block(&rules, CssLayer::Framework).is_empty());
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
        let none = &A11yAnswers::default();
        let nested = block_rules(
            ".a{x:1}@container c (min-width: 1px){.a > b{y:2}}",
            true,
            none,
        );
        assert_eq!(
            nested.unwrap(),
            [".a{x:1}", ".a > b{@container c (min-width: 1px){y:2}}"]
        );
        assert!(block_rules("@media x{.a{y:2}}", false, none).is_none());
        assert!(block_rules("@keyframes k{to{opacity:0}}", true, none).is_none());
        assert_eq!(
            block_rules("@media x{@supports y{.a{z:3}}.b{w:4}}", true, none).unwrap(),
            [".a{@media x{@supports y{z:3}}}", ".b{@media x{w:4}}"]
        );
        assert!(block_rules("@media x{@keyframes k{to{opacity:0}}}", true, none).is_none());
        assert_eq!(block_rules(".a{x:1}", false, none).unwrap(), [".a{x:1}"]);
        let motion = "@media (prefers-reduced-motion: reduce){.a{y:2}}";
        assert!(block_rules(motion, true, none).is_some());
        assert!(block_rules(motion, true, &forced_motion()).is_none());
    }

    /// A pseudo-element moves behind the `&`, as a pseudo-element rule cannot hold an at-rule.
    #[test]
    fn a_pseudo_element_under_a_query_nests_behind_the_ampersand() {
        let none = &A11yAnswers::default();
        let css = r#"@media (prefers-reduced-motion: reduce){.a[data-state~="on"]::before{animation:none}}"#;
        assert_eq!(
            block_rules(css, true, none).unwrap(),
            [
                r#".a[data-state~="on"]{@media (prefers-reduced-motion: reduce){&::before{animation:none}}}"#
            ]
        );
        assert_eq!(
            block_rules("@media x{.a::before, .b[s='1,2']::after{y:2}}", true, none).unwrap(),
            [
                ".a{@media x{&::before{y:2}}}",
                ".b[s='1,2']{@media x{&::after{y:2}}}"
            ]
        );
        assert_eq!(
            block_rules(
                "@media x{.a > [s] ::-webkit-x, .a>.b::after{y:2}}",
                true,
                none
            ),
            None,
            "a combinator right before the pseudo-element"
        );
        assert_eq!(
            block_rules("@media x{.a > [s]::-webkit-x{y:2}}", true, none).unwrap(),
            [".a > [s]{@media x{&::-webkit-x{y:2}}}"]
        );
        for refused in [
            "@media x{.a::before, .b:after{y:2}}",
            "@media x{.a ::before{y:2}}",
            "@media x{.a:before{y:2}}",
            "@media x{.a::part(x){y:2}}",
            "@media x{::before{y:2}}",
        ] {
            assert!(block_rules(refused, true, none).is_none(), "{refused}");
        }
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
