//! `Marquee`: the pause toggle sits over the moving strip. A link focused
//! while under it must still show (WCAG 2.4.11), the overlap Carousel's pause
//! control had with Next (todo 551).

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const LINKS: &str = "[data-slot=group]:first-child a";
const PAUSE: &str = "[data-slot=pause]";

/// Every live link, stopped where its centre meets the toggle's, focused, and
/// sampled on a grid: some part of it has to be the topmost element.
#[test]
fn a_focused_link_is_never_hidden_under_the_pause_toggle() {
    block_on(async {
        let fixture = Fixture::open("/marquee", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, PAUSE).await.unwrap();
        let hidden: Vec<String> = page
            .evaluate(format!(
                r#"(async () => {{
                    const track = document.querySelector('[data-slot=track]');
                    const anim = track.getAnimations()[0];
                    anim.pause();
                    anim.currentTime = 0;
                    const duration = anim.effect.getComputedTiming().duration;
                    const groups = track.querySelectorAll('[data-slot=group]');
                    const shift = groups[1].getBoundingClientRect().left - groups[0].getBoundingClientRect().left;
                    const toggle = document.querySelector({PAUSE:?}).getBoundingClientRect();
                    const target = toggle.left + toggle.width / 2;
                    const hidden = [];
                    let tried = 0;
                    for (const link of document.querySelectorAll({LINKS:?})) {{
                        anim.currentTime = 0;
                        const r0 = link.getBoundingClientRect();
                        const travel = r0.left + r0.width / 2 - target;
                        if (travel < 0 || travel >= shift) continue;
                        tried++;
                        anim.currentTime = travel / shift * duration;
                        link.focus();
                        await new Promise(requestAnimationFrame);
                        const r = link.getBoundingClientRect();
                        let seen = false;
                        for (let i = 1; i < 6 && !seen; i++)
                            for (let j = 1; j < 4 && !seen; j++) {{
                                const at = document.elementFromPoint(r.left + r.width * i / 6, r.top + r.height * j / 4);
                                seen = !!at && link.contains(at);
                            }}
                        if (!seen) hidden.push(link.textContent);
                        link.blur();
                    }}
                    return tried ? hidden : ['no link passes the toggle; the fixture tests nothing'];
                }})()"#
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            hidden.is_empty(),
            "focused links wholly under the pause toggle: {hidden:?}"
        );

        // Tab from the last link still reaches the toggle, drawn again.
        page.evaluate(format!(
            "[...document.querySelectorAll({LINKS:?})].pop().focus()"
        ))
        .await
        .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const t = document.querySelector({PAUSE:?}); \
                 return document.activeElement === t && getComputedStyle(t).opacity === '1'; }})()"
            ),
            "Tab from the last link to reach the drawn toggle",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the marquee").unwrap();
        fixture.close().await.unwrap();
    });
}
