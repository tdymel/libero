use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::use_hook;

use crate::{
    components::{ClassList, Input, States, Variables},
    hooks::{SxSource, use_box_css},
    sx::{StaticSx, Sx},
};

/// The class / `data-state` / `style` triple the styling props resolve to.
#[derive(Clone)]
pub(crate) struct StyleAttributes {
    pub class: String,
    pub data_state: Option<String>,
    pub style: Option<String>,
}

/// Resolves the styling props every `Box`-shaped component shares.
///
/// Calls [`use_box_css`], so it is a hook: call it before any early return.
pub(crate) fn use_style_attributes(
    class: &Input<ClassList>,
    framework_sx: Option<&'static StaticSx>,
    sx: &Input<Sx>,
    states: &Input<States>,
    variables: &Input<Variables>,
    style: Option<String>,
    focus_ring: bool,
) -> StyleAttributes {
    // Not `sx.as_ref()`: that collapses the lifetime, so a caller's `static`
    // is rehashed and rebuilt as if it were freshly built this render.
    let sx_source = match sx {
        Input::None => None,
        Input::Value(sx) => Some(SxSource::Owned(sx)),
        Input::Static(sx) => Some(SxSource::Static(sx)),
    };
    let written = use_hook(|| Rc::new(RefCell::new(Vec::<String>::new())));
    let class = use_box_css(
        class,
        focus_ring.then_some(&BOX_FOCUS_SX),
        framework_sx,
        sx_source,
    );

    let variables_style = variables
        .as_ref()
        .map(Variables::render)
        .filter(|style| !style.is_empty());

    StyleAttributes {
        class,
        data_state: states.as_ref().and_then(States::data_state),
        style: keep_dropped_vars(
            match (variables_style, style) {
                (Some(variables), Some(raw)) => Some(format!("{variables}{raw}")),
                (Some(style), None) | (None, Some(style)) => Some(style),
                (None, None) => None,
            },
            &mut written.borrow_mut(),
        ),
    }
}

// Shared rather than living in `Box`: a plain div only reaches this once it
// has a tabindex, but an `<a href>` is focusable on its own.
static BOX_FOCUS_SX: StaticSx =
    StaticSx::new(|| crate::sx::sx().focus_visible(super::focus_ring_sx()));

/// A custom property's value that computes as if it were absent - see
/// [`keep_dropped_vars`]. For a `style` the revert cannot see, one set with
/// `.attr("style", ..)`: write this rather than leave the var out.
pub(crate) const ABSENT: &str = "revert-layer";

/// Writes `--x:revert-layer` for every custom property this element had
/// before and `style` no longer declares.
///
/// When the dioxus interpreter sets `style`, it puts back every property the
/// new string omits, so a var dropped from one render to the next would keep
/// its last value. `revert-layer` is what "absent" computes to: measured in
/// Chromium, the element still takes a class rule's value, a layered one, its
/// parent's, and `var(--x, fallback)`'s fallback. `initial` would lose all
/// but the last. `written` is every name this element has ever declared.
fn keep_dropped_vars(style: Option<String>, written: &mut Vec<String>) -> Option<String> {
    if written.is_empty() && style.is_none() {
        return None;
    }
    let mut style = style.unwrap_or_default();
    // No allocation once every name has been seen: both passes rescan `style`.
    for name in custom_properties(&style) {
        if !written.iter().any(|seen| seen == name) {
            written.push(name.to_string());
        }
    }
    let dropped: String = written
        .iter()
        .filter(|seen| !custom_properties(&style).any(|name| name == seen.as_str()))
        .map(|name| format!("{name}:{ABSENT};"))
        .collect();
    style.push_str(&dropped);
    (!style.is_empty()).then_some(style)
}

/// The custom property names `style` declares.
fn custom_properties(style: &str) -> impl Iterator<Item = &str> {
    style
        .split(';')
        .filter_map(|declaration| declaration.split_once(':'))
        .map(|(name, _)| name.trim())
        .filter(|name| name.starts_with("--"))
}

#[cfg(test)]
mod tests {
    use super::keep_dropped_vars;

    #[test]
    fn a_dropped_var_is_reverted_and_a_returning_one_is_not() {
        let mut written = Vec::new();

        let first = keep_dropped_vars(Some("--a:red;--b:1px;".into()), &mut written);
        assert_eq!(first.as_deref(), Some("--a:red;--b:1px;"));

        let shrunk = keep_dropped_vars(Some("--b:2px;".into()), &mut written);
        assert_eq!(shrunk.as_deref(), Some("--b:2px;--a:revert-layer;"));

        let emptied = keep_dropped_vars(None, &mut written);
        assert_eq!(
            emptied.as_deref(),
            Some("--a:revert-layer;--b:revert-layer;")
        );

        let back = keep_dropped_vars(Some("--a:blue;".into()), &mut written);
        assert_eq!(back.as_deref(), Some("--a:blue;--b:revert-layer;"));
    }

    /// A plain property is not ours to revert, and an element that never had
    /// a var pays nothing.
    #[test]
    fn only_custom_properties_are_remembered() {
        let mut written = Vec::new();

        keep_dropped_vars(Some("left:4px;".into()), &mut written);
        assert!(written.is_empty());
        assert_eq!(keep_dropped_vars(None, &mut written), None);
    }
}
