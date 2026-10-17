use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use dioxus::document::{Eval, EvalError, Evaluator, NoOpDocument};
use dioxus::prelude::*;
use dioxus::signals::{AnyStorage, Owner, UnsyncStorage};
use serde_json::Value;

use super::*;
use crate::platform::file_dialog::held_file;

fn app() -> Element {
    rsx! {}
}

fn answers(dom: &VirtualDom) -> [bool; 5] {
    dom.in_scope(ScopeId::ROOT, || {
        [
            document().is_some(),
            scroll().is_some(),
            clipboard().is_some(),
            color_scheme().is_some(),
            keyboard().is_some(),
        ]
    })
}

#[test]
fn a_page_observes_by_tag_and_a_server_does_not() {
    let observes = |dom: &VirtualDom| dom.in_scope(ScopeId::ROOT, crate::platform::observes_by_tag);
    assert!(observes(&page_dom(false)));
    let server = VirtualDom::new(app).with_root_context(Rc::new(NoOpDocument) as Rc<dyn Document>);
    assert!(!observes(&server));
}

#[test]
fn a_slot_call_binds_its_argument_as_data() {
    let script = Slot::new().call_script("move", &serde_json::json!("x\"); alert(1); (\""));
    let (binding, call) = script.split_once('\n').unwrap();
    assert_eq!(binding, r#"const data = "x\"); alert(1); (\"";"#);
    assert!(call.ends_with("?.move(data);"), "{call}");
    assert!(!call.contains("alert"), "{call}");
}

#[test]
fn plain_ssr_without_a_document_gets_nothing() {
    assert!(document().is_none());
    assert_eq!(answers(&VirtualDom::new(app)), [false; 5]);
}

/// Fullstack's server document evaluates through `NoOpDocument`.
#[test]
fn a_server_document_gets_nothing() {
    let dom = VirtualDom::new(app).with_root_context(Rc::new(NoOpDocument) as Rc<dyn Document>);
    assert_eq!(answers(&dom), [false; 5]);
    assert!(!dom.in_scope(ScopeId::ROOT, prefers_reduced_motion));
    assert!(dom.in_scope(ScopeId::ROOT, focus_page).is_none());
    assert!(!dom.in_scope(ScopeId::ROOT, typing_target));
}

/// A page whose every script answers `dark` once.
struct FakePage {
    dark: bool,
    owners: RefCell<Vec<Owner<UnsyncStorage>>>,
}

struct Answer(Option<bool>);

impl Evaluator for Answer {
    fn poll_join(&mut self, _: &mut Context<'_>) -> Poll<Result<Value, EvalError>> {
        Poll::Pending
    }

    fn poll_recv(&mut self, _: &mut Context<'_>) -> Poll<Result<Value, EvalError>> {
        match self.0.take() {
            Some(dark) => Poll::Ready(Ok(Value::Bool(dark))),
            None => Poll::Pending,
        }
    }

    fn send(&self, _: Value) -> Result<(), EvalError> {
        Ok(())
    }
}

impl Document for FakePage {
    fn eval(&self, _: String) -> Eval {
        let owner = UnsyncStorage::owner();
        let answer = owner.insert(Box::new(Answer(Some(self.dark))) as Box<dyn Evaluator>);
        self.owners.borrow_mut().push(owner);
        Eval::new(answer)
    }
}

fn page_dom(dark: bool) -> VirtualDom {
    let page = FakePage {
        dark,
        owners: RefCell::default(),
    };
    let mut dom = VirtualDom::new(app).with_root_context(Rc::new(page) as Rc<dyn Document>);
    dom.rebuild_in_place();
    dom
}

fn settle(dom: &mut VirtualDom) {
    let mut cx = Context::from_waker(Waker::noop());
    for _ in 0..4 {
        let _ = pin!(dom.wait_for_work()).poll(&mut cx);
    }
}

/// Desktop provides its document in the window's scope; a `spawn_forever`
/// task runs at the root and must still reach it (1027).
#[test]
fn the_root_reaches_a_document_provided_below_it() {
    #[component]
    fn Window() -> Element {
        use_hook(|| {
            let page = FakePage {
                dark: true,
                owners: RefCell::default(),
            };
            provide_context(Rc::new(page) as Rc<dyn Document>);
            assert!(runs_scripts());
        });
        rsx! {}
    }
    fn windowed() -> Element {
        rsx! { Window {} }
    }
    let mut dom = VirtualDom::new(windowed);
    assert!(dom.in_scope(ScopeId::ROOT, page_document).is_none());
    dom.rebuild_in_place();
    assert!(dom.in_scope(ScopeId::ROOT, page_document).is_some());
}

/// Liveview sessions and desktop windows share a thread: one's answer must not reach another.
#[test]
fn two_pages_on_one_thread_keep_their_own_scheme() {
    let mut dark = page_dom(true);
    let mut light = page_dom(false);
    let heard = Rc::new(Cell::new(0));
    let system =
        |dom: &VirtualDom| dom.in_scope(ScopeId::ROOT, || color_scheme().unwrap().system());
    let _subscription = light.in_scope(ScopeId::ROOT, || {
        let heard = heard.clone();
        color_scheme()
            .unwrap()
            .on_change(Box::new(move |_| heard.set(heard.get() + 1)))
    });
    system(&dark);
    settle(&mut dark);
    settle(&mut light);
    assert_eq!(system(&dark), ColorScheme::Dark);
    assert_eq!(system(&light), ColorScheme::Light);
    assert_eq!(
        heard.get(),
        0,
        "the dark page's answer reached the light one"
    );
}

#[test]
fn base64_decodes_as_btoa_wrote_it() {
    assert_eq!(decode_base64("").unwrap(), b"");
    assert_eq!(decode_base64("YQ==").unwrap(), b"a");
    assert_eq!(decode_base64("YWI=").unwrap(), b"ab");
    assert_eq!(decode_base64("YWJj").unwrap(), b"abc");
    assert_eq!(decode_base64("AP+A/w==").unwrap(), [0, 255, 128, 255]);
    assert_eq!(decode_base64("YW\nJj"), None);
}

#[test]
fn a_held_file_reads_from_memory() {
    let file = held_file("a.txt".into(), "text/plain".into(), 7, b"hello".to_vec());
    assert_eq!(file.name(), "a.txt");
    assert_eq!(file.size(), 5);
    assert_eq!(file.last_modified(), 7);
    assert_eq!(file.content_type().as_deref(), Some("text/plain"));
    let mut cx = Context::from_waker(Waker::noop());
    let Poll::Ready(text) = pin!(file.read_string()).poll(&mut cx) else {
        panic!("a held file's read waits on nothing");
    };
    assert_eq!(text.unwrap(), "hello");
    let untyped = held_file("a".into(), String::new(), 0, Vec::new());
    assert_eq!(untyped.content_type(), None);
}
