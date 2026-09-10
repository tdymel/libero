//! `Button`: a busy button swallows keyboard activation, every mode shows a
//! ring on Tab, and link mode navigates through the router (todo 406).

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const PLAIN: &str = "#plain";
const BUSY: &str = "#busy";
const LINK: &str = "#to-landing";

#[test]
fn it_meets_the_baseline() {
    Suite::new("button", "/button")
        .focusable(PLAIN)
        .focusable(BUSY)
        .focusable(LINK)
        .targets(PLAIN)
        .targets(BUSY)
        .targets(LINK)
        .run();
}

fn presses(which: &str) -> String {
    format!("document.querySelector('#presses').dataset.{which}")
}

/// The plain button is the control: its presses land after the busy ones in
/// the same event queue, so a busy count still at 0 once they have landed
/// means the busy presses were swallowed, not merely not yet handled.
#[test]
fn a_loading_button_swallows_enter_and_space() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, PLAIN, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '2'", presses("plain")),
            "Enter and Space to activate the plain button",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, BUSY, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();

        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '3'", presses("plain")),
            "the plain button's press after the busy ones",
        )
        .await
        .unwrap();

        let busy: String = page
            .evaluate(presses("busy"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(busy, "0", "a loading button ran its onclick");

        fixture
            .console
            .assert_clean("pressing a busy button")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The label against the background as painted, `null` while either is
/// translucent (the resting unfilled variants, or a transition under way).
const PAIR: &str = "(selector => {
    const style = getComputedStyle(document.querySelector(selector));
    const rgb = value => {
        const [r, g, b, a = 1] = value.match(/[\\d.]+/g).map(Number);
        return a < 1 ? null : [r, g, b];
    };
    const luminance = c => {
        const [r, g, b] = c.map(v => v / 255).map(v =>
            v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
    };
    const [fg, bg] = [rgb(style.color), rgb(style.backgroundColor)];
    if (!fg || !bg) return null;
    const [a, b] = [luminance(fg), luminance(bg)];
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
})";

/// Todo 452: an outlined `primary` label read 3.52:1 on its hover tint. The
/// resting unfilled buttons paint no background, so a ratio at all means the
/// pointer's rule applied; the wait also rides out any transition.
#[test]
fn a_hovered_button_s_label_reads_on_its_hover_fill() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        for selector in ["#outlined", "#standard-error", "#elevated", "#filled-muted"] {
            pointer::hover(page, selector).await.unwrap();
            let settled = format!(
                "(() => {{ const r = {PAIR}('{selector}'); \
                 if (r === null || r !== window.__last) {{ window.__last = r; return false; }} \
                 return true; }})()"
            );
            wait::for_js_true(page, &settled, "a settled hover pair")
                .await
                .unwrap();
            let ratio: f64 = page
                .evaluate(format!("{PAIR}('{selector}')"))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            eprintln!("{selector} hovered: {ratio:.2}:1");
            assert!(ratio >= 4.5, "{selector} hovered reads {ratio:.2}:1");
        }

        fixture.console.assert_clean("hovering buttons").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A marker on `window` survives only a client-side navigation, so it tells
/// the router's `Link` apart from a full page load of the same URL.
#[test]
fn link_mode_navigates_through_the_router() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        page.evaluate("window.__buttonMarker = 1").await.unwrap();
        keyboard::tab_to(page, LINK, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        wait::for_selector(page, "#landing").await.unwrap();
        let (path, kept): (String, bool) = page
            .evaluate("[location.pathname, window.__buttonMarker === 1]")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(path, "/button/landing");
        assert!(kept, "the link reloaded the page instead of routing");

        fixture
            .console
            .assert_clean("following a link button")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
