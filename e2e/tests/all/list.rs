//! `List`: a nested list takes the indent of the list it sits in.

use e2e::browser::block_on;
use e2e::{Fixture, Suite, Viewport, wait};

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
        let ltr: String = page.evaluate(READ).await.unwrap().into_value().unwrap();
        assert_eq!(ltr, "left,true,true,true");

        page.evaluate("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        let float = "getComputedStyle(document.querySelector('#long-item [data-slot=icon]')).float";
        wait::for_js_true(page, &format!("{float} === 'right'"), "the icon to flip")
            .await
            .unwrap();
        fixture.close().await.unwrap();
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
            .evaluate(
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
