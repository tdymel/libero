//! `Highlight`: every match of its terms in a `Mark`.

use e2e::browser::block_on;
use e2e::passes::contrast;
use e2e::wait;
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport};

#[test]
fn it_meets_the_baseline() {
    Suite::new("highlight", "/highlight").run();
}

/// Todo 3983: every case of a term is a `mark`, the longest overlapping term wins, and the
/// text reads whole; each tint clears contrast.
#[test]
fn a_highlight_marks_every_match() {
    block_on(async {
        let fixture = Fixture::open("/highlight", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#highlight mark").await.unwrap();
        let read: Vec<String> = page
            .evaluate_bounded(
                "(() => { const h = document.querySelector('#highlight'); \
                 return [h.textContent, ...[...h.querySelectorAll('mark')].map(m => m.textContent)]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            read,
            [
                "Rust for rust, RUST in dioxus apps",
                "Rust",
                "rust",
                "RUST",
                "dioxus"
            ]
        );
        contrast::assert_clean(page, "body").await.unwrap();
        fixture
            .console
            .assert_clean("the highlight fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
