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

#[test]
fn it_walks_like_a_tree() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        WALK.assert_contract(&fixture.page).await.unwrap();
        fixture.console.assert_clean("walking the tree").unwrap();
        fixture.close().await.unwrap();
    });
}
