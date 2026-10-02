//! The harness's own promise that tests sharing one browser cannot disturb
//! each other.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, frames, js, wait};

/// A chord one unit sends leaves every other page alone (597): Ctrl+PageDown once
/// activated another test's page, and Alt+ArrowLeft sent it Back.
#[test]
fn a_chord_in_one_page_does_not_navigate_another() {
    block_on(async {
        let other = Fixture::open("/loader", Viewport::Desktop).await.unwrap();
        let sender = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &sender.page;
        // The active tab takes the Back, as after `frames::start` in a full run.
        let front = e2e::frames::bring_to_front(&other.page).await.unwrap();
        let outcome = async {
            let refused = keyboard::press_with(page, keyboard::PAGE_DOWN, keyboard::CTRL).await;
            anyhow::ensure!(refused.is_err(), "Ctrl+PageDown was sent");
            keyboard::assert_chords_ignored(page, &[keyboard::PAGE_UP, keyboard::PAGE_DOWN], "1")
                .await?;
            keyboard::press_with(page, keyboard::ARROW_LEFT, keyboard::ALT).await?;
            page.evaluate("new Promise(r => setTimeout(() => r(1), 300))")
                .await?;
            let href: String = other.page.evaluate("location.href").await?.into_value()?;
            anyhow::ensure!(href.ends_with("/loader"), "the other page went to {href}");
            Ok(())
        }
        .await;
        let released = front.release().await;
        let _ = sender.close().await;
        let _ = other.close().await;
        outcome.unwrap();
        released.unwrap();
    });
}

/// An open combobox stays open while another test opens a page beside it.
#[test]
fn a_page_keeps_focus_while_another_opens() {
    block_on(async {
        let first = Fixture::open("/autocomplete", Viewport::Desktop)
            .await
            .unwrap();
        keyboard::tab_to(&first.page, "[role=combobox]", 10)
            .await
            .unwrap();
        keyboard::press(&first.page, keyboard::ARROW_DOWN)
            .await
            .unwrap();
        let expanded =
            "document.querySelector('[role=combobox]').getAttribute('aria-expanded') === 'true'";
        wait::for_js_true(&first.page, expanded, "the list to open")
            .await
            .unwrap();

        let second = Fixture::open("/select", Viewport::Desktop).await.unwrap();
        keyboard::tab_to(&second.page, "[role=combobox]", 10)
            .await
            .unwrap();

        let (has_focus, still_open): (bool, bool) = first
            .page
            .evaluate(format!("[document.hasFocus(), {expanded}]"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let _ = second.close().await;
        let _ = first.close().await;
        assert!(
            has_focus && still_open,
            "opening a second page took focus from the first (hasFocus {has_focus}, list open {still_open})"
        );
    });
}

/// A click lands on its target or fails (todo 1756): an off-screen target is scrolled to,
/// a covered one and a point off the viewport are errors, not clicks elsewhere.
#[test]
fn a_click_reaches_its_target_or_fails() {
    block_on(async {
        let fixture = Fixture::open("/loader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let _: bool = js(
            page,
            "(() => { const add = (id, style) => { const b = document.createElement('button'); \
               b.id = id; b.textContent = id; b.style.cssText = style; \
               b.onclick = () => b.dataset.clicked = 'true'; document.body.append(b); }; \
             add('far', 'position:absolute;top:3000px;left:10px'); \
             add('covered', 'position:fixed;top:10px;left:10px'); \
             const cover = document.createElement('div'); cover.id = 'cover'; \
             cover.style.cssText = 'position:fixed;top:0;left:0;width:200px;height:200px;z-index:9'; \
             document.body.append(cover); return true; })()",
        )
        .await;

        pointer::click(page, "#far").await.unwrap();
        wait::for_js_true(
            page,
            "far.dataset.clicked === 'true'",
            "the far button's click",
        )
        .await
        .unwrap();

        let covered = pointer::click(page, "#covered").await.unwrap_err();
        assert!(
            covered.to_string().contains("would land on div#cover"),
            "{covered}"
        );
        let off = pointer::click_at(page, pointer::Point { x: -5.0, y: -5.0 })
            .await
            .unwrap_err();
        assert!(off.to_string().contains("hits nothing"), "{off}");
        e2e::clock::settle(page).await.unwrap();
        let clicked: bool = js(page, "covered.dataset.clicked === 'true'").await;
        assert!(!clicked, "the covered button took a click");

        fixture.close().await.unwrap();
    });
}

/// A gesture inside a fullscreen hold runs at once rather than waiting on the hold it is
/// part of, and a wait for front behind another test's hold ends within the budget (1755).
#[test]
fn the_front_lock_neither_deadlocks_nor_waits_forever() {
    block_on(async {
        let fixture = Fixture::open("/loader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let bound = std::time::Duration::from_secs(10);

        let held = frames::keep_fullscreen().await;
        let own = tokio::time::timeout(bound, frames::in_front(page, async { Ok(()) })).await;
        drop(held);
        assert!(
            own.is_ok(),
            "a gesture waited on its own test's fullscreen hold"
        );

        // Another test's hold: a spawned task is outside this test's scope.
        let (taken, take) = tokio::sync::oneshot::channel();
        let (done, finish) = tokio::sync::oneshot::channel::<()>();
        let other = tokio::spawn(async move {
            let _held = frames::keep_fullscreen().await;
            let _ = taken.send(());
            let _ = finish.await;
        });
        take.await.unwrap();
        let waited = tokio::time::timeout(
            bound,
            wait::expecting_failure_in(wait::QUICK_FAILURE_SHARE, frames::bring_to_front(page)),
        )
        .await;
        let _ = done.send(());
        other.await.unwrap();
        let error = match waited {
            Ok(Err(error)) => error.to_string(),
            Ok(Ok(_)) => panic!("brought a page to front over another test's fullscreen"),
            Err(_) => panic!("the wait for front ran past {bound:?}"),
        };
        assert!(
            error.contains("a test in fullscreen holds the tabs"),
            "{error}"
        );

        fixture.close().await.unwrap();
    });
}
