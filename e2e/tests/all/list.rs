//! `List`: a nested list takes the indent of the list it sits in.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, ax, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("list", "/list").run();
}

/// Todo 2923: the icon floats at the start edge, and a long word wraps in the body beside
/// it instead of dropping below or running out of a 160px column.
#[test]
fn an_icon_floats_and_a_long_word_wraps_beside_it() {
    block_on(async {
        let fixture = Fixture::open("/list", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#long-item").await.unwrap();

        const READ: &str = "(() => { const li = document.querySelector('#long-item'); \
             const icon = li.querySelector('[data-slot=icon]').getBoundingClientRect(); \
             const body = li.querySelector('[data-slot=body]').getBoundingClientRect(); \
             const column = li.parentElement.getBoundingClientRect(); \
             return [getComputedStyle(li.querySelector('[data-slot=icon]')).float, \
                     body.left >= icon.right, body.top < icon.bottom, body.right <= column.right + 1].join(); })()";
        let ltr: String = page
            .evaluate_bounded(READ)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(ltr, "left,true,true,true");

        page.evaluate_bounded("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        let float = "getComputedStyle(document.querySelector('#long-item [data-slot=icon]')).float";
        wait::for_js_true(page, &format!("{float} === 'right'"), "the icon to flip")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4147: the action takes the item's end, and the body wraps beside it, in either direction.
#[test]
fn an_action_sits_at_the_items_end_beside_the_body() {
    block_on(async {
        for (dir, flip) in [("ltr", 1.0), ("rtl", -1.0)] {
            let fixture = Fixture::open("/list/action", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            page.evaluate_bounded(format!("document.documentElement.dir = '{dir}'"))
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("getComputedStyle(document.body).direction === '{dir}'"),
                "the page to take its direction",
            )
            .await
            .unwrap();
            // Body, then action, and whether each sits inside the item.
            let read: Vec<f64> = page
                .evaluate_bounded(
                    "['#acted', '#plain-acted'].flatMap(id => { \
                     const li = document.querySelector(id); const box = li.getBoundingClientRect(); \
                     const body = li.querySelector(':scope > [data-slot=body]').getBoundingClientRect(); \
                     const action = li.querySelector(':scope > [data-slot=action]').getBoundingClientRect(); \
                     const mid = r => (r.left + r.right) / 2; \
                     return [Math.sign(mid(action) - mid(body)), \
                             action.left >= box.left - 1 && action.right <= box.right + 1 \
                               && body.left >= box.left - 1 && body.right <= box.right + 1 ? 1 : 0, \
                             body.right <= action.left + 1 || body.left >= action.right - 1 ? 1 : 0]; })",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(
                read,
                [flip, 1.0, 1.0, flip, 1.0, 1.0],
                "{dir}: action beside the body"
            );
            fixture.console.assert_clean("the action fixture").unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 2468: the indent follows the nearest parent `List`, and a raw `ul` keeps its own.
#[test]
fn a_nested_list_is_indented_by_its_parent_list_only() {
    block_on(async {
        let fixture = Fixture::open("/list", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#inner").await.unwrap();

        let indents: Vec<String> = page
            .evaluate_bounded(
                "['#outer', '#middle', '#inner', '#raw', '#iconed'].map(id => \
                   getComputedStyle(document.querySelector(id)).paddingInlineStart)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        // An `xs` parent indents 8px, an `lg` one 20px; the raw `ul` keeps the browser's 40px.
        assert_eq!(indents, ["0px", "8px", "20px", "40px", "8px"]);
        fixture.console.assert_clean("the list fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A `ListSubheader` is no numbered item: a block flush with the ordered list's start edge,
/// while its items keep the number gutter.
#[test]
fn a_subheader_sits_outside_the_number_gutter() {
    block_on(async {
        let fixture = Fixture::open("/list/subheader", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#subheader").await.unwrap();
        let read: Vec<String> = page
            .evaluate_bounded(
                "(() => { const [list, head, item] = ['#ordered', '#subheader', '#first'] \
                   .map(id => document.querySelector(id)); \
                 const left = e => Math.round(e.getBoundingClientRect().left); \
                 return [getComputedStyle(head).display, String(left(head) === left(list)), \
                         String(left(item) > left(list))]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(read, ["block", "true", "true"]);
        // Todo 4172: the flex action item still counts.
        let numbers = ax::snapshot(page, "#ordered").await.unwrap();
        assert!(
            numbers.contains("ListMarker \"3.\""),
            "the item after an action item lost its number:\n{numbers}"
        );
        // Todo 4213: counting down, the item before the action item is 3.
        let numbers = ax::snapshot(page, "#reversed").await.unwrap();
        assert!(
            numbers.contains("ListMarker \"3.\"") && numbers.contains("ListMarker \"1.\""),
            "a reversed list skipped its action item:\n{numbers}"
        );
        fixture
            .console
            .assert_clean("the subheader fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Each plain item's number from its inline `list-style-type`, in DOM order.
async fn markers<D: Driver>(d: &mut D) -> Result<Vec<String>> {
    Ok(d.attrs("#keyed > li", "style")
        .await?
        .iter()
        .map(|style| style.split('"').nth(1).unwrap_or_default().to_string())
        .collect())
}

/// Todo 4401: a keyed insert and a reversal renumber in DOM order, not mount order, on Blitz.
async fn a_keyed_insert_renumbers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#insert").await?;
    eventually(d, "three items", async |d| Ok(markers(d).await?.len() == 3)).await?;
    eventually(d, "1, 2, 3 in DOM order after the insert", async |d| {
        Ok(markers(d).await? == ["1. ", "2. ", "3. "])
    })
    .await?;
    d.click("#swap").await?;
    eventually(d, "1, 2, 3 in DOM order after the swap", async |d| {
        Ok(markers(d).await? == ["1. ", "2. ", "3. "])
    })
    .await
}

e2e::scenario!(
    a_keyed_insert_renumbers_the_items_in_dom_order,
    "/list/keyed-insert",
    a_keyed_insert_renumbers,
    web: skip("the browser numbers the items itself; no inline marker to read")
);

/// Todo 4422: a keyed move made inside a child component renumbers in DOM order on Blitz.
async fn a_keyed_move_in_a_child_renumbers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "1, 2, 3 at first", async |d| {
        Ok(markers(d).await? == ["1. ", "2. ", "3. "])
    })
    .await?;
    d.click("#swap").await?;
    eventually(d, "C first after the swap", async |d| {
        Ok(d.text("#keyed > li").await?.trim().starts_with('C'))
    })
    .await?;
    eventually(d, "1, 2, 3 in DOM order after the swap", async |d| {
        Ok(markers(d).await? == ["1. ", "2. ", "3. "])
    })
    .await
}

e2e::scenario!(
    a_keyed_move_in_a_child_component_renumbers_in_dom_order,
    "/list/keyed-child",
    a_keyed_move_in_a_child_renumbers,
    web: skip("the browser numbers the items itself; no inline marker to read")
);
