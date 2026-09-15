//! `Mark`: a link inside it draws its focus ring against the tint.
//!
//! Todo 53, part two. `Mark` tints its background from a `var()`, which `sx`'s
//! `background()` cannot read a contrast twin off, so a link inside it drew
//! its ring from the `primary.6` fallback. The ring is an offset outline, so
//! the surface it is measured against is the link's parent: the `Mark`.

use e2e::browser::block_on;
use e2e::passes::focus;
use e2e::{Fixture, Viewport};

const LINKS: &[&str] = &[
    "#mark-default",
    "#mark-primary",
    "#mark-secondary",
    "#mark-error",
    "#mark-info",
    "#mark-success",
    // A dark shade: its white twin met the white halo at 1:1 until the fill became the halo (todo 630).
    "#mark-info-6",
];

#[test]
fn a_link_inside_a_mark_has_a_ring_that_clears_three_to_one() {
    block_on(async {
        let fixture = Fixture::open("/mark", Viewport::Desktop).await.unwrap();

        for link in LINKS {
            let ring = focus::assert_focus_ring(&fixture.page, link, LINKS.len() + 2)
                .await
                .unwrap();
            focus::assert_ring_contrast(&ring).unwrap_or_else(|e| panic!("{link}: {e}"));
            eprintln!(
                "{link}: outline {} against {}",
                ring.outline_color, ring.against
            );
        }

        fixture.console.assert_clean("the mark fixture").unwrap();
        fixture.close().await.unwrap();
    });
}
