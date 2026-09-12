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
use e2e::passes::{focus, keyboard, pointer};
use e2e::wait;
use e2e::{Fixture, Suite, Viewport};

const ROW: &str = "[role=treeitem]";

const STAR: keyboard::Key = keyboard::Key {
    key: "*",
    code: "NumpadMultiply",
    vk: 106,
    text: Some("*"),
};
const DELETE: keyboard::Key = keyboard::Key {
    key: "Delete",
    code: "Delete",
    vk: 46,
    text: None,
};

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

/// Review 449: a click on a row's button left focus on the button, where the
/// arrow keys do nothing. The row takes focus back.
#[test]
fn the_arrows_work_after_a_click_on_a_row_button() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        pointer::click(page, "[data-tree-id='README.md'] button")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.dataset.treeId === 'README.md'",
            "the clicked row to hold focus",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.dataset.treeId === 'docs'",
            "Up to move to the row above",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("clicking a row").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The same for a link leaf, as the docs sidebar renders them.
#[test]
fn a_clicked_link_row_takes_focus() {
    block_on(async {
        let fixture = Fixture::open("/tree/links", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "[data-tree-id='/tree/links'] a")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.dataset.treeId === '/tree/links'",
            "the clicked link row to hold focus",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 516: APG's `*` opens every closed branch on the focused row's level,
/// and focus stays on the row.
#[test]
fn star_expands_the_sibling_branches() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, ROW, 5).await.unwrap();
        keyboard::press(page, keyboard::END).await.unwrap();
        keyboard::press(page, STAR).await.unwrap();
        wait::for_js_true(
            page,
            "['src', 'docs'].every(id => document.querySelector(`[data-tree-id='${id}']`)\
             ?.getAttribute('aria-expanded') === 'true')",
            "* to expand src and docs",
        )
        .await
        .unwrap();
        focus::assert_focused(page, "[data-tree-id='README.md']", "*")
            .await
            .unwrap();
        fixture.console.assert_clean("expanding with *").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 516: a focused row removed from `data` used to drop focus to
/// `<body>`. The row that took its place takes focus.
#[test]
fn removing_the_focused_row_keeps_focus_in_the_tree() {
    block_on(async {
        let fixture = Fixture::open("/tree/delete", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, ROW, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        focus::wait_for_focus(page, "[data-tree-id='b.rs']", "ArrowDown")
            .await
            .unwrap();
        keyboard::press(page, DELETE).await.unwrap();
        focus::wait_for_focus(page, "[data-tree-id='c.rs']", "deleting b.rs")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        focus::wait_for_focus(page, "[data-tree-id='a.rs']", "ArrowUp after the delete")
            .await
            .unwrap();
        fixture.console.assert_clean("deleting a row").unwrap();
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
