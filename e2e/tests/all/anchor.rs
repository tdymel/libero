//! `Anchor`: when each `underline` mode draws its line, at rest and under the pointer.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const LINE: &str = "text-decoration-line";

async fn lined<D: Driver>(d: &mut D, id: &str) -> Result<bool> {
    Ok(d.style(&format!("#{id}"), LINE)
        .await?
        .contains("underline"))
}

/// Todo 3996: `not-hover` is the mirror of `hover`: underlined at rest, bare under the pointer.
async fn the_underline_modes_draw_at_rest_and_under_the_pointer<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    for (id, at_rest) in [
        ("always", true),
        ("hover", false),
        ("not-hover", true),
        ("never", false),
    ] {
        ensure!(lined(d, id).await? == at_rest, "#{id} at rest");
    }

    for (id, under_pointer) in [
        ("always", true),
        ("hover", true),
        ("not-hover", false),
        ("never", false),
    ] {
        d.hover(&format!("#{id}")).await?;
        eventually(d, &format!("#{id} under the pointer"), async |d| {
            Ok(lined(d, id).await? == under_pointer)
        })
        .await?;
    }
    Ok(())
}

e2e::scenario!(
    the_underline_modes_draw_at_rest_and_under_the_pointer_in_every_renderer,
    "/anchor",
    the_underline_modes_draw_at_rest_and_under_the_pointer,
    android: skip("a touch screen has no hover"),
    ios: skip("a touch screen has no hover")
);

/// Todo 5334: a long word wraps inside a shrink-to-fit parent (1.4.10), and the docs' Header nav
/// at 320 px keeps each link on one line, though `anywhere` lowers min-content.
#[test]
fn a_long_word_wraps_and_a_nav_row_keeps_its_words() {
    block_on(async {
        let fixture = Fixture::open("/anchor/narrow", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#long-word").await.unwrap();
        let fit: String = page
            .evaluate_bounded(
                "(() => { const rect = (id) => document.querySelector(id).getBoundingClientRect(); \
                   const inside = rect('#long-word').right <= rect('#column').right + 0.5; \
                   const lines = (id) => { const range = document.createRange(); \
                     range.selectNodeContents(document.querySelector(id)); return range.getClientRects().length; }; \
                   const one = lines('#docs-link') === 1 && lines('#header-link') === 1; \
                   return `inside ${inside}, one line ${one}`; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(fit, "inside true, one line true");
        fixture.close().await.unwrap();
    });
}
