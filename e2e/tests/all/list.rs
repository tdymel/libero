//! `List`: a nested list takes the indent of the list it sits in.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

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
