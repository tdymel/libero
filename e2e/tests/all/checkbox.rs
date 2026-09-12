//! `Checkbox`: APG's checkbox pattern, in every variant its docs page shows.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, ENTER, SPACE};
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
        .evaluate("document.activeElement.id")
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

        // A native checkbox ignores `aria-checked`; the mixed state has to
        // reach the accessibility tree through the `indeterminate` property.
        let all = e2e::ax::snapshot(page, "#all").await.unwrap();
        assert!(all.contains("[checked=mixed]"), "{all}");

        keyboard::tab_to(page, "#all", 10).await.unwrap();
        keyboard::press(page, SPACE).await.unwrap();
        assert_eq!(emitted(&fixture).await, "all:true");
        settle(&fixture).await;
        let all = e2e::ax::snapshot(page, "#all").await.unwrap();
        assert!(all.contains("[checked]"), "{all}");

        // APG: Space alone toggles a checkbox.
        keyboard::tab_to(page, "#terms", 10).await.unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        settle(&fixture).await;
        assert_eq!(emitted(&fixture).await, "all:true", "Enter toggled");

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

/// A native checkbox takes focus from a click, and its blur is what shows the
/// rules; the visually hidden input has to be handed it by hand.
#[test]
fn a_click_on_the_box_focuses_the_input() {
    block_on(async {
        let fixture = Fixture::open("/checkbox", Viewport::Desktop).await.unwrap();

        for (target, expected, input) in [
            ("#bare + [aria-hidden]", "bare:true", "bare"),
            ("label[for=terms]", "terms:true", "terms"),
            ("#support + [aria-hidden]", "support:true", "support"),
        ] {
            pointer::click(&fixture.page, target).await.unwrap();
            settle(&fixture).await;
            assert_eq!(emitted(&fixture).await, expected, "click on {target}");
            assert_eq!(focused(&fixture).await, input, "focus after {target}");
        }

        fixture.console.assert_clean("checkbox clicks").unwrap();
        fixture.close().await.unwrap();
    });
}
