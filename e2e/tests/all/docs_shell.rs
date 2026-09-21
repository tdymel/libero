//! The docs shell's focus move and scroll reset after a navigation, driven
//! through the docs' own file.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

#[test]
fn a_navigation_focuses_the_new_heading_but_a_load_does_not() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/heading", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#to-b").await.unwrap();
        let focused: String = page
            .evaluate("document.activeElement.tagName")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(focused, "BODY", "the first render moved focus");
        page.evaluate("document.querySelector('#to-b').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.tagName === 'H1' && document.activeElement.textContent === 'Page B'",
            "focus on the new heading",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the heading fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `a11y_attributes()` gives the trigger a generated id, so the attribute finds it.
const TLDR: &str = "[aria-haspopup=menu]";

#[test]
fn the_tldr_menu_lists_four_new_tab_links_and_returns_focus() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/tldr", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, TLDR).await.unwrap();
        page.evaluate(format!("document.querySelector('{TLDR}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('[role=menu] a[role=menuitem]')].map((a) => a.textContent).join() \
             === 'ChatGPT,Google AI,Claude,Perplexity'",
            "the four providers",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('[role=menu] a')].every((a) => a.target === '_blank' \
             && a.rel === 'noopener noreferrer' && a.href.startsWith('https://example.test/')) \
             && document.querySelector('[role=menu] [role=group]').getAttribute('aria-labelledby') \
             && document.getElementById(document.querySelector('[role=menu] [role=group]') \
                .getAttribute('aria-labelledby')).textContent === 'Summarize with'",
            "new-tab links under a labelled group",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.textContent === 'ChatGPT'",
            "focus on the first provider",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.textContent === 'Google AI'",
            "ArrowDown moving on",
        )
        .await
        .unwrap();

        // Records whether the menu cancelled the click, then cancels it so no tab opens.
        page.evaluate(
            "window.__clicks = []; document.addEventListener('click', (e) => { \
             if (e.target.closest('a')) { window.__clicks.push(e.defaultPrevented); e.preventDefault(); } })",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            "window.__clicks.length === 1 && window.__clicks[0] === false \
             && !document.querySelector('[role=menu]') && document.activeElement.matches('[aria-haspopup=menu]')",
            "Space following the link, closing the menu and returning focus",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.textContent === 'ChatGPT'",
            "the menu reopened",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "window.__clicks.length === 2 && window.__clicks[1] === false \
             && document.activeElement.matches('[aria-haspopup=menu]')",
            "Enter following the link",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.textContent === 'ChatGPT'",
            "the menu reopened again",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=menu]') && document.activeElement.matches('[aria-haspopup=menu]')",
            "Escape returning focus to the button",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the tldr fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The largest `scrollTop` in the fixture's area: whichever box scrolls.
const AREA_SCROLL: &str = "Math.max(...[document.querySelector('#page-area'), \
    ...document.querySelectorAll('#page-area *')].map((e) => e.scrollTop))";

#[test]
fn a_navigation_starts_the_new_page_at_the_top() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/scroll", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#to-b").await.unwrap();
        page.evaluate(
            "document.querySelectorAll('#page-area, #page-area *').forEach((e) => e.scrollTop = 600)",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &format!("{AREA_SCROLL} >= 600"), "the area to scroll")
            .await
            .unwrap();
        page.evaluate("document.querySelector('#to-b').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('#page-area h1').textContent === 'Page B' && {AREA_SCROLL} === 0"),
            "the new page at the top",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the scroll fixture").unwrap();
        fixture.close().await.unwrap();
    });
}
