//! `Tree`: the component the archetype table *claims* to cover.
//!
//! Added deliberately to test a claim rather than a component. `Tree` is listed
//! under `RovingTabindex`, and that listing was written from APG without
//! checking. A tree is roving **and** hierarchical: it has expansion, levels,
//! and rows that appear and disappear. Whether the flat archetype survives that
//! is the question this unit answers.

use e2e::archetypes::{Orientation, RovingTabindex};
use e2e::browser::block_on;
use e2e::{Fixture, Suite, Viewport, passes::keyboard, wait};

const ROW: &str = "[role=treeitem]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("tree", "/tree")
        .focusable(ROW)
        .targets(ROW)
        .run();
}

/// The part of the roving contract a tree really does share: one tab stop, and
/// the vertical arrows move within it.
///
/// **Not** the full `assert_contract`. See the note on
/// `the_flat_archetype_does_not_cover_expansion` for what a tree needs that a
/// strip does not.
#[test]
fn it_is_a_single_tab_stop() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();

        let tabbable: usize = fixture
            .page
            .evaluate(
                "[...document.querySelectorAll('[role=treeitem]')]\
                 .filter(el => el.getAttribute('tabindex') !== '-1').length",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();

        assert_eq!(
            tabbable, 1,
            "a tree is one tab stop with the arrows moving inside it, found {tabbable}"
        );

        fixture.close().await.unwrap();
    });
}

/// What the flat archetype cannot express, recorded as a test so the gap is
/// visible rather than assumed.
///
/// APG's tree pattern adds four things on top of roving tabindex, none of which
/// `RovingTabindex` knows about:
///
/// * **Right** expands a collapsed row, or moves to its first child;
/// * **Left** collapses an expanded row, or moves to its parent;
/// * the set of navigable rows **changes** as rows expand and collapse, so
///   "item N of M" is not stable the way it is in a strip;
/// * `aria-level`, `aria-setsize` and `aria-posinset` describe a shape a flat
///   index cannot.
///
/// This asserts the first two, which is the minimum a tree owes a keyboard
/// user, and leaves the archetype honest about not covering them.
#[test]
fn the_flat_archetype_does_not_cover_expansion() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, ROW, 5).await.unwrap();

        let visible = "document.querySelectorAll('[role=treeitem]').length";
        let before: f64 = page.evaluate(visible).await.unwrap().into_value().unwrap();

        // Right on a collapsed parent expands it, so more rows become visible.
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(page, &format!("{visible} > {before}"), "the row to expand")
            .await
            .expect("ArrowRight should expand a collapsed row");

        // Left collapses it again.
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{visible} === {before}"),
            "the row to collapse",
        )
        .await
        .expect("ArrowLeft should collapse an expanded row");

        fixture
            .console
            .assert_clean("expanding and collapsing")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Kept as documentation of the boundary: the horizontal strip contract does
/// not apply to a tree, and running it would be asserting the wrong thing.
#[test]
#[ignore = "documents the archetype boundary; a tree is vertical and hierarchical"]
fn the_horizontal_strip_contract_is_not_a_tree() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        RovingTabindex {
            items: ROW,
            orientation: Orientation::Vertical,
            wraps: false,
        }
        .assert_contract(&fixture.page)
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
