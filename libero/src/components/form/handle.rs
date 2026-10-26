use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::{
    core::{Runtime, ScopeId, current_scope_id},
    prelude::*,
};

use crate::{
    components::form::{FormErrors, FormScope, SummaryItem},
    hooks::{ElementHandle, id_selector},
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
    /// While an async submit handler runs.
    pub(crate) submitting: Signal<bool>,
    /// Filled in when the form mounts.
    control: CopyValue<Option<Control>>,
}

#[derive(Clone)]
pub(crate) struct Control {
    /// While [`FormHandle::reset`] runs the native reset, so the form's `onreset` lets it through.
    pub resetting: Rc<Cell<bool>>,
    /// The `<form>`'s id, for a WebView, whose handle cannot reset it.
    pub id: Signal<String>,
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
            submitting: Signal::new(false),
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
        let mut submitting = self.submitting;
        self.scope.in_owner(|| {
            if slot.try_peek().is_ok_and(|control| control.is_some()) {
                slot.set(None);
            }
            // The running handler went with the form.
            if submitting.try_peek().is_ok_and(|running| *running) {
                submitting.set(false);
            }
        });
    }

    fn control(&self) -> Option<Control> {
        self.control
            .try_peek()
            .ok()
            .and_then(|control| control.clone())
    }

    /// Checks the form as a submit does, without `onsubmit`; `true` when nothing is an error.
    /// Invalid, it shows the summary and focuses it; [`is_valid`](Self::is_valid) does neither.
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
        // On the web the element outlives the form: a detached `<form>` would swallow it (4263).
        let control = self.control().ok_or(PlatformError::Unsupported)?;
        match self.element.request_submit() {
            Err(PlatformError::Unsupported) => {
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
        let control = self.control();
        if let Some(control) = &control {
            control.resetting.set(true);
        }
        // Before the value, so a bound control ends on the value's default.
        if let (Err(PlatformError::Unsupported), Some(control)) = (self.element.reset(), &control) {
            let _ = platform::reset_selector(&id_selector(&control.id.peek()));
        }
        if let Some(control) = control {
            control.resetting.set(false);
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

    /// Whether an async submit handler ([`Submit::parse_async`](crate::components::Submit::parse_async),
    /// [`Submit::task`](crate::components::Submit::task)) is running. Reactive, so it can set a
    /// button's `loading`; a submit meanwhile is ignored.
    pub fn is_submitting(&self) -> bool {
        (self.submitting)()
    }

    /// Shows each error on the field with its path, as an explicit `status` would, lists them
    /// in the summary and focuses it. Replaces the errors set before; a field's error clears
    /// on its next edit, all of them on the next submit or reset.
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// use libero::components::{Fields, Form, FormErrors, TextField, use_form};
    ///
    /// #[derive(Clone, PartialEq, Default, Fields)]
    /// struct Signup { email: String }
    ///
    /// # fn app() -> Element {
    /// let signup = use_store(Signup::default);
    /// let form = use_form();
    /// rsx! {
    ///     Form { form, value: signup,
    ///         onsubmit: move |_| {
    ///             // The server said no.
    ///             form.set_errors(FormErrors::new().at(Signup::FIELDS.email(), "This address is taken."));
    ///         },
    ///         TextField { label: "Email", name: Signup::FIELDS.email() }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn set_errors(&self, errors: FormErrors) {
        let mut scope = self.scope;
        scope.set_errors(errors.into_entries());
        if let Some(control) = self.control() {
            let summary = scope.summary();
            control.summary.set(summary);
        }
    }

    /// Drops the errors [`set_errors`](Self::set_errors) set.
    pub fn clear_errors(&self) {
        let mut scope = self.scope;
        scope.clear_errors();
    }

    /// Hides the error summary. Resets nothing.
    pub fn clear_summary(&self) {
        if let Some(control) = self.control() {
            control.summary.set(Vec::new());
        }
    }
}

/// The error summary a failed submit leaves: fixed lines drop out, lines still in error follow
/// their message, none is added before the next submit (todo 2573).
#[derive(Clone)]
pub(crate) struct Summary {
    /// Outside a signal: the form reads it while rendering and trims it there.
    items: Rc<RefCell<Vec<SummaryItem>>>,
    owner: ScopeId,
    focus_requests: Signal<u32>,
    /// The `<form>`, asked for the fields' page order.
    form: ElementHandle,
}

impl Summary {
    pub fn new(focus_requests: Signal<u32>, form: ElementHandle) -> Self {
        Self {
            items: Rc::default(),
            owner: current_scope_id(),
            focus_requests,
            form,
        }
    }

    /// Replaces the lines, in the order their fields sit on the page. A non-empty list takes focus.
    pub fn set(&self, mut items: Vec<SummaryItem>) {
        let show = !items.is_empty();
        let ids = self.page_ids(&items);
        in_page_order(&mut items, &ids);
        *self.items.borrow_mut() = items;
        Runtime::current().needs_update(self.owner);
        if show {
            let mut focus_requests = self.focus_requests;
            focus_requests += 1;
        }
    }

    /// The first field that shows its line's message itself.
    pub fn first_inline(&self) -> Option<String> {
        self.items
            .borrow()
            .iter()
            .filter(|item| item.inline)
            .find_map(|item| item.target.clone())
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

    /// The ids of the lines' fields and group labels, in DOM order; empty where the renderer
    /// cannot query (a WebView), which keeps registration order.
    fn page_ids(&self, items: &[SummaryItem]) -> Vec<String> {
        let selector = items
            .iter()
            .filter_map(|item| item.target.as_deref())
            .flat_map(|id| [id_selector(id), id_selector(&format!("{id}-label"))])
            .collect::<Vec<_>>()
            .join(", ");
        if selector.is_empty() {
            return Vec::new();
        }
        self.form
            .query_selector_all(&selector)
            .unwrap_or_default()
            .iter()
            .filter_map(|element| element.attribute("id").ok().flatten())
            .collect()
    }
}

/// Sorts the lines by where their field sits on the page (GOV.UK). A line linking to nothing
/// found goes last, in its old order.
fn in_page_order(items: &mut [SummaryItem], ids: &[String]) {
    items.sort_by_key(|item| {
        item.target
            .as_deref()
            .and_then(|target| {
                ids.iter()
                    .position(|id| id == target || id.strip_suffix("-label") == Some(target))
            })
            .unwrap_or(usize::MAX)
    });
}

/// A handle to control a form from code. Pass it as the form's `form` prop.
pub fn use_form() -> FormHandle {
    use_hook(FormHandle::new)
}

/// The handle of the `Form` around this component, or `None` outside one.
pub fn use_form_context() -> Option<FormHandle> {
    use_hook(try_consume_context::<FormHandle>)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(target: Option<&str>) -> SummaryItem {
        SummaryItem {
            message: target.unwrap_or("none").to_string(),
            target: target.map(str::to_string),
            inline: true,
        }
    }

    fn order(items: &[SummaryItem]) -> Vec<&str> {
        items.iter().map(|item| item.message.as_str()).collect()
    }

    #[test]
    fn lines_follow_the_page_and_unfound_ones_go_last() {
        let mut items = vec![
            line(Some("age")),
            line(None),
            line(Some("gone")),
            line(Some("passwords")),
            line(Some("email")),
        ];
        let ids = ["email", "passwords-label", "age"].map(str::to_string);
        in_page_order(&mut items, &ids);
        assert_eq!(order(&items), ["email", "passwords", "age", "none", "gone"]);
    }

    /// Todo 4263: with no form attached, `submit` is `Unsupported` before it tries the element.
    #[test]
    fn a_handle_without_its_form_cannot_submit() {
        use dioxus::core::NoOpMutations;

        fn app() -> Element {
            let form = use_form();
            assert!(matches!(form.submit(), Err(PlatformError::Unsupported)));
            rsx! {}
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut NoOpMutations);
    }

    #[test]
    fn no_page_keeps_registration_order() {
        let mut items = vec![line(Some("b")), line(Some("a"))];
        in_page_order(&mut items, &[]);
        assert_eq!(order(&items), ["b", "a"]);
    }
}
