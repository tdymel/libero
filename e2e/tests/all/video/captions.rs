//! `Video` captions: the button and C toggle the subtitle track, a default track
//! loads shown, and the cues clear the bar.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

use super::{C, CAPTIONS, PLAY};

fn track_mode() -> &'static str {
    "document.querySelector('#player video').textTracks[0].mode"
}

#[test]
fn the_captions_button_and_c_toggle_the_subtitle_track() {
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{} === 'disabled'", track_mode()),
            "the track off at the start",
        )
        .await
        .unwrap();
        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} === 'showing' && document.querySelector('{CAPTIONS}').getAttribute('aria-pressed') === 'true' \
                 && !document.querySelector('{CAPTIONS}').hasAttribute('aria-haspopup')",
                track_mode()
            ),
            "the button to show the track",
        )
        .await
        .unwrap();

        keyboard::press(page, C).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} === 'hidden' && document.querySelector('{CAPTIONS}').getAttribute('aria-pressed') === 'false'",
                track_mode()
            ),
            "C to hide it",
        )
        .await
        .unwrap();

        // Todo 1324: without tracks the button stays, disabled but focusable and explained.
        let plain: bool = page
            .evaluate(
                "(() => { const b = document.querySelector('#plain button[aria-label=Captions]');
                 b.focus(); b.click();
                 return document.activeElement === b && b.getAttribute('aria-disabled') === 'true'
                     && b.getAttribute('aria-pressed') === 'false'
                     && document.getElementById(b.getAttribute('aria-describedby')).textContent === 'No captions for this video'; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            plain,
            "a player without tracks has a disabled, explained captions button"
        );

        fixture.close().await.unwrap();
    });
}

/// Todo 1324, as the docs demo: a default captions track loads its cues, starts
/// shown, and the button hides and shows it again; pressed, a bar marks its icon (1370).
#[test]
fn a_default_captions_track_loads_and_toggles() {
    block_on(async {
        let fixture = Fixture::open("/video/captions", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let state = |mode: &str, pressed: &str| {
            let mark = if pressed == "true" { "block" } else { "none" };
            format!(
                "(() => {{ const t = document.querySelector('#player video').textTracks[0];
                 const button = document.querySelector('{CAPTIONS}');
                 return t.cues !== null && t.cues.length === 1 && t.mode === '{mode}'
                     && button.getAttribute('aria-pressed') === '{pressed}'
                     && getComputedStyle(button.querySelector('[data-mark]')).display === '{mark}'; }})()"
            )
        };
        wait::for_js_true(page, &state("showing", "true"), "the cues, shown")
            .await
            .unwrap();
        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(page, &state("hidden", "false"), "the button to hide them")
            .await
            .unwrap();
        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(page, &state("showing", "true"), "the button to show them")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1284: with two caption or subtitle tracks the captions button opens a menu of
/// them and Off, as native players do; C toggles the one last chosen, else the first.
#[test]
fn two_tracks_make_the_captions_button_a_track_menu() {
    const MODES: &str =
        "[...document.querySelector('#player video').textTracks].map((t) => t.mode).join()";
    const ITEM: &str = "[...document.querySelectorAll('[role=menuitemradio]')]";
    block_on(async {
        let fixture = Fixture::open("/video/languages", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let closed_on_button = format!(
            "document.querySelector('[role=menu]') === null \
             && document.activeElement === document.querySelector('{CAPTIONS}')"
        );
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{CAPTIONS}')?.getAttribute('aria-haspopup') === 'menu' \
                 && !document.querySelector('{CAPTIONS}').hasAttribute('aria-pressed')"
            ),
            "a menu button, not a toggle",
        )
        .await
        .unwrap();
        page.evaluate(format!("document.querySelector('{CAPTIONS}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, C).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{MODES} === 'showing,hidden'"),
            "C with no choice yet to show the first track",
        )
        .await
        .unwrap();
        keyboard::press(page, C).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{MODES} === 'hidden,hidden'"),
            "C to hide it",
        )
        .await
        .unwrap();

        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{ITEM}.map((e) => e.textContent.trim() + ':' + e.getAttribute('aria-checked')).join() \
                 === 'Off:true,English:false,Deutsch:false'"
            ),
            "Off and both tracks, Off checked",
        )
        .await
        .unwrap();
        page.evaluate(format!(
            "{ITEM}.find((e) => e.textContent.includes('Deutsch')).click()"
        ))
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{MODES} === 'hidden,showing' && {closed_on_button}"),
            "Deutsch shown, the menu closed and focus on the button",
        )
        .await
        .unwrap();

        keyboard::press(page, C).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{MODES} === 'hidden,hidden'"),
            "C to hide it",
        )
        .await
        .unwrap();
        keyboard::press(page, C).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{MODES} === 'hidden,showing'"),
            "C to show the last chosen again",
        )
        .await
        .unwrap();

        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{ITEM}.find((e) => e.textContent.includes('Deutsch'))?.getAttribute('aria-checked') === 'true'"
            ),
            "Deutsch checked",
        )
        .await
        .unwrap();
        page.evaluate(format!(
            "{ITEM}.find((e) => e.textContent.includes('Off')).click()"
        ))
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{MODES} === 'hidden,hidden' && {closed_on_button}"),
            "Off to hide them",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The cue box Chromium draws in the `<video>`'s own shadow tree, by its bottom edge;
/// `None` until it is drawn.
async fn cue_bottom(page: &chromiumoxide::Page) -> anyhow::Result<Option<f64>> {
    use chromiumoxide::cdp::browser_protocol::dom::{GetBoxModelParams, GetDocumentParams, Node};
    fn find(node: &Node) -> Option<&Node> {
        let attrs = node.attributes.as_deref().unwrap_or_default();
        if attrs
            .chunks(2)
            .any(|pair| pair[0] == "pseudo" && pair[1] == "-webkit-media-text-track-display")
        {
            return Some(node);
        }
        [&node.children, &node.shadow_roots, &node.pseudo_elements]
            .into_iter()
            .flatten()
            .flatten()
            .find_map(find)
    }
    let document = page
        .execute(GetDocumentParams::builder().depth(-1).pierce(true).build())
        .await?;
    let Some(cue) = find(&document.result.root).map(|node| node.backend_node_id) else {
        return Ok(None);
    };
    let model = page
        .execute(GetBoxModelParams::builder().backend_node_id(cue).build())
        .await?;
    // Corners clockwise from the top left: the bottom edge's y is the sixth number.
    Ok(model.result.model.border.inner().get(5).copied())
}

/// Todos 1371 and 1389: the shown captions sit above the bar, not under it, at xl too.
#[test]
fn the_captions_clear_the_bar() {
    block_on(async {
        for route in ["/video/captions", "/video/captions/xl"] {
            captions_clear_the_bar(route).await;
        }
    });
}

async fn captions_clear_the_bar(route: &str) {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    let page = &fixture.page;
    wait::for_js_true(
        page,
        "document.querySelector('#player video').textTracks[0].cues?.length === 1",
        "the cues",
    )
    .await
    .unwrap();
    pointer::click(page, PLAY).await.unwrap();
    let seek_top: f64 = page
        .evaluate("document.querySelector('#player [data-slot=seek]').getBoundingClientRect().top")
        .await
        .unwrap()
        .into_value()
        .unwrap();
    let last = &std::cell::Cell::new(None);
    wait::until(
        &format!("{route}: the cue to end above the seek row's top {seek_top}"),
        move || async move {
            let bottom = cue_bottom(page).await?;
            last.set(bottom);
            Ok(bottom.is_some_and(|bottom| bottom <= seek_top))
        },
    )
    .await
    .unwrap_or_else(|e| panic!("{e}; the cue ends at {:?}", last.get()));
    fixture.close().await.unwrap();
}
