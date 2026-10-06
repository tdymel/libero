use std::{cell::RefCell, rc::Rc};

use dioxus::{
    core::{Runtime, ScopeId, current_scope_id},
    prelude::*,
};

use crate::{
    components::form::{FormScope, SummaryItem},
    hooks::ElementHandle,
    platform::{self, ElementApi, PlatformError},
};

/// Resets, validates or submits a [`Form`](crate::components::Form) from code.
/// Made with [`use_form`], or reached from inside with [`use_form_context`].
#[derive(Clone, Copy, PartialEq)]
pub struct FormHandle {
    pub(crate) scope: FormScope,
    /// Created with the handle, so a parent holding it owns the signal.
    pub(crate) element: ElementHandle,
    /// Bumped when the summary appears; the form's effect focuses it.
    pub(crate) focus_requests: Signal<u32>,
    /// Filled in when the form mounts.
    control: CopyValue<Option<Control>>,
}

#[derive(Clone)]
pub(crate) struct Control {
    /// Puts the form's value back to `V::default()`, where `V` is still known.
    pub reset_value: Rc<dyn Fn()>,
    pub summary: Summary,
    /// The form's submit handler: validation, then `onsubmit`.
    pub submit: Callback<FormEvent>,
}

impl FormHandle {
    /// Every signal in here belongs to the scope that calls this.
    pub(crate) fn new() -> Self {
        Self {
            scope: FormScope::new(),
            element: ElementHandle::new(),
            focus_requests: Signal::new(0),
            control: CopyValue::new(None),
        }
    }

    pub(crate) fn attach(&self, control: Control) {
        let mut slot = self.control;
        slot.set(Some(control));
    }

    /// Called on the form's drop, as the scope that owns the handle.
    pub(crate) fn detach(&self) {
        let mut slot = self.control;
        self.scope.in_owner(|| {
            if slot.try_peek().is_ok_and(|control| control.is_some()) {
                slot.set(None);
            }
        });
    }

    fn control(&self) -> Option<Control> {
        self.control
            .try_peek()
            .ok()
            .and_then(|control| control.clone())
    }

    /// Checks the form as a submit does, without `onsubmit`. `true` when
    /// nothing is an error; warnings never count. When invalid it shows the
    /// summary and moves focus to it; [`is_valid`](Self::is_valid) does neither.
    pub fn validate(&self) -> bool {
        let mut scope = self.scope;
        scope.submit();
        scope.warn_unknown_paths();
        let valid = !scope.has_errors();
        if let Some(control) = self.control() {
            control.summary.set(match valid {
                true => Vec::new(),
                false => scope.summary(),
            });
        }
        valid
    }

    /// Submits the form as its submit button would. [`PlatformError::Unsupported`]
    /// only once the form is gone.
    pub fn submit(&self) -> Result<(), PlatformError> {
        match self.element.request_submit() {
            Err(PlatformError::Unsupported) => {
                let control = self.control().ok_or(PlatformError::Unsupported)?;
                control
                    .submit
                    .call(platform::submit_event(self.element.mounted()));
                Ok(())
            }
            submitted => submitted,
        }
    }

    /// Back to the start: default value, nothing touched, no summary. A field
    /// with its own `value` and handler keeps what it shows.
    pub fn reset(&self) {
        // Before the value, so a bound control ends on the value's default.
        let _ = self.element.reset();
        if let Some(control) = self.control() {
            // In the form's scope: a parent holding the handle is no descendant.
            Runtime::current().in_scope(control.summary.owner, || (control.reset_value)());
            control.summary.set(Vec::new());
        }
        let mut scope = self.scope;
        scope.reset();
    }

    /// Whether nothing is an error right now, shown or not. Reactive, so it can
    /// disable a button.
    pub fn is_valid(&self) -> bool {
        !self.scope.has_errors_tracked()
    }

    /// Hides the error summary. Resets nothing.
    pub fn clear_summary(&self) {
        if let Some(control) = self.control() {
            control.summary.set(Vec::new());
        }
    }
}

/// The error summary a failed submit leaves. Fixed lines drop out, a line whose
/// field is still in error follows its message; none is
/// added until the next submit, so it is not re-announced.
#[derive(Clone)]
pub(crate) struct Summary {
    /// Outside a signal: the form reads it while rendering and trims it there.
    items: Rc<RefCell<Vec<SummaryItem>>>,
    owner: ScopeId,
    focus_requests: Signal<u32>,
}

impl Summary {
    pub fn new(focus_requests: Signal<u32>) -> Self {
        Self {
            items: Rc::default(),
            owner: current_scope_id(),
            focus_requests,
        }
    }

    /// Replaces the lines. A non-empty list takes focus.
    pub fn set(&self, items: Vec<SummaryItem>) {
        let show = !items.is_empty();
        *self.items.borrow_mut() = items;
        Runtime::current().needs_update(self.owner);
        if show {
            let mut focus_requests = self.focus_requests;
            focus_requests += 1;
        }
    }

    /// The lines still failing. Subscribes to statuses only while there are lines.
    pub fn visible(&self, scope: &FormScope) -> Vec<SummaryItem> {
        let mut items = self.items.borrow_mut();
        if items.is_empty() {
            return Vec::new();
        }
        scope.has_errors_tracked();
        // A field still in error keeps its line with the new text (todo 2556).
        let mut current = scope.summary();
        *items = items
            .iter()
            .filter_map(|item| {
                let at = current.iter().position(|line| line == item).or_else(|| {
                    item.target.as_ref()?;
                    current.iter().position(|line| line.target == item.target)
                })?;
                Some(current.remove(at))
            })
            .collect();
        items.clone()
    }
}

/// A handle to control a form from code. Pass it as the form's `form` prop.
pub fn use_form() -> FormHandle {
    use_hook(FormHandle::new)
}

/// The handle of the `Form` around this component, or `None` outside one.
pub fn use_form_context() -> Option<FormHandle> {
    use_hook(try_consume_context::<FormHandle>)
}
