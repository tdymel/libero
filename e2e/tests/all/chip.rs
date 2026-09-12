//! `Chip`: APG's checkbox pattern for a filter chip, plus the tag, button and
//! link kinds.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, ENTER, SPACE};
use e2e::passes::pointer::{self, Point};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport};

#[test]
fn it_meets_the_baseline() {
    Suite::new("chip", "/chip")
        .focusable("#rust > input")
        .focusable("#action")
        .focusable("#link")
        .targets("#rust")
        .targets("#action")
        // An xs chip is 20px tall; a row of them conforms through the spacing.
        .targets_spaced("#small")
        .state(
            "ticked",
            &[Step::TabTo("#css > input"), Step::Press(SPACE)],
            "#css > input:checked",
        )
        .run();
}

/// The value the last `onchange` or `onclick` carried, as `<id>:<value>`.
async fn emitted(fixture: &Fixture) -> String {
    fixture
        .page
        .evaluate("document.querySelector('[data-emitted]').dataset.emitted")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn settle(fixture: &Fixture) {
    fixture
        .page
        .evaluate("new Promise(r => setTimeout(() => r(1), 100))")
        .await
        .unwrap();
}

async fn focused(fixture: &Fixture) -> String {
    fixture
        .page
        // The focused input's chip: its parent carries the fixture's id.
        .evaluate("document.activeElement.parentElement?.id ?? ''")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

#[test]
fn space_toggles_a_filter_chip_and_enter_does_not() {
    block_on(async {
        let fixture = Fixture::open("/chip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#css > input", 10).await.unwrap();
        keyboard::press(page, SPACE).await.unwrap();
        assert_eq!(emitted(&fixture).await, "css:true");
        settle(&fixture).await;
        let css = e2e::ax::snapshot(page, "#css").await.unwrap();
        assert!(css.contains("checkbox \"css\" [checked]"), "{css}");

        keyboard::tab_to(page, "#rust > input", 10).await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        settle(&fixture).await;
        assert_eq!(emitted(&fixture).await, "css:true", "Enter toggled");

        pointer::click(page, "#off label").await.unwrap();
        settle(&fixture).await;
        assert_eq!(emitted(&fixture).await, "css:true", "a disabled chip moved");

        fixture.console.assert_clean("chip keys").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The whole pill is the target, not only its label: the label used to cover
/// the text alone, so a click on the pill's padding did nothing.
#[test]
fn a_click_anywhere_on_the_pill_toggles_it() {
    block_on(async {
        let fixture = Fixture::open("/chip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        for (id, expected) in [("css", "css:true"), ("small", "small:true")] {
            let edge: Point = page
                .evaluate(format!(
                    "(() => {{ const r = document.getElementById('{id}').getBoundingClientRect(); \
                     return {{ x: r.left + 3, y: r.top + r.height / 2 }}; }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            pointer::drag(page, edge, edge, 0).await.unwrap();
            settle(&fixture).await;
            assert_eq!(emitted(&fixture).await, expected, "a click on #{id}'s edge");
            assert_eq!(focused(&fixture).await, id, "focus after a click on #{id}");
        }

        // A `name` alone keeps its own state.
        pointer::click(page, "#named").await.unwrap();
        settle(&fixture).await;
        let ticked: bool = page
            .evaluate("!!document.querySelector('#named > input:checked')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(ticked, "an uncontrolled chip did not tick");

        fixture.console.assert_clean("chip clicks").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A disabled link chip is an `<a>` without `href`, which is `generic`; it
/// keeps its link role, as a disabled link `Button` does.
#[test]
fn a_disabled_link_chip_is_still_a_link() {
    block_on(async {
        let fixture = Fixture::open("/chip", Viewport::Desktop).await.unwrap();
        let tree = e2e::ax::snapshot(&fixture.page, "#dead-link")
            .await
            .unwrap();
        assert!(tree.contains("link \"Dead link\" [disabled]"), "{tree}");
        fixture.close().await.unwrap();
    });
}
