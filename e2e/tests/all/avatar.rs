//! `Avatar` and `AvatarGroup`: a name for every avatar, none for a decorative
//! one, and a keyboard-reachable `+N` chip naming the people it hides.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    // The fallback lands with the 404: axe and the baseline see it, not the `<img>` (todo 703).
    Suite::new("avatar", "/avatar")
        .ready("#broken:not(:has(img))")
        .focusable("#group [tabindex='0']")
        .run();
}

#[test]
fn a_broken_picture_falls_back_to_the_initials_under_the_same_name() {
    block_on(async {
        let fixture = Fixture::open("/avatar", Viewport::Desktop).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            "(() => { const a = document.querySelector('#broken'); \
             return !a.querySelector('img') && a.textContent === 'GH' \
             && a.getAttribute('aria-label') === 'Grace Hopper'; })()",
            "the initials in place of the broken picture",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3119: keyboard focus on the `+N` chip shows the hidden names its `aria-label`
/// names, and Escape closes the bubble.
#[test]
fn the_more_chip_tooltip_names_the_hidden_people() {
    const CHIP: &str = "#group [tabindex='0']";
    const HIDDEN: &str = "Radia Perlman, Barbara Liskov";
    block_on(async {
        let fixture = Fixture::open("/avatar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, CHIP, 20).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "[...document.querySelectorAll('[role=tooltip]:not([hidden])')].some(t => t.textContent === {HIDDEN:?})"
            ),
            "focus on the chip to show the hidden names",
        )
        .await
        .unwrap();
        let label: String = e2e::js(
            page,
            format!("document.querySelector({CHIP:?}).getAttribute('aria-label')"),
        )
        .await;
        // Todo 4538: named by the count, the names said once as its description.
        assert_eq!(label, "2 more", "the chip's name");

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=tooltip]:not([hidden])')",
            "Escape to close the bubble",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the +N chip tooltip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 4143, 4144: `total` counts people the list does not hold, and `surplus` draws the chip.
#[test]
fn the_chip_counts_the_total_and_draws_the_surplus() {
    block_on(async {
        let fixture = Fixture::open("/avatar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let [counted, drawn]: [String; 2] = e2e::js(
            page,
            "[...document.querySelectorAll('#group-total [tabindex=\"0\"], #group-surplus [tabindex=\"0\"]')]\
             .map(chip => chip.textContent)",
        )
        .await;
        assert_eq!([counted.as_str(), drawn.as_str()], ["+10", "2 more"]);
        fixture.console.assert_clean("the chip's total").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4905: under a finger the `+N` chip takes taps in a 44px box, over its neighbour.
#[test]
fn a_finger_reaches_the_small_chips_over_their_neighbour() {
    block_on(async {
        let fixture = Fixture::open("/avatar", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        crate::hit_area::emulate_coarse_pointer(page).await;
        for group in ["#group-xs", "#group-sm"] {
            // The points of a 40px box around the chip's centre that miss it.
            let misses: Vec<String> = e2e::js(
                page,
                format!(
                    "(() => {{ const chip = document.querySelector('{group} [tabindex=\"0\"]'); \
                     chip.scrollIntoView({{ block: 'center' }}); \
                     const r = chip.getBoundingClientRect(); \
                     const cx = r.x + r.width / 2, cy = r.y + r.height / 2; const misses = []; \
                     for (const dx of [-20, -11.5, 0, 11.5, 20]) for (const dy of [-20, 0, 20]) {{ \
                       const at = document.elementFromPoint(cx + dx, cy + dy); \
                       if (!at || at.closest('[tabindex]') !== chip) \
                         misses.push(`${{dx}},${{dy}}: ${{at ? at.outerHTML.slice(0, 50) : null}}`); }} \
                     return misses; }})()"
                ),
            )
            .await;
            assert!(misses.is_empty(), "{group}: taps miss the chip: {misses:?}");
        }
        fixture
            .console
            .assert_clean("tapping the small chips")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 2363, 2601: the smallest initials draw at 10 px or more, and every size follows the root.
#[test]
fn the_initials_are_legible_and_follow_the_text_size() {
    block_on(async {
        let fixture = Fixture::open("/avatar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let sizes = "[...document.querySelectorAll('#sizes > *')]\
                     .map(a => parseFloat(getComputedStyle(a).fontSize))";
        let at_default: Vec<f64> = page
            .evaluate_bounded(sizes)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(at_default, [10.0, 12.0, 15.0, 22.0, 34.0, 48.0]);
        let doubled: Vec<f64> = page
            .evaluate_bounded(format!(
                "(() => {{ document.documentElement.style.fontSize = '200%'; return {sizes}; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(doubled, [20.0, 24.0, 30.0, 44.0, 68.0, 96.0]);
        fixture.close().await.unwrap();
    });
}

/// Forced colours drop `box-shadow`: the ring that separates grouped avatars
/// stays as a `Canvas` outline (todo 2469).
#[test]
fn the_group_ring_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/avatar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(
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
        let rings: Vec<String> = page
            .evaluate_bounded(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.color = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).color; probe.remove(); \
                 return [...document.querySelectorAll(\"#group [data-state~='grouped']\")].map(a => { \
                   const s = getComputedStyle(a); \
                   return s.outlineStyle === 'solid' && s.outlineWidth !== '0px' \
                     && s.outlineColor === canvas ? 'ring' : `${s.outlineStyle} ${s.outlineColor}`; }); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            !rings.is_empty() && rings.iter().all(|ring| ring == "ring"),
            "{rings:?}"
        );
        fixture.close().await.unwrap();
    });
}
