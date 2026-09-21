//! `use_accessibility()` follows the browser's own answers, live, and a forced
//! reduced motion wins over them in libero's CSS (954) and is kept (981).

use e2e::browser::{Scheme, block_on, emulate_media};
use e2e::{Fixture, Viewport, wait};

/// Test pages share one profile, so a kept `reduce` would leak into later fixtures.
/// A stand-in store, seeded with `kept`, records what libero tried to keep.
fn shield_storage(kept: Option<&str>) -> String {
    let seed = kept.map_or("[]".into(), |kept| {
        format!("[['lsx-reduced-motion', '{kept}']]")
    });
    format!(
        r#"(() => {{
        window.__realStorage = window.localStorage;
        window.__stored = [];
        const items = new Map({seed});
        Object.defineProperty(window, 'localStorage', {{ configurable: true, value: {{
            getItem: k => items.has(k) ? items.get(k) : null,
            setItem: (k, v) => {{ items.set(k, String(v)); window.__stored.push(k + '=' + v); }},
            removeItem: k => {{ items.delete(k); window.__stored.push('-' + k); }},
        }}}});
    }})()"#
    )
}

/// What libero tried to keep, and what reached the real store.
async fn kept(page: &chromiumoxide::Page) -> (Vec<String>, Option<String>) {
    page.evaluate("[window.__stored, window.__realStorage.getItem('lsx-reduced-motion')]")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn wait_for(page: &chromiumoxide::Page, motion: &str, width: &str) {
    wait::for_js_true(
        page,
        &format!(
            "document.querySelector('#motion')?.textContent === '{motion}' && \
             getComputedStyle(document.querySelector('#motion-box')).width === '{width}'"
        ),
        &format!("the page to read {motion:?} at width {width}"),
    )
    .await
    .unwrap();
}

async fn click(page: &chromiumoxide::Page, id: &str) {
    page.evaluate(format!("document.querySelector('#{id}').click()"))
        .await
        .unwrap();
}

#[test]
fn reduced_motion_follows_the_browser_until_forced() {
    block_on(async {
        let fixture = Fixture::open("/use-accessibility", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(shield_storage(None)).await.unwrap();
        emulate_media(page, Scheme::Light, Some(false))
            .await
            .unwrap();
        wait_for(page, "no-preference", "10px").await;
        emulate_media(page, Scheme::Light, Some(true))
            .await
            .unwrap();
        wait_for(page, "reduce", "20px").await;

        click(page, "moving").await;
        wait_for(page, "no-preference", "10px").await;
        emulate_media(page, Scheme::Light, Some(false))
            .await
            .unwrap();
        click(page, "still").await;
        wait_for(page, "reduce", "20px").await;

        click(page, "system").await;
        wait_for(page, "no-preference", "10px").await;
        let (stored, leaked) = kept(page).await;
        assert_eq!(
            stored,
            [
                "lsx-reduced-motion=no-preference",
                "lsx-reduced-motion=reduce",
                "-lsx-reduced-motion"
            ],
            "each choice is kept, and following the system drops it"
        );
        assert_eq!(leaked, None, "a choice reached the shared localStorage");
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_kept_choice_wins_after_a_reload() {
    block_on(async {
        let fixture = Fixture::open("/use-accessibility", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        emulate_media(page, Scheme::Light, Some(false))
            .await
            .unwrap();
        wait_for(page, "no-preference", "10px").await;

        page.evaluate_on_new_document(shield_storage(Some("reduce")))
            .await
            .unwrap();
        page.reload().await.unwrap();
        wait_for(page, "reduce", "20px").await;

        click(page, "system").await;
        wait_for(page, "no-preference", "10px").await;
        let (stored, leaked) = kept(page).await;
        assert_eq!(stored, ["-lsx-reduced-motion"]);
        assert_eq!(leaked, None, "a choice reached the shared localStorage");
        fixture
            .console
            .assert_clean("reading a kept reduced motion")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
