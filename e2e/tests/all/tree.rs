//! `Tree`, against the tree pattern rather than the strip it was once filed
//! under.
//!
//! The archetype table used to list `Tree` under `RovingTabindex`, written from
//! APG without checking. It shares that pattern's single tab stop and vertical
//! arrows and nothing else: Right and Left open and close rows, and the set of
//! rows changes as they do. `TreeWalk` is the pattern it actually is (todo
//! 310); this unit runs it, and keeps only what is `Tree`'s own beside it.

use e2e::archetypes::TreeWalk;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::wait;
use e2e::{Fixture, Suite, Viewport};

const ROW: &str = "[role=treeitem]";

/// `/tree` starts as three roots, `src` and `docs` collapsed and `README.md` a
/// leaf. The two indices `TreeWalk` needs are declared here so the fixture
/// changing under it is a failure rather than a quietly weaker pass.
const WALK: TreeWalk = TreeWalk {
    rows: ROW,
    collapsed_parent: 0,
    leaf: 2,
    tab_budget: 5,
};

#[test]
fn it_meets_the_baseline() {
    Suite::new("tree", "/tree")
        .focusable(ROW)
        .targets(ROW)
        .run();
}

/// Todo 4: Enter on a leaf that is a link navigates. `Tree` activates the
/// link through `ElementApi::click`, since the leaf is the caller's content.
#[test]
fn enter_on_a_link_leaf_navigates() {
    block_on(async {
        let fixture = Fixture::open("/tree/links", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, ROW, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "location.pathname === '/tree/links/arrived'",
            "Enter on the link leaf to navigate",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("activating a link leaf")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_walks_like_a_tree() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        WALK.assert_contract(&fixture.page).await.unwrap();
        fixture.console.assert_clean("walking the tree").unwrap();
        fixture.close().await.unwrap();
    });
}
