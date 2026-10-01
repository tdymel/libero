//! `Checkbox`: APG's checkbox pattern, in every variant its docs page shows.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard::{self, ENTER, SPACE};
use e2e::passes::{focus, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport};

#[test]
fn it_meets_the_baseline() {
    Suite::new("checkbox", "/checkbox")
        .focusable("#terms")
        .focusable("#support")
        // An `aria_label`-only checkbox's 18px control passes by the spacing exception;
        // the control, so its own hidden input is no neighbour.
        .targets_spaced("span:has(> #bare)")
        .state(
            "ticked",
            &[Step::TabTo("#terms"), Step::Press(SPACE)],
            // The box beside it: the input itself is `opacity: 0` (todo 757).
            "#terms:checked + *",
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

/// WCAG 1.4.11: the outline is all an unchecked box shows (todo 490).
#[test]
fn an_unchecked_box_parts_from_the_page() {
    crate::boundary::assert_boundaries(
        "/checkbox",
        "const box = document.querySelector('#terms + [aria-hidden]');
         return [['unchecked outline on the page', RATIO(CSS(box, 'borderTopColor'), PAGE(box))]];",
    );
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
            focus::wait_for_focus(
                &fixture.page,
                &format!("#{input}"),
                &format!("a click on {target}"),
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("checkbox clicks").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn emits<D: Driver>(d: &mut D, expected: &str, after: &str) -> Result<()> {
    eventually(d, &format!("{expected} emitted after {after}"), async |d| {
        Ok(d.attr("[data-emitted]", "data-emitted").await?.as_deref() == Some(expected))
    })
    .await
}

async fn a_box_click_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const BOX: &str = "#terms + [aria-hidden=true]";
    d.click(BOX).await?;
    emits(d, "terms:true", "a click on the box").await?;
    d.click(BOX).await?;
    emits(d, "terms:false", "a second click").await
}

async fn space_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#terms").await?;
    eventually_focused(d, "#terms", "focus").await?;
    d.press(SPACE).await?;
    emits(d, "terms:true", "Space").await?;
    d.press(SPACE).await?;
    emits(d, "terms:false", "a second Space").await
}

e2e::scenario!(
    a_click_on_the_box_toggles_a_checkbox,
    "/checkbox",
    a_box_click_toggles
);
e2e::scenario!(space_toggles_a_focused_checkbox, "/checkbox", space_toggles);
