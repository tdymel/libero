//! Rules edited in place inside a mounted `<style>`'s top-level blocks (todo 2186).

use std::rc::Rc;

use dioxus::prelude::MountedData;

use super::backend;

/// A mounted `<style>` whose top-level rules are grouping blocks (`@layer x{}`).
pub(crate) trait StyleRulesApi {
    /// Inserts `rule` at `index` in block `block`; `false` when the browser rejects it.
    fn insert(&self, block: usize, index: usize, rule: &str) -> bool;
    fn delete(&self, block: usize, index: usize);
    /// Whether the browser takes nested rules (`.a{@media x{..}}`).
    fn nests(&self) -> bool;
}

/// The blocks of `style`; `None` off the web, where every sheet stays its own `<style>`.
pub(crate) fn style_rules(style: &Rc<MountedData>) -> Option<Box<dyn StyleRulesApi>> {
    backend::style_rules(style)
}

/// Whether this renderer can edit a mounted sheet's rules.
pub(crate) const fn edits_style_rules() -> bool {
    cfg!(target_arch = "wasm32")
}
