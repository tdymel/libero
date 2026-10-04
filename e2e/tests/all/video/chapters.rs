//! Todo 2136: chapters on the seek track, beside the time, in a menu and on Ctrl+Arrow.

use e2e::browser::block_on;
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::keyboard::{self, CTRL};
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

const LOADED: &str = "['prop', 'vtt', 'both', 'rtl', 'narrow'].every((id) =>
    document.querySelector(`#${id} [data-slot=time]`)?.textContent.endsWith('0:04'))";

fn time(id: &str) -> String {
    format!("document.querySelector('#{id} video').currentTime")
}

/// The prop's three chapters as three segments, 2px apart and as wide as their
/// spans; the track's two as two; the prop beats the track; the title is in the
/// thumb's value and beside the time, which a 320px player drops, keeping one row.
#[test]
fn the_seek_track_splits_into_chapters() {
    block_on(async {
        let fixture = Fixture::open("/video/chapters", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, LOADED, "every duration")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "(() => { const q = (id, s) => document.querySelectorAll(`#${id} ${s}`);
             const widths = [...q('prop', '[data-slot=segment]')].map((s) => s.getBoundingClientRect());
             if (widths.length !== 3 || q('vtt', '[data-slot=segment]').length !== 2) return false;
             const gap = widths[1].left - widths[0].right;
             const thumb = document.querySelector('#prop [role=slider]');
             const label = (id) => q(id, '[data-slot=chapter]')[0]?.textContent;
             return Math.abs(gap - 2) < 0.5
                 && Math.abs(widths[2].width / widths[1].width - 1 / 1.5) < 0.2
                 && thumb.getAttribute('aria-valuetext') === '0:00 of 0:04, Intro'
                 && label('prop') === 'Intro' && label('vtt') === 'Opening' && label('both') === 'Intro'
                 && getComputedStyle(q('narrow', '[data-slot=chapter]')[0]).display === 'none'
                 && new Set([...q('narrow', '[data-slot=controls] > :not([data-slot=seek])')]
                     .map((e) => e.getBoundingClientRect()).filter((r) => r.width > 1)
                     .map((r) => Math.round(r.top + r.height / 2))).size === 1; })()",
            "the segments, the valuetext and the label",
        )
        .await
        .unwrap();
        // A screenshot activates the tab, which ends the other video tests' fullscreen (todo 2169).
        let shot = e2e::frames::in_front(page, fixture.screenshot("video-chapters"))
            .await
            .unwrap();
        println!("screenshot: {}", shot.display());
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.11: the segments and their fill keep 3:1 on the scrim over a white
/// frame, and forced colours draw each segment's edge.
#[test]
fn the_segments_keep_their_contrast() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/video/chapters", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, LOADED, "every duration")
            .await
            .unwrap();
        page.evaluate("document.querySelector('#prop video').currentTime = 2")
            .await
            .unwrap();
        let ratios = format!(
            r#"(() => {{ {COLOUR_JS}
            const style = (s) => getComputedStyle(document.querySelector('#prop ' + s));
            const bar = style('[data-slot=controls]');
            const base = OVER(RGBA(bar.backgroundImage.match(/rgba?\([^)]+\)/)[0]), [255, 255, 255, 1]);
            const ratio = (c) => CONTRAST(OVER(RGBA(c), base), base);
            return [ratio(style('[data-slot=segment]').backgroundColor),
                ratio(style('[data-slot=segment-fill]').backgroundColor)]; }})()"#
        );
        let [segment, fill]: [f64; 2] = page
            .evaluate(ratios.as_str())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(segment >= 3.0, "a segment is {segment:.2}:1");
        assert!(fill >= 3.0, "a segment's fill is {fill:.2}:1");
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('#prop [data-slot=segment]')].every((s) => {
                 const style = getComputedStyle(s);
                 return style.outlineStyle === 'solid' && style.outlineColor !== 'rgba(0, 0, 0, 0)'; })",
            "an edge on every segment",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A press on the second segment seeks into it; the menu lists the chapters and
/// seeks to one picked.
#[test]
fn a_press_or_the_menu_seeks_to_a_chapter() {
    block_on(async {
        let fixture = Fixture::open("/video/chapters", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, LOADED, "every duration")
            .await
            .unwrap();
        let middle: pointer::Point = page
            .evaluate(
                "(() => { const r = document.querySelectorAll('#prop [data-slot=segment]')[1].getBoundingClientRect();
                 return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        pointer::click_at(page, middle).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === 2 && document.querySelector('#prop [data-slot=chapter]').textContent === 'Middle'", time("prop")),
            "the press to seek into Middle",
        )
        .await
        .unwrap();
        pointer::click(page, "#prop button[aria-label=Chapters]")
            .await
            .unwrap();
        let outro = "[...document.querySelectorAll('[role=menuitemradio]')].find((e) => e.textContent.includes('0:03 Outro'))";
        wait::for_js_true(
            page,
            "document.querySelector('[role=menuitemradio][aria-checked=true]')?.textContent.includes('0:01 Middle')",
            "the menu, Middle checked",
        )
        .await
        .unwrap();
        page.evaluate(format!("{outro}.click()")).await.unwrap();
        wait::for_js_true(page, &format!("{} === 3", time("prop")), "Outro picked")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Ctrl+ArrowRight goes to the next chapter and Ctrl+ArrowLeft back; under RTL
/// they swap, as the track runs right to left.
#[test]
fn ctrl_arrows_step_through_the_chapters() {
    block_on(async {
        let fixture = Fixture::open("/video/chapters", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, LOADED, "every duration")
            .await
            .unwrap();
        for (id, next, back) in [
            ("prop", keyboard::ARROW_RIGHT, keyboard::ARROW_LEFT),
            ("rtl", keyboard::ARROW_LEFT, keyboard::ARROW_RIGHT),
        ] {
            page.evaluate(format!(
                "document.querySelector('#{id} [role=slider]').focus()"
            ))
            .await
            .unwrap();
            for (key, want) in [(next, 1.5), (next, 3.0), (back, 1.5)] {
                keyboard::press_with(page, key, CTRL).await.unwrap();
                wait::for_js_true(
                    page,
                    &format!("{} === {want}", time(id)),
                    &format!("{id}: {want} s"),
                )
                .await
                .unwrap();
            }
        }
        fixture.close().await.unwrap();
    });
}
