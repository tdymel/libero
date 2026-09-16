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

/// Todo 491: a selected chip showed only a faint tint.
#[test]
fn a_selected_chip_shows_the_on_state_line() {
    use crate::button::{
        assert_gray_in_forced_colours, assert_on_in_forced_colours, assert_on_marker,
    };
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/chip", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        assert_on_marker(page, "#rust", "#css").await;
        crate::calendar::force_colours(page).await;
        assert_on_in_forced_colours(page, "#rust", "#css").await;
        assert_gray_in_forced_colours(page, "#off").await;
        fixture.close().await.unwrap();
    });
}

/// Todo 646: the remove x inside a selected chip follows the label under forced colours.
#[test]
fn a_selected_chips_remove_x_follows_it_in_forced_colours() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/chip/removable", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        crate::button::assert_on_marker(page, "#removable", "#plain").await;
        crate::calendar::force_colours(page).await;
        crate::button::assert_on_in_forced_colours(page, "#removable", "#plain").await;
        fixture.close().await.unwrap();
    });
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

/// A disabled `Fieldset` disables a `<button>` natively; every button-rooted
/// control must dim with it rather than look clickable (todos 499, 514). The
/// opacity is the product up the tree, so a wrapper's dimming counts and a
/// double dimming shows.
#[test]
fn buttons_in_a_disabled_fieldset_look_disabled() {
    block_on(async {
        let fixture = Fixture::open("/chip/fieldset", Viewport::Desktop)
            .await
            .unwrap();
        let looks: Vec<String> = fixture
            .page
            .evaluate(
                "['#fs-chip', '#fs-button', '#fs-icon', '#fs-swatch', '#fs-tree-item', \
                 '#fs-phone button', '#fs-image button', '#tree-off'].map((id) => { \
                 const el = document.querySelector(id); let opacity = 1; \
                 for (let at = el; at; at = at.parentElement) \
                 opacity *= parseFloat(getComputedStyle(at).opacity); \
                 return `${id} ${el.matches(':disabled')} ${opacity} ${getComputedStyle(el).cursor}`; })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            looks,
            [
                "#fs-chip true 0.5 not-allowed",
                "#fs-button true 0.5 not-allowed",
                "#fs-icon true 0.5 not-allowed",
                "#fs-swatch true 0.5 not-allowed",
                "#fs-tree-item true 0.5 not-allowed",
                "#fs-phone button true 0.5 not-allowed",
                "#fs-image button true 0.5 not-allowed",
                "#tree-off true 0.5 not-allowed",
            ],
            "[id :disabled opacity cursor]"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 596: a disabled filter chip and a disabled link chip take the pointer
/// and show `not-allowed`, and a click on the filter chip still emits nothing.
#[test]
fn a_disabled_chip_shows_not_allowed_and_ignores_a_click() {
    block_on(async {
        let fixture = Fixture::open("/chip", Viewport::Desktop).await.unwrap();
        crate::action_icon::assert_disabled_look(&fixture.page, "#off").await;
        crate::action_icon::assert_disabled_look(&fixture.page, "#dead-link").await;

        pointer::click(&fixture.page, "#off").await.unwrap();
        settle(&fixture).await;
        assert_eq!(emitted(&fixture).await, "", "the disabled chip emitted");
        fixture.close().await.unwrap();
    });
}

/// Todo 674: 636's text span wrapped every child, so an icon among them lost
/// the chip's gap and centring. Both icon shapes keep them now.
#[test]
fn an_icon_keeps_the_gap_and_the_text_centre() {
    block_on(async {
        let fixture = Fixture::open("/chip/icons", Viewport::Desktop)
            .await
            .unwrap();
        let failures: Vec<serde_json::Value> = fixture
            .page
            .evaluate(
                "['in-children', 'in-prop'].flatMap(id => { \
                 const chip = document.getElementById(id); \
                 const icon = chip.querySelector('svg').getBoundingClientRect(); \
                 const range = document.createRange(); \
                 range.selectNodeContents([...chip.childNodes].find(n => n.nodeType === 3 && n.textContent.trim())); \
                 const text = range.getBoundingClientRect(); \
                 const box = chip.getBoundingClientRect(); \
                 const out = []; \
                 if (text.left - icon.right < 2) out.push([id, 'no gap', text.left - icon.right]); \
                 const mid = r => (r.top + r.bottom) / 2; \
                 if (Math.abs(mid(icon) - mid(box)) > 1.5) out.push([id, 'icon off centre', mid(icon), mid(box)]); \
                 if (Math.abs(mid(text) - mid(box)) > 1.5) out.push([id, 'text off centre', mid(text), mid(box)]); \
                 return out; })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(failures.is_empty(), "{failures:?}");
        fixture.close().await.unwrap();
    });
}

/// Todo 672: a `_blank` link chip draws Anchor's icon inside the pill and says
/// so in its name, even when its label is cut.
#[test]
fn a_new_tab_chip_shows_an_icon_and_says_so() {
    block_on(async {
        let fixture = Fixture::open("/chip/new-tab", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let tree = e2e::ax::snapshot(page, "#new-tab").await.unwrap();
        assert!(tree.contains("External (opens in a new tab)"), "{tree}");
        let drawn: bool = page
            .evaluate(
                "['#new-tab', '#new-tab-long'].every(id => { \
                 const svg = document.querySelector(id + ' svg').getBoundingClientRect(); \
                 const chip = document.querySelector(id).getBoundingClientRect(); \
                 return svg.width > 8 && svg.width < 16 && svg.right <= chip.right + 1 \
                     && svg.left >= chip.left && svg.top >= chip.top - 1 \
                     && svg.bottom <= chip.bottom + 1; })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(drawn, "the icon sits whole inside the chip, at text size");

        fixture.close().await.unwrap();
    });
}
