//! `Checkbox`: APG's checkbox pattern, in every variant its docs page shows.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, SPACE};
use e2e::passes::pointer;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport};

#[test]
fn it_meets_the_baseline() {
    Suite::new("checkbox", "/checkbox")
        .focusable("#terms")
        .focusable("#support")
        // The control of a checkbox named by `aria_label` alone: an 18px box,
        // so it conforms through the spacing exception. The control, not the
        // box, so its own visually hidden input does not count as a neighbour.
        .targets_spaced("span:has(> #bare)")
        .state(
            "ticked",
            &[Step::TabTo("#terms"), Step::Press(SPACE)],
            "#terms:checked",
        )
        .run();
}

/// The value the last `onchange` carried, as `<id>:<value>`.
async fn emitted(fixture: &Fixture) -> String {
    fixture
        .page
        .evaluate("document.querySelector('[data-emitted]').dataset.emitted")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

#[test]
fn space_and_clicks_toggle_and_a_mixed_box_turns_true() {
    block_on(async {
        let fixture = Fixture::open("/checkbox", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#all", 10).await.unwrap();
        keyboard::press(page, SPACE).await.unwrap();
        assert_eq!(emitted(&fixture).await, "all:true");

        keyboard::tab_to(page, "#locked", 10).await.unwrap();
        keyboard::press(page, SPACE).await.unwrap();
        assert_eq!(emitted(&fixture).await, "all:true", "a readonly box moved");

        pointer::click(page, "#locked + [aria-hidden]")
            .await
            .unwrap();
        pointer::click(page, "#off + [aria-hidden]").await.unwrap();
        assert_eq!(
            emitted(&fixture).await,
            "all:true",
            "a click moved a locked box"
        );

        pointer::click(page, "#support-description").await.unwrap();
        assert_eq!(
            emitted(&fixture).await,
            "support:true",
            "the card took no click"
        );

        fixture.console.assert_clean("checkbox keys").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A disabled box used to keep its own `cursor: pointer`, over the control's
/// `not-allowed`, so the one part a user aims at promised a click.
#[test]
fn a_disabled_box_shows_it_takes_no_click() {
    block_on(async {
        let fixture = Fixture::open("/checkbox", Viewport::Desktop).await.unwrap();
        let cursor = |id: &str| {
            format!("getComputedStyle(document.querySelector('#{id} + [aria-hidden]')).cursor")
        };
        for (id, expected) in [("off", "not-allowed"), ("terms", "pointer")] {
            let actual: String = fixture
                .page
                .evaluate(cursor(id))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(actual, expected, "the cursor over #{id}'s box");
        }
        fixture.close().await.unwrap();
    });
}
