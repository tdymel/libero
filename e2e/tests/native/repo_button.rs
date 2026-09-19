//! `RepoButton` fetches through the network provider dioxus-native puts in the
//! root context. A stub stands in for it: no call leaves the test.

use std::{cell::RefCell, sync::Arc, time::Duration};

use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};
use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{RepoButton, RepoHost};

const LINK: &str = "#repo";

thread_local! {
    /// Every URL the stub was asked for, on this test's thread.
    static ASKED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Answers every fetch with `body`; `None` drops the handler, as a failed fetch does.
struct StubNet(Option<&'static str>);

impl NetProvider for StubNet {
    fn fetch(&self, _: usize, request: Request, handler: Box<dyn NetHandler>) {
        ASKED.with_borrow_mut(|asked| asked.push(request.url.to_string()));
        if let Some(body) = self.0 {
            handler.bytes(request.url.to_string(), Bytes::from_static(body.as_bytes()));
        }
    }
}

fn provide(body: Option<&'static str>) {
    use_context_provider(|| Arc::new(StubNet(body)) as Arc<dyn NetProvider>);
}

// One repository per app: the session cache is per thread and per repository.
fn failing() -> Element {
    provide(None);
    rsx! { RepoButton { id: "repo", repo: "native/failing" } }
}

fn zero() -> Element {
    provide(Some(r#"{"stargazers_count": 0}"#));
    rsx! { RepoButton { id: "repo", repo: "native/zero" } }
}

fn counted() -> Element {
    provide(Some(r#"{"stargazers_count": 1234}"#));
    rsx! { RepoButton { id: "repo", repo: "native/counted" } }
}

fn gitlab() -> Element {
    provide(Some(r#"{"star_count": 5}"#));
    rsx! { RepoButton { id: "repo", repo: "group/native", host: RepoHost::GitLab } }
}

fn unprovided() -> Element {
    rsx! { RepoButton { id: "repo", repo: "native/unprovided" } }
}

/// The page after the fetch task had its turn.
fn landed(app: fn() -> Element) -> Page {
    let mut page = mount(app);
    page.wait(Duration::from_millis(100));
    page
}

fn asked() -> Vec<String> {
    ASKED.with_borrow(Clone::clone)
}

#[test]
fn a_count_joins_the_icon() {
    let page = landed(counted);
    assert_eq!(asked(), ["https://api.github.com/repos/native/counted"]);
    assert_eq!(
        page.attr(LINK, "aria-label").as_deref(),
        Some("GitHub, 1.2k stars (opens in a new tab)"),
        "{}",
        page.tree()
    );
    assert_eq!(page.text(LINK).trim(), "1.2k");
    let (_, _, width, height) = page.rect(LINK);
    assert!(width > height, "no pill: {width}x{height}");
    assert_eq!(
        page.attr(LINK, "href").as_deref(),
        Some("https://github.com/native/counted")
    );
}

#[test]
fn a_remount_reads_the_session_not_the_network() {
    drop(landed(counted));
    let page = landed(counted);
    assert_eq!(asked().len(), 1, "fetched again: {:?}", asked());
    assert_eq!(page.text(LINK).trim(), "1.2k");
}

#[test]
fn a_failed_fetch_leaves_the_plain_icon() {
    let page = landed(failing);
    assert_eq!(asked().len(), 1);
    assert_eq!(
        page.attr(LINK, "aria-label").as_deref(),
        Some("GitHub (opens in a new tab)")
    );
    assert_eq!(page.text(LINK).trim(), "");
}

#[test]
fn zero_stars_leave_the_plain_icon() {
    let page = landed(zero);
    assert_eq!(
        page.attr(LINK, "aria-label").as_deref(),
        Some("GitHub (opens in a new tab)")
    );
    assert_eq!(page.text(LINK).trim(), "");
}

#[test]
fn gitlab_asks_its_own_endpoint_and_field() {
    let page = landed(gitlab);
    assert_eq!(
        asked(),
        ["https://gitlab.com/api/v4/projects/group%2Fnative"]
    );
    assert_eq!(
        page.attr(LINK, "aria-label").as_deref(),
        Some("GitLab, 5 stars (opens in a new tab)")
    );
}

/// An app that turned dioxus-native's `net` feature off: no count, no panic.
#[test]
fn without_a_provider_the_icon_stands_alone() {
    let page = landed(unprovided);
    assert_eq!(
        page.attr(LINK, "aria-label").as_deref(),
        Some("GitHub (opens in a new tab)")
    );
}
