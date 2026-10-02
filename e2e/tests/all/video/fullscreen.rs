//! `Video` fullscreen: the native API or the drawn box takes the player, and Escape,
//! F, Back, the exit button or a Tab out gives it back; menus and tooltips open inside.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

use super::{F, FULLSCREEN, PLAY, STATE};

const BODY_OVERFLOW: &str = "getComputedStyle(document.body).overflowY";

/// The fallback where the Fullscreen API is refused: the player as a fixed box over the
/// page, which locks the page's scroll as a modal does (1285), and which Escape and F
/// leave. The route refuses it, so this runs every time (1758).
#[test]
fn fullscreen_takes_the_player_and_escape_or_f_gives_it_back() {
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video/refused", Viewport::Desktop)
            .await
            .unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "document.querySelector('#player[data-refused] [data-slot=time]')?.textContent === '0:00 / 0:04'",
            "the refusal and the duration",
        )
        .await
        .unwrap();
        pointer::click(page, FULLSCREEN).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{STATE} !== 'none' \
                 && document.querySelector('#player button[aria-label=\"Exit fullscreen\"]') !== null"
            ),
            "the player to fill the screen",
        )
        .await
        .unwrap();
        let state: String = page.evaluate(STATE).await.unwrap().into_value().unwrap();
        assert_eq!(state, "drawn", "the refused player draws its own box");
        wait::for_js_true(
            page,
            "(() => { const r = document.querySelector('#player [role=group]').getBoundingClientRect(); \
             return r.top === 0 && r.left === 0 && r.width === innerWidth && r.height === innerHeight; })()",
            "the fixed box to cover the viewport",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{BODY_OVERFLOW} === 'hidden'"),
            "the page behind to stop scrolling",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{STATE} === 'none' && {BODY_OVERFLOW} !== 'hidden'"),
            "Escape to leave and give the scroll back",
        )
        .await
        .unwrap();
        keyboard::press(page, F).await.unwrap();
        wait::for_js_true(page, &format!("{STATE} !== 'none'"), "F to enter")
            .await
            .unwrap();

        keyboard::press(page, F).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{STATE} === 'none' && document.fullscreenElement === null"),
            "F to leave fullscreen",
        )
        .await
        .unwrap();

        fixture.close().await.unwrap();
    });
}

/// Native or pseudo, whichever the renderer grants; Blitz has no controls to press.
async fn fullscreen_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let _fullscreen = e2e::frames::keep_fullscreen().await;
    if d.platform() == Platform::Native {
        return Ok(());
    }
    eventually(d, "the controls", async |d| d.exists(FULLSCREEN).await).await?;
    let filled = async |d: &mut D| {
        d.click(FULLSCREEN).await?;
        eventually(d, "the player to fill the screen", async |d| {
            Ok(d.attr("#player [role=group]", "data-fullscreen")
                .await?
                .is_some())
        })
        .await
    };
    // Android's native fullscreen too, which ignored Escape before 1256.
    filled(d).await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to give the player back", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_none())
    })
    .await?;
    // Back leaves it too, rather than the app (1275).
    if d.platform() == Platform::Android {
        filled(d).await?;
        d.press_back().await?;
        eventually(d, "Back to give the player back", async |d| {
            Ok(d.attr("#player [role=group]", "data-fullscreen")
                .await?
                .is_none())
        })
        .await?;
    }
    filled(d).await?;
    d.click("#player button[aria-label='Exit fullscreen']")
        .await?;
    eventually(d, "the player back in the page", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_none())
    })
    .await
}

e2e::scenario!(the_fullscreen_button_toggles, "/video", fullscreen_toggles);

fn pseudo_state(fullscreen: Option<String>) -> bool {
    fullscreen.as_deref() == Some("drawn")
}

/// The drawn box covers the page, so a Tab out of the player gives the page back;
/// a Tab between its controls does not. The route refuses the Fullscreen API, so
/// WebKitGTK, which grants it, draws the box too.
async fn tab_out_leaves_pseudo<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let _fullscreen = e2e::frames::keep_fullscreen().await;
    if d.platform() == Platform::Native {
        return Ok(());
    }
    eventually(d, "the refusal and the controls", async |d| {
        Ok(d.exists("#player[data-refused]").await? && d.exists(FULLSCREEN).await?)
    })
    .await?;
    d.click(FULLSCREEN).await?;
    eventually(d, "the drawn box", async |d| {
        Ok(pseudo_state(
            d.attr("#player [role=group]", "data-fullscreen").await?,
        ))
    })
    .await?;
    d.focus(PLAY).await?;
    d.press(keyboard::TAB).await?;
    d.settle().await?;
    let inside = d.attr("#player [role=group]", "data-fullscreen").await?;
    anyhow::ensure!(pseudo_state(inside), "a Tab inside the player left the box");
    d.focus("#player button[aria-label='Exit fullscreen']")
        .await?;
    d.press(keyboard::TAB).await?;
    eventually(d, "a Tab out to give the page back", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_none())
    })
    .await
}

e2e::scenario!(
    a_tab_out_of_the_drawn_fullscreen_leaves_it,
    "/video/refused",
    tab_out_leaves_pseudo
);

/// Todo 1370: in fullscreen the page's portal is outside the player, which hides
/// it under the native API; the speed menu opens inside the player and works.
#[test]
fn the_speed_menu_works_in_fullscreen() {
    const SPEED: &str = "#player button[aria-label^='Playback speed']";
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{FULLSCREEN}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        pointer::click(page, FULLSCREEN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{STATE} !== 'none'"),
            "the player to fill the screen",
        )
        .await
        .unwrap();
        pointer::click(page, SPEED).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const item = [...document.querySelectorAll('[role=menuitemradio]')]
                 .find((e) => e.textContent.includes('1.5×'));
             if (!item || !document.querySelector('#player [role=group]').contains(item)) return false;
             const r = item.getBoundingClientRect();
             return item.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)); })()",
            "the menu inside the player, on top",
        )
        .await
        .unwrap();
        page.evaluate(
            "[...document.querySelectorAll('[role=menuitemradio]')].find((e) => e.textContent.includes('1.5×')).dataset.pick = ''",
        )
        .await
        .unwrap();
        pointer::click(page, "[data-pick]").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('#player video').playbackRate === 1.5
                 && document.querySelector(\"{SPEED}\").textContent.includes('1.5×')"
            ),
            "the rate at 1.5×",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1349: in fullscreen a control's tooltip opens inside the player, on top.
#[test]
fn a_tooltip_shows_in_fullscreen() {
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{FULLSCREEN}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        pointer::click(page, FULLSCREEN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{STATE} !== 'none'"),
            "the player to fill the screen",
        )
        .await
        .unwrap();
        pointer::hover(page, PLAY).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const tip = [...document.querySelectorAll('[role=tooltip]')].find((e) => e.textContent.includes('Play'));
             if (!tip || !document.querySelector('#player [role=group]').contains(tip)) return false;
             const r = tip.getBoundingClientRect();
             return r.width > 0 && tip.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)); })()",
            "the tooltip inside the player, on top",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
