use std::{cell::RefCell, rc::Rc};

use dioxus::{
    core::{Runtime, ScopeId, current_scope_id},
    prelude::*,
};

use crate::{
    components::form::{FormScope, SummaryItem},
    hooks::ElementHandle,
    platform::{ElementApi, PlatformError},
};

/// Controls a [`Form`](crate::components::Form) from code: reset it, validate
/// it, submit it. Make one with [`use_form`] and pass it as the form's `form`
/// prop to reach it from outside; anything inside a form gets it from
/// [`use_form_context`].
#[derive(Clone, Copy, PartialEq)]
pub struct FormHandle {
    pub(crate) scope: FormScope,
    /// The `<form>` element. Created with the handle rather than by the form,
    /// so a parent holding the handle uses a signal it owns - dioxus warns
    /// when a scope uses a signal its child created.
    pub(crate) element: ElementHandle,
    /// Bumped when the summary appears; the form's effect focuses it. Owned
    /// here for the same reason as `element`.
    pub(crate) focus_requests: Signal<u32>,
    /// What only the `Form` itself has - its value's type and its summary.
    /// Filled in when the form mounts.
    control: CopyValue<Option<Control>>,
}

#[derive(Clone)]
pub(crate) struct Control {
    /// Puts the form's value back to `V::default()`, where `V` is still known.
    pub reset_value: Rc<dyn Fn()>,
    pub summary: Summary,
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

    /// Checks the form the way a submit does, without `onsubmit`: every status
    /// shows, and with any error the summary appears and takes focus. `true`
    /// when nothing is an error - warnings never count.
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

    /// Submits the form as its submit button would: validation, then
    /// `onsubmit`. Only the web can fire a submit from code; elsewhere this is
    /// [`PlatformError::Unsupported`] and nothing happens.
    pub fn submit(&self) -> Result<(), PlatformError> {
        self.element.request_submit()
    }

    /// Back to the start: the form's value to its default, nothing touched,
    /// not submitted, no summary. On the web the controls not bound to the
    /// value reset too, the way a native reset does.
    pub fn reset(&self) {
        // Before the value, so a bound control ends on the value's default
        // rather than on its markup default.
        let _ = self.element.reset();
        if let Some(control) = self.control() {
            (control.reset_value)();
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

/// The error summary a failed submit leaves. Lines drop out as their errors
/// are fixed; none is added until the next submit, so the list is not
/// re-announced while someone works through it.
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

    /// The lines still failing. Only while there are lines does it subscribe
    /// the form to the fields' statuses, so a form with no summary does not
    /// re-render when a field's status changes.
    pub fn visible(&self, scope: &FormScope) -> Vec<SummaryItem> {
        let mut items = self.items.borrow_mut();
        if items.is_empty() {
            return Vec::new();
        }
        scope.has_errors_tracked();
        let current = scope.summary();
        items.retain(|item| current.contains(item));
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
