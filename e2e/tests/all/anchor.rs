//! `Anchor`: when each `underline` mode draws its line, at rest and under the pointer.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard::{ENTER, SPACE};
use e2e::passes::pointer;
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

async fn count<D: Driver>(d: &mut D, id: &str) -> Result<String> {
    d.text(&format!("#{id}")).await
}

/// Todo 5363: without `to` it is a `<button>` with the link's underline, run by click, Enter and Space.
async fn an_anchor_without_to_is_a_button<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.exists("button#action[type=button]").await?,
        "a <button type=button>"
    );
    ensure!(lined(d, "action").await?, "the link's underline");

    d.click("#action").await?;
    eventually(d, "a click runs onclick", async |d| {
        Ok(count(d, "actions").await? == "1")
    })
    .await?;
    d.focus("#action").await?;
    d.press(ENTER).await?;
    eventually(d, "Enter runs onclick", async |d| {
        Ok(count(d, "actions").await? == "2")
    })
    .await?;
    d.press(SPACE).await?;
    eventually(d, "Space runs onclick", async |d| {
        Ok(count(d, "actions").await? == "3")
    })
    .await?;
    Ok(())
}

e2e::scenario!(
    an_anchor_without_to_is_a_button_in_every_renderer,
    "/anchor/action",
    an_anchor_without_to_is_a_button
);

/// Todo 5363: with `to`, a click follows the link and runs `onclick` too.
#[test]
fn a_link_with_onclick_follows_and_runs_it() {
    block_on(async {
        let fixture = Fixture::open("/anchor/action", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#follow").await.unwrap();
        pointer::click(page, "#follow").await.unwrap();
        let js = "location.hash === '#followed' && document.getElementById('follows').textContent === '1'";
        wait::for_js_true(page, js, "the link followed and onclick ran")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}
