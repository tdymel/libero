//! `Badge`, `Indicator`, `Kbd` and `List`: small-text contrast in both schemes, list
//! semantics that survive `list-style: none`, an `Indicator` dot in forced colours.

use e2e::browser::block_on;
use e2e::passes::contrast;
use e2e::suite::Suite;
use e2e::{Fixture, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("badge", "/badge")
        .waive(contrast::TODO_297)
        .run();
}

/// Todo 741: at 200% text size a px box clipped its rem label (md: 20px text in
/// an 18px box). Every size's line of text fits its box.
#[test]
fn a_badge_grows_with_the_text_size() {
    block_on(async {
        let fixture = Fixture::open("/badge", Viewport::Desktop).await.unwrap();
        let clipped: Vec<String> = fixture
            .page
            .evaluate(
                "(() => { document.documentElement.style.fontSize = '200%'; \
                 return [...document.querySelectorAll('#badge-sizes > span')] \
                 .filter(b => parseFloat(getComputedStyle(b).fontSize) * 1.2 > b.clientHeight) \
                 .map(b => b.textContent); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(clipped.is_empty(), "clipped at 200%: {clipped:?}");
        fixture.close().await.unwrap();
    });
}

/// Todo 743: an ordered `List` draws its numbers, in a gutter inside its box.
#[test]
fn an_ordered_list_shows_its_numbers() {
    block_on(async {
        let fixture = Fixture::open("/badge", Viewport::Desktop).await.unwrap();
        let shown: bool = fixture
            .page
            .evaluate(
                "(() => { const ol = document.querySelector('ol#ordered-list'); \
                 const li = ol.querySelector('li'); const s = getComputedStyle(li); \
                 return s.display === 'list-item' && s.listStyleType === 'decimal' \
                 && li.getBoundingClientRect().left - ol.getBoundingClientRect().left >= 16; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            shown,
            "the ordered list draws no numbers, or no room for them"
        );
        fixture.close().await.unwrap();
    });
}

/// Forced colours paint every fill `Canvas`: a bare `Indicator` dot, which is
/// nothing but its fill, must not vanish into the page.
#[test]
fn a_bare_indicator_dot_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/badge", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        let [dot, canvas]: [String; 2] = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 return [getComputedStyle(document.querySelector('#dot')).backgroundColor, canvas]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(dot, canvas, "the dot is filled with the page's own colour");
        fixture.close().await.unwrap();
    });
}
