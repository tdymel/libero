//! `Avatar` and `AvatarGroup`: a name for every avatar, none for a decorative
//! one, and a keyboard-reachable `+N` chip naming the people it hides.

use e2e::browser::block_on;
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
