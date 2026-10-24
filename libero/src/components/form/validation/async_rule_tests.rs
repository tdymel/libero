//! Async field rules (todo 4087): the debounce, stale answers, the pending status and the
//! submit that waits. Runs on the manual clock; the test answers each request by hand.

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
        FieldPath, Form, FormHandle, FormScope, TextField, checked_async, use_form,
    },
    hooks::polling_tests::elapse,
    localization::Localization,
    platform::manual_timer,
};

#[derive(Clone, PartialEq, Default)]
struct Signup {
    email: Option<String>,
}

#[derive(Debug)]
enum EmailError {
    Invalid,
    Taken,
}

fn email(text: String) -> Result<String, EmailError> {
    match text.contains('@') {
        true => Ok(text),
        false => Err(EmailError::Invalid),
    }
}

fn email_message(error: &EmailError, _: &Localization) -> String {
    match error {
        EmailError::Invalid => "Not an email address.".into(),
        EmailError::Taken => "This address is taken.".into(),
    }
}

/// Not the field's error type, so it needs its own words.
struct Busy;

fn busy_message(_: &Busy, _: &Localization) -> String {
    "Try again later.".into()
}

fn email_path() -> FieldPath<Signup, Option<String>> {
    crate::path!(Signup => email)
}

/// One request to the server, open until the test answers it.
#[derive(Clone, Default)]
struct Call {
    email: String,
    answer: Rc<RefCell<Option<Result<(), EmailError>>>>,
    waker: Rc<RefCell<Option<Waker>>>,
    /// The rule dropped the request before its answer.
    dropped: Rc<Cell<bool>>,
}

struct Asking(Call);

impl Future for Asking {
    type Output = Result<(), EmailError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.answer.borrow_mut().take() {
            Some(result) => Poll::Ready(result),
            None => {
                *self.0.waker.borrow_mut() = Some(context.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl Drop for Asking {
    fn drop(&mut self) {
        if self.0.waker.borrow().is_some() {
            self.0.dropped.set(true);
        }
    }
}

#[derive(Clone, Default)]
struct Server {
    calls: Rc<RefCell<Vec<Call>>>,
}

impl Server {
    fn ask(&self, email: String) -> Asking {
        let call = Call {
            email,
            ..Call::default()
        };
        self.calls.borrow_mut().push(call.clone());
        Asking(call)
    }

    fn answer(&self, index: usize, result: Result<(), EmailError>) {
        let call = self.calls.borrow()[index].clone();
        *call.answer.borrow_mut() = Some(result);
        if let Some(waker) = call.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    fn asked(&self) -> Vec<String> {
        self.calls
            .borrow()
            .iter()
            .map(|call| call.email.clone())
            .collect()
    }

    fn dropped(&self, index: usize) -> bool {
        self.calls.borrow()[index].dropped.get()
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
struct Setup {
    warning: bool,
    /// The rule fails with its own error type and words.
    foreign: bool,
    /// The `Form`'s `pending_loader`.
    loader: bool,
}

#[derive(Clone, Default)]
struct Sent(Rc<Cell<u32>>);

fn expose<T: Clone + 'static>(value: T) {
    use_hook(|| provide_root_context(value));
}

fn exposed<T: Clone + 'static>(dom: &VirtualDom) -> T {
    dom.in_scope(ScopeId::ROOT, consume_context::<T>)
}

#[component]
fn SignupForm(setup: Setup) -> Element {
    let signup = use_store(Signup::default);
    let form = use_form();
    let server = use_hook(Server::default);
    let sent = use_hook(Sent::default);
    let shown = use_signal(|| true);
    expose(signup);
    expose(form);
    expose(server.clone());
    expose(sent.clone());
    expose(shown);
    let mapped = email_path()
        .map(String::clone, email)
        .message(email_message);
    let mapped = match (setup.foreign, setup.warning) {
        (true, _) => mapped.rules([checked_async(move |text: String| {
            let asking = server.ask(text);
            async move { asking.await.map_err(|_| Busy) }
        })
        .message(busy_message)]),
        (false, true) => {
            mapped.rules([checked_async(move |text: String| server.ask(text)).warning()])
        }
        (false, false) => mapped.rules([checked_async(move |text: String| server.ask(text))]),
    };
    rsx! {
        Form { form, value: signup, pending_loader: setup.loader,
            onsubmit: move |_| sent.0.set(sent.0.get() + 1),
            if shown() {
                TextField { label: "Email", name: mapped }
            }
        }
    }
}

fn mount_with(setup: Setup) -> VirtualDom {
    let mut dom = VirtualDom::new_with_props(
        |setup: Setup| rsx! { LiberoProvider { SignupForm { setup } } },
        setup,
    );
    dom.rebuild_in_place();
    settle(&mut dom);
    dom
}

fn mount() -> VirtualDom {
    mount_with(Setup::default())
}

fn settle(dom: &mut VirtualDom) -> String {
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(dom)
}

/// Stores `text` as if typed and parsed.
fn type_text(dom: &mut VirtualDom, text: &str) -> String {
    let mut signup = exposed::<Store<Signup>>(dom);
    dom.in_runtime(|| signup.write().email = Some(text.to_string()));
    settle(dom)
}

fn submit(dom: &mut VirtualDom) -> String {
    let form = exposed::<FormHandle>(dom);
    dom.in_runtime(|| form.submit().expect("the form is mounted"));
    settle(dom)
}

fn scope(dom: &VirtualDom) -> FormScope {
    exposed::<FormHandle>(dom).scope
}

fn checking(dom: &VirtualDom) -> bool {
    dom.in_runtime(|| scope(dom).checking())
}

fn has_errors(dom: &VirtualDom) -> bool {
    dom.in_runtime(|| scope(dom).has_errors())
}

fn sent(dom: &VirtualDom) -> u32 {
    exposed::<Sent>(dom).0.get()
}

#[test]
fn edits_inside_the_debounce_ask_once_for_the_last_value() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    elapse(&mut dom, 100);
    type_text(&mut dom, "ab@x");
    elapse(&mut dom, 100);
    type_text(&mut dom, "abc@x");
    assert!(server.asked().is_empty(), "asked before the debounce");
    assert!(checking(&dom));
    elapse(&mut dom, 300);
    assert_eq!(server.asked(), ["abc@x"]);
}

#[test]
fn an_answer_for_an_older_value_is_dropped() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    type_text(&mut dom, "b@x");
    assert!(server.dropped(0), "the older request still runs");
    server.answer(0, Err(EmailError::Taken));
    let html = settle(&mut dom);
    assert!(!has_errors(&dom), "the older answer landed");
    assert!(!html.contains("This address is taken."), "{html}");

    elapse(&mut dom, 300);
    assert_eq!(server.asked(), ["a@x", "b@x"]);
    server.answer(1, Ok(()));
    settle(&mut dom);
    assert!(!checking(&dom));
    assert!(!has_errors(&dom));
}

#[test]
fn the_answer_is_kept_for_the_value_it_judged() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    server.answer(0, Err(EmailError::Taken));
    settle(&mut dom);
    assert!(has_errors(&dom), "the answer blocks no submit");
    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    assert_eq!(server.asked().len(), 1, "the same value asked again");
}

#[test]
fn a_submit_waits_for_the_answer_then_sends() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    let form = exposed::<FormHandle>(&dom);
    type_text(&mut dom, "a@x");
    let html = submit(&mut dom);
    assert_eq!(
        server.asked(),
        ["a@x"],
        "the submit did not end the debounce"
    );
    assert_eq!(sent(&dom), 0, "sent before the answer");
    assert!(dom.in_runtime(|| form.is_submitting()));
    // The form's for the submit under way, the input's for its rule.
    assert_eq!(html.matches(r#"aria-busy="true""#).count(), 2, "{html}");
    assert!(html.contains(r#"radius-sm pending""#), "{html}");

    server.answer(0, Ok(()));
    settle(&mut dom);
    assert_eq!(sent(&dom), 1);
    assert!(!dom.in_runtime(|| form.is_submitting()));
}

#[test]
fn a_submit_waiting_on_a_taken_value_shows_the_answer() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    submit(&mut dom);
    server.answer(0, Err(EmailError::Taken));
    let html = settle(&mut dom);
    assert_eq!(sent(&dom), 0, "a taken address was sent");
    assert!(html.contains("This address is taken."), "{html}");
    assert!(html.contains(r#"data-slot="summary""#), "{html}");
}

#[test]
fn a_warning_rule_holds_no_submit() {
    let _clock = manual_timer::install();
    let mut dom = mount_with(Setup {
        warning: true,
        ..Setup::default()
    });
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    submit(&mut dom);
    assert_eq!(sent(&dom), 1, "the submit waited for a warning");
    elapse(&mut dom, 300);
    server.answer(0, Err(EmailError::Taken));
    let html = settle(&mut dom);
    assert!(!has_errors(&dom));
    assert!(html.contains("This address is taken."), "{html}");
}

#[test]
fn a_rule_with_its_own_words_shows_them() {
    let _clock = manual_timer::install();
    let mut dom = mount_with(Setup {
        foreign: true,
        ..Setup::default()
    });
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    submit(&mut dom);
    server.answer(0, Err(EmailError::Taken));
    let html = settle(&mut dom);
    assert!(html.contains("Try again later."), "{html}");
}

#[test]
fn a_reset_drops_the_request_and_its_answer() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    let form = exposed::<FormHandle>(&dom);
    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    dom.in_runtime(|| form.reset());
    settle(&mut dom);
    assert!(server.dropped(0), "the request outlived the reset");
    assert!(!checking(&dom));

    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    assert_eq!(server.asked().len(), 2, "the reset kept the value's answer");
}

#[test]
fn an_unmount_ends_the_request() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    let mut shown = exposed::<Signal<bool>>(&dom);
    dom.in_runtime(|| shown.set(false));
    settle(&mut dom);
    assert!(server.dropped(0), "the request outlived the field");
    assert!(!checking(&dom));
}

/// Todo 4117: going back to an earlier value takes its answer without asking again.
#[test]
fn an_earlier_value_takes_its_kept_answer() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    server.answer(0, Err(EmailError::Taken));
    settle(&mut dom);
    type_text(&mut dom, "b@x");
    elapse(&mut dom, 300);
    server.answer(1, Ok(()));
    settle(&mut dom);
    assert!(!has_errors(&dom));

    type_text(&mut dom, "a@x");
    assert!(!checking(&dom), "the earlier value is asked again");
    assert!(has_errors(&dom), "the earlier value lost its answer");
    type_text(&mut dom, "b@x");
    elapse(&mut dom, 300);
    assert!(!has_errors(&dom));
    assert_eq!(server.asked(), ["a@x", "b@x"]);
}

/// Todo 4117: a reset forgets every kept answer, not only the last.
#[test]
fn a_reset_forgets_the_kept_answers() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    let server = exposed::<Server>(&dom);
    let form = exposed::<FormHandle>(&dom);
    for (index, text) in ["a@x", "b@x"].into_iter().enumerate() {
        type_text(&mut dom, text);
        elapse(&mut dom, 300);
        server.answer(index, Ok(()));
        settle(&mut dom);
    }
    dom.in_runtime(|| form.reset());
    settle(&mut dom);

    type_text(&mut dom, "a@x");
    elapse(&mut dom, 300);
    assert_eq!(
        server.asked(),
        ["a@x", "b@x", "a@x"],
        "the reset kept an answer"
    );
}

/// Todo 4116: the loader shows only while the revealed status is pending, and only opted in.
#[test]
fn the_pending_loader_shows_only_while_the_rule_asks() {
    let _clock = manual_timer::install();
    let mut dom = mount_with(Setup {
        loader: true,
        ..Setup::default()
    });
    let server = exposed::<Server>(&dom);
    let html = type_text(&mut dom, "a@x");
    assert!(
        !html.contains("data-pending=true"),
        "shown before a blur or submit: {html}"
    );
    let html = submit(&mut dom);
    assert_eq!(html.matches("data-pending=true").count(), 1, "{html}");
    server.answer(0, Ok(()));
    let html = settle(&mut dom);
    assert!(!html.contains("data-pending=true"), "{html}");
}

#[test]
fn no_pending_loader_by_default() {
    let _clock = manual_timer::install();
    let mut dom = mount();
    type_text(&mut dom, "a@x");
    let html = submit(&mut dom);
    assert!(html.contains(r#"aria-busy="true""#), "{html}");
    assert!(!html.contains("data-pending=true"), "{html}");
}
