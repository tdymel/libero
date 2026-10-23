use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{components::common::Variant, sx::ThemeAwareValue, theme::Size};

/// A `ButtonGroup`'s defaults for the buttons inside it; a button's own prop wins.
#[derive(Clone, Default, PartialEq)]
pub(crate) struct ButtonGroupContext {
    pub size: Option<Size>,
    pub radius: Option<ThemeAwareValue>,
    pub variant: Option<Variant>,
    pub color: Option<ThemeAwareValue>,
    pub disabled: Option<bool>,
    pub focusable_when_disabled: Option<bool>,
    pub value: Option<HashSet<String>>,
    pub multiple: bool,
    pub onchange: Option<EventHandler<HashSet<String>>>,
}

impl ButtonGroupContext {
    /// Whether a button with this `value` is pressed; `None` when the group selects nothing or the button has no value.
    pub fn pressed(&self, value: Option<&str>) -> Option<bool> {
        let (selected, value) = self.value.as_ref().zip(value)?;
        Some(selected.contains(value))
    }

    /// Asks the group for `value` pressed or released; a single group swaps its one.
    pub fn toggle(&self, value: &str) {
        let (Some(selected), Some(onchange)) = (&self.value, &self.onchange) else {
            return;
        };
        let mut next = selected.clone();
        if !next.remove(value) {
            if !self.multiple {
                next.clear();
            }
            next.insert(value.to_string());
        }
        onchange.call(next);
    }
}

// A signal: a group re-rendered with new props must reach buttons its parent memoized.
#[derive(Clone, Copy)]
struct ButtonGroupScope(Signal<ButtonGroupContext>);

/// Hands `defaults` to the buttons below, updating them when they change.
pub(crate) fn use_provide_button_group(defaults: ButtonGroupContext) {
    let mut scope = use_context_provider(|| ButtonGroupScope(Signal::new(defaults.clone())));
    if *scope.0.peek() != defaults {
        scope.0.set(defaults);
    }
}

/// The enclosing group's defaults, all unset outside one.
pub(crate) fn use_button_group() -> ButtonGroupContext {
    let scope = use_hook(try_consume_context::<ButtonGroupScope>);
    scope
        .map(|scope| scope.0.read().clone())
        .unwrap_or_default()
}
