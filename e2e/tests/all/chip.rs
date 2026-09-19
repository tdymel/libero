//! `Chip`: APG's checkbox pattern for a filter chip, plus the tag, button and
//! link kinds.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, linger};
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
            // What it draws: the input itself is `opacity: 0` (todo 757).
            "#css > input:checked + *",
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

async fn emits<D: Driver>(d: &mut D, expected: &str) -> Result<()> {
    eventually(d, &format!("the chip to emit {expected}"), async |d| {
        Ok(d.attr("[data-emitted]", "data-emitted").await?.as_deref() == Some(expected))
    })
    .await
}

/// A filter chip is a hidden checkbox beside its `<label>`: Blitz toggles the
/// input and reports `input`, not `click`, the shape `Switch` had.
async fn a_click_on_the_label_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#css label").await?;
    emits(d, "css:true").await?;
    d.click("#css label").await?;
    emits(d, "css:false").await
}

async fn space_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#css > input").await?;
    d.press(SPACE).await?;
    emits(d, "css:true").await?;
    d.press(SPACE).await?;
    emits(d, "css:false").await
}

e2e::scenario!(
    a_click_on_the_label_toggles_it,
    "/chip",
    a_click_on_the_label_toggles
);
/// Todo 941: Blitz placed the label's `::after` hit area against the label, so
/// the pill's padding took no click natively.
async fn the_padding_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let pill = d.rect("#css").await?;
    let middle = pill.y + pill.height / 2.0;
    d.click_at(pill.x + 3.0, middle).await?;
    emits(d, "css:true").await?;
    eventually_focused(d, "#css > input", "a click on the padding").await?;
    d.click_at(pill.x + pill.width - 3.0, middle).await?;
    emits(d, "css:false").await
}

/// The remove x is a control of its own: the padding's click leaves it alone.
async fn the_remove_x_does_not_toggle<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#remove").await?;
    emits(d, "remove").await?;
    linger(d, 3).await;
    emits(d, "remove").await
}

e2e::scenario!(space_toggles_the_focused_chip, "/chip", space_toggles);
e2e::scenario!(
    the_remove_x_leaves_the_chip_as_it_is,
    "/chip/removable",
    the_remove_x_does_not_toggle
);
e2e::scenario!(
    a_click_on_the_pills_padding_toggles_it,
    "/chip",
    the_padding_toggles
);

/// Todo 491: a selected chip showed only a faint tint.
#[test]
fn a_selected_chip_shows_the_on_state_ring() {
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
        crate::button::assert_on_in_forced_colours(page, "#removable-filled", "#plain").await;
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

/// Todo 661: a control in an `onclick`/`to` chip's `trailing` is nested
/// interactive content, and a debug build says so for each chip. A count there
/// is fine and stays quiet.
#[test]
fn a_control_in_a_clickable_chips_trailing_warns() {
    block_on(async {
        let fixture = Fixture::open("/chip/trailing-badge", Viewport::Desktop)
            .await
            .unwrap();
        settle(&fixture).await;
        fixture.console.assert_clean("a count in trailing").unwrap();
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/chip/trailing-control", Viewport::Desktop)
            .await
            .unwrap();
        let warned = || {
            fixture
                .console
                .peek()
                .iter()
                .filter(|message| message.contains("Chip: `trailing` holds a control"))
                .count()
        };
        e2e::wait::until("both chips to warn", || async { Ok(warned() == 2) })
            .await
            .unwrap();
        fixture.console.drain();
        fixture.close().await.unwrap();
    });
}

/// Todo 629: a readonly chip keeps its tab stop and says it is read only, but
/// Space and clicks leave it as it is, and in a form it still posts.
#[test]
fn a_readonly_chip_is_focusable_but_does_not_toggle_and_still_posts() {
    block_on(async {
        let fixture = Fixture::open("/chip/readonly", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        for id in ["locked", "locked-form"] {
            let input = format!("#{id} > input");
            keyboard::tab_to(page, &input, 10).await.unwrap();
            assert_eq!(focused(&fixture).await, id, "#{id} is not a tab stop");
            keyboard::press(page, SPACE).await.unwrap();
            pointer::click(page, &format!("#{id} label")).await.unwrap();
            settle(&fixture).await;
            assert_eq!(emitted(&fixture).await, "", "#{id} toggled");
        }

        let state: String = page
            .evaluate(
                "(() => { const form = document.getElementById('chip-form'); \
                 const inputs = [...form.querySelectorAll('input')]; \
                 return inputs.map(i => `${i.checked} ${i.getAttribute('aria-readonly')}`).join(',') \
                 + '|' + new FormData(form).getAll('tags').join(','); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            state, "true true,true true|rust",
            "[checked aria-readonly],..|posted"
        );

        fixture.console.assert_clean("readonly chips").unwrap();
        fixture.close().await.unwrap();
    });
}
