//! `Tree`: the `TreeWalk` archetype (310), not `RovingTabindex`: Right and Left open and
//! close rows, changing the set of rows.

use anyhow::{Result, ensure};
use e2e::archetypes::TreeWalk;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::{focus, keyboard, pointer};
use e2e::wait;
use e2e::{Fixture, Suite, Viewport};

const ROW: &str = "[role=treeitem]";
const BRANCH: &str = "[role=treeitem][aria-expanded]";
const CHEVRON: &str = "[role=treeitem][aria-expanded] [data-tree-chevron]";
const ROVING: &str = "[role=tree] [tabindex='0']";

/// The chevron turns by a state-driven `transform` as its branch expands.
async fn chevron_turns<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let collapsed = d.style(CHEVRON, "transform").await?;
    d.focus(ROVING).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "ArrowRight to expand src", async |d| {
        Ok(d.attr(BRANCH, "aria-expanded").await?.as_deref() == Some("true"))
    })
    .await?;
    eventually(d, &format!("the chevron to leave {collapsed}"), async |d| {
        Ok(d.style(CHEVRON, "transform").await? != collapsed)
    })
    .await
}

/// Enter on a leaf clicks the link it renders (todo 4).
async fn enter_clicks_the_link<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(ROVING).await?;
    eventually_focused(d, ROW, "focusing the tab stop").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Enter on Elsewhere to click its link", async |d| {
        Ok(d.text("#activated").await? == "elsewhere")
    })
    .await
}

e2e::scenario!(
    the_chevron_turns_when_the_branch_expands,
    "/tree/chevron",
    chevron_turns,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    enter_on_a_leaf_clicks_its_link,
    "/tree/activate",
    enter_clicks_the_link,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

const STAR: keyboard::Key = keyboard::Key {
    key: "*",
    code: "NumpadMultiply",
    vk: 106,
    text: Some("*"),
};
const O: keyboard::Key = keyboard::Key {
    key: "o",
    code: "KeyO",
    vk: 79,
    text: Some("o"),
};
const DELETE: keyboard::Key = keyboard::Key {
    key: "Delete",
    code: "Delete",
    vk: 46,
    text: None,
};

/// `/tree`: roots `src`, `docs` (collapsed) and leaf `README.md`. Declared here so a
/// fixture change fails rather than weakens `TreeWalk`.
pub(crate) const WALK: TreeWalk = TreeWalk {
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

/// Todo 1570: on the `<li>` the ring wrapped an open branch's whole subtree; it is
/// drawn on the row's own line.
#[test]
fn an_open_branch_rings_its_row_line_not_its_subtree() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, ROW, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.getAttribute('aria-expanded') === 'true'",
            "ArrowRight to open src",
        )
        .await
        .unwrap();
        let rings: String = page
            .evaluate(
                "(() => { const li = document.activeElement; \
                 const line = li.querySelector(':scope > [data-slot=content]'); \
                 return [li, line].map(el => { const s = getComputedStyle(el); \
                 return `${s.outlineStyle} ${s.boxShadow}`; }).join(' / '); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let (row, line) = rings.split_once(" / ").unwrap();
        assert_eq!(row, "none none", "the open branch's li is ringed: {rings}");
        assert!(
            line.starts_with("solid ") && !line.ends_with(" none"),
            "the row's line is not ringed: {rings}"
        );
        fixture
            .console
            .assert_clean("ringing an open branch")
            .unwrap();
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

async fn expanded_of<D: Driver>(d: &mut D, tree: &str, id: &str) -> Result<Option<String>> {
    d.attr(&format!("#{tree} [data-tree-id='{id}']"), "aria-expanded")
        .await
}

/// Todo 770: a controlled tree opens what the caller stores, from a key or
/// from outside without a remount, and only asks when the caller keeps its set.
async fn controlled<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let src = "#stored [data-tree-id='src']";
    d.focus(src).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "ArrowRight to open src", async |d| {
        Ok(expanded_of(d, "stored", "src").await?.as_deref() == Some("true"))
    })
    .await?;
    d.press(O).await?;
    eventually(d, "the caller to open docs", async |d| {
        Ok(expanded_of(d, "stored", "docs").await?.as_deref() == Some("true"))
    })
    .await?;
    eventually_focused(d, src, "opening docs from outside").await?;

    d.focus("#fixed [data-tree-id='src']").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "the request", async |d| {
        Ok(d.attr("#requests", "data-requests").await?.as_deref() == Some("1"))
    })
    .await?;
    ensure!(
        expanded_of(d, "fixed", "src").await?.as_deref() == Some("false"),
        "a request the caller did not store opened the branch"
    );
    Ok(())
}

e2e::scenario!(
    a_controlled_tree_follows_its_caller,
    "/tree/controlled",
    controlled,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

/// Todos 1135/1136: the guide runs under the branch's chevron on the inline-start
/// side, the current row's marker sits on it without moving its label, and the
/// trailing icon follows the label.
async fn guides_line_up<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    let rtl = route.ends_with("/rtl");
    let start = |r: &e2e::driver::Rect| if rtl { r.x + r.width } else { r.x };
    let group = d.rect("[role=group]").await?;
    let chevron = d.rect("[data-tree-id=src] [data-tree-chevron]").await?;
    let current = d.rect("[data-tree-id='src/lib.rs'] > div").await?;
    let label = d
        .rect("[data-tree-id='src/lib.rs'] [data-tree-label]")
        .await?;
    let sibling = d
        .rect("[data-tree-id='src/main.rs'] [data-tree-label]")
        .await?;
    let star = d
        .rect("[data-tree-id='src/lib.rs'] [data-tree-icon]:last-of-type")
        .await?;

    let side = if rtl { "right" } else { "left" };
    let guide = d
        .style("[role=group]", &format!("border-{side}-width"))
        .await?;
    let marker = d
        .style(
            "[data-tree-id='src/lib.rs'] > div",
            &format!("border-{side}-width"),
        )
        .await?;
    // Android snaps borders to device pixels: 1px reads 0.76px at a 2.625 ratio.
    let px = |value: &str| value.trim_end_matches("px").parse::<f64>().unwrap_or(0.0);
    ensure!(
        px(&guide) > 0.5 && px(&marker) > px(&guide),
        "guide {guide}, marker {marker}"
    );

    let sign = if rtl { -1.0 } else { 1.0 };
    let guide_mid = start(&group) + sign * 0.5;
    let chevron_mid = chevron.x + chevron.width / 2.0;
    ensure!(
        (guide_mid - chevron_mid).abs() <= 1.0,
        "guide at {guide_mid}, chevron centre at {chevron_mid}"
    );
    ensure!(
        (start(&current) - start(&group)).abs() <= 0.5,
        "marker starts at {current:?}, guide at {group:?}"
    );
    ensure!(
        (start(&label) - start(&sibling)).abs() <= 0.5,
        "the current label moved: {label:?} vs {sibling:?}"
    );
    let after = if rtl {
        star.x + star.width <= label.x
    } else {
        star.x >= label.x + label.width
    };
    ensure!(after, "trailing icon {star:?}, label {label:?}");
    Ok(())
}

e2e::scenario!(
    guides_line_up_under_the_chevron,
    "/tree/guides",
    guides_line_up
);
e2e::scenario!(
    guides_line_up_under_the_chevron_rtl,
    "/tree/guides/rtl",
    guides_line_up
);

#[test]
fn it_walks_like_a_tree() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        WALK.assert_contract(&fixture.page).await.unwrap();
        fixture.console.assert_clean("walking the tree").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 596: a disabled row takes the pointer and shows `not-allowed`, but a
/// press neither focuses it, off the roving stop, nor reaches its `TreeItem`.
#[test]
fn a_disabled_row_shows_not_allowed_and_takes_no_focus() {
    block_on(async {
        let fixture = Fixture::open("/chip/fieldset", Viewport::Desktop)
            .await
            .unwrap();
        let row = "[data-tree-id=\"off\"]";
        crate::action_icon::assert_disabled_look(&fixture.page, row).await;

        pointer::click(&fixture.page, row).await.unwrap();
        let focused: String = fixture
            .page
            .evaluate("document.activeElement.dataset.treeId ?? document.activeElement.id")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            !["off", "tree-off"].contains(&focused.as_str()),
            "a press focused the disabled row: {focused}"
        );
        fixture.close().await.unwrap();
    });
}
