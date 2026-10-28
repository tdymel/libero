//! Async submit handlers, form-wide errors and the hidden summary (todos 4097, 4099, 4100).

use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use dioxus::{core::provide_root_context, prelude::*};

use crate::{
    LiberoProvider,
    components::form::{
        FieldPath, Form, FormErrors, FormHandle, Rule, Submit, SummaryMode, TextField, use_form,
    },
};

#[derive(Clone, PartialEq, Default)]
struct Login {
    email: String,
}

fn email_path() -> FieldPath<Login, String> {
    crate::path!(Login => email)
}

fn expose<T: Clone + 'static>(value: T) {
    use_hook(|| provide_root_context(value));
}

fn exposed<T: Clone + 'static>(dom: &VirtualDom) -> T {
    dom.in_scope(ScopeId::ROOT, consume_context::<T>)
}

fn settle(dom: &mut VirtualDom) -> String {
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(dom)
}

/// A request the test answers: pending until `answer`.
#[derive(Clone, Default)]
struct Gate {
    answer: Rc<RefCell<Option<Result<(), FormErrors>>>>,
    waker: Rc<RefCell<Option<Waker>>>,
    calls: Rc<Cell<u32>>,
}

impl Gate {
    fn answer(&self, result: Result<(), FormErrors>) {
        *self.answer.borrow_mut() = Some(result);
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    fn request(&self) -> Pin<Box<dyn Future<Output = Result<(), FormErrors>>>> {
        self.calls.set(self.calls.get() + 1);
        let gate = self.clone();
        Box::pin(std::future::poll_fn(
            move |context: &mut Context<'_>| match gate.answer.borrow_mut().take() {
                Some(result) => Poll::Ready(result),
                None => {
                    *gate.waker.borrow_mut() = Some(context.waker().clone());
                    Poll::Pending
                }
            },
        ))
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Setup {
    mode: SummaryMode,
    /// The email field must be filled, so an empty submit is blocked.
    required: bool,
    /// A rule over the whole form that names no field.
    whole: bool,
}

#[component]
fn LoginForm(setup: Setup) -> Element {
    let login = use_store(Login::default);
    let form = use_form();
    let gate = use_hook(Gate::default);
    expose(form);
    expose(gate.clone());
    let validate = match setup.whole {
        true => vec![(|_: &Login| false).error("Try again later.")],
        false => Vec::new(),
    };
    rsx! {
        Form { form, value: login, summary: setup.mode, validate,
            onsubmit: Submit::task(move |_| gate.request()),
            TextField { label: "Email", required: setup.required, name: email_path() }
        }
    }
}

fn mount_with(setup: Setup) -> VirtualDom {
    let mut dom = VirtualDom::new_with_props(
        |setup: Setup| rsx! { LiberoProvider { LoginForm { setup } } },
        setup,
    );
    dom.rebuild_in_place();
    settle(&mut dom);
    dom
}

fn mount(mode: SummaryMode) -> VirtualDom {
    mount_with(Setup {
        mode,
        required: false,
        whole: false,
    })
}

fn blocked(mode: SummaryMode) -> (VirtualDom, String) {
    let mut dom = mount_with(Setup {
        mode,
        required: true,
        whole: false,
    });
    let html = submit(&mut dom);
    (dom, html)
}

#[test]
fn a_visible_summary_lists_a_blocked_submit_without_a_count() {
    let (_, html) = blocked(SummaryMode::Visible);
    assert!(html.contains(r#"data-slot="summary""#), "{html}");
    assert!(!html.contains("error in the form"), "{html}");
}

#[test]
fn a_hidden_summary_shows_no_list_and_counts_the_errors() {
    let (dom, html) = blocked(SummaryMode::Hidden);
    let gate = exposed::<Gate>(&dom);
    assert_eq!(gate.calls.get(), 0, "a blocked submit ran the handler");
    assert!(!html.contains(r#"data-slot="summary""#), "{html}");
    assert!(
        html.contains("Fill in this field."),
        "the field shows its error: {html}"
    );
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains("1 error in the form."), "{html}");
}

#[test]
fn a_hidden_summary_still_shows_a_form_wide_error() {
    let mut dom = mount(SummaryMode::Hidden);
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(FormErrors::new().form("Wrong email or password.")));
    let html = settle(&mut dom);
    assert!(html.contains(r#"data-slot="summary""#), "{html}");
    assert!(html.contains("Wrong email or password."), "{html}");
    assert!(
        !html.contains("error in the form"),
        "announced twice: {html}"
    );
}

#[test]
fn a_hidden_summary_still_shows_a_form_rule_that_names_no_field() {
    let mut dom = mount_with(Setup {
        mode: SummaryMode::Hidden,
        required: false,
        whole: true,
    });
    let html = submit(&mut dom);
    assert!(html.contains(r#"data-slot="summary""#), "{html}");
    assert!(html.contains("Try again later."), "{html}");
}

#[test]
fn a_form_wide_error_has_an_unlinked_line_no_edit_clears() {
    let mut dom = mount(SummaryMode::Visible);
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(FormErrors::new().form("Wrong email or password.")));
    let html = settle(&mut dom);
    assert!(html.contains("Wrong email or password."), "{html}");
    assert!(
        !html.contains(r##"href="#"##),
        "the line links somewhere: {html}"
    );
    assert!(
        !html.contains(r#"aria-invalid="true""#),
        "a field took it: {html}"
    );
}

fn submit(dom: &mut VirtualDom) -> String {
    let form = exposed::<FormHandle>(dom);
    dom.in_runtime(|| form.submit().expect("the form is mounted"));
    settle(dom)
}

#[test]
fn an_async_handler_marks_the_form_submitting_until_it_ends() {
    let mut dom = mount(SummaryMode::Visible);
    let form = exposed::<FormHandle>(&dom);
    let gate = exposed::<Gate>(&dom);
    assert!(!dom.in_runtime(|| form.is_submitting()));
    let html = submit(&mut dom);
    assert_eq!(gate.calls.get(), 1);
    assert!(dom.in_runtime(|| form.is_submitting()));
    assert!(html.contains(r#"aria-busy="true""#), "{html}");

    submit(&mut dom);
    assert_eq!(
        gate.calls.get(),
        1,
        "a submit while running ran the handler again"
    );

    gate.answer(Ok(()));
    let html = settle(&mut dom);
    assert!(!dom.in_runtime(|| form.is_submitting()));
    assert!(!html.contains(r#"aria-busy="true""#), "{html}");
    submit(&mut dom);
    assert_eq!(gate.calls.get(), 2, "the next submit runs again");
}

#[test]
fn an_async_error_lands_as_set_errors() {
    let mut dom = mount(SummaryMode::Visible);
    let gate = exposed::<Gate>(&dom);
    submit(&mut dom);
    gate.answer(Err(FormErrors::new().form("Wrong email or password.")));
    let html = settle(&mut dom);
    assert!(html.contains(r#"data-slot="summary""#), "{html}");
    assert!(html.contains("Wrong email or password."), "{html}");
    let form = exposed::<FormHandle>(&dom);
    assert!(!dom.in_runtime(|| form.is_submitting()));
}

/// Todo 4345: the refusal belongs to the values before the reset.
#[test]
fn an_async_error_after_a_reset_is_dropped() {
    let mut dom = mount(SummaryMode::Visible);
    let form = exposed::<FormHandle>(&dom);
    let gate = exposed::<Gate>(&dom);
    submit(&mut dom);
    dom.in_runtime(|| form.reset());
    settle(&mut dom);
    gate.answer(Err(FormErrors::new().form("Wrong email or password.")));
    let html = settle(&mut dom);
    assert!(!html.contains("Wrong email or password."), "{html}");
    assert!(!dom.in_runtime(|| form.is_submitting()));
}
