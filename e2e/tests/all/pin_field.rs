//! `PinField`: a label click focuses the first cell (483), modifier chords are the
//! browser's (509), a cell shows the pin however the character arrived (449).

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::InsertTextParams;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, ax, wait};

const CELL: &str = "[role=group] input";

async fn one_digit_per_cell<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(CELL).await?;
    d.type_text("12").await?;
    eventually_text(d, "#echo", "12", "typing 12").await?;
    eventually_focused(d, "[data-pin-index='2']", "two digits").await?;
    // The first press only moves back from the empty third cell.
    d.press(keyboard::BACKSPACE).await?;
    d.press(keyboard::BACKSPACE).await?;
    eventually_text(d, "#echo", "1", "two Backspaces").await?;
    eventually_focused(d, "[data-pin-index='0']", "two Backspaces").await
}

e2e::scenario!(
    a_pin_field_takes_one_digit_per_cell_and_moves_on,
    "/pin-field/echo",
    one_digit_per_cell
);

/// Todo 1564: a cell-to-cell move stays inside the field, so the rule waits until Tab
/// leaves the cells. Web only: a WebView cannot tell, and touches on any move.
#[test]
fn the_rule_waits_until_focus_leaves_the_cells() {
    const SHOWN: &str = "document.body.textContent.includes('Enter all four digits.') \
        || document.querySelector('[aria-invalid=true]') !== null";
    block_on(async {
        let fixture = Fixture::open("/pin-field/rule", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::passes::pointer::click(page, "[data-pin-index='0']")
            .await
            .unwrap();
        keyboard::type_text(page, "1").await.unwrap();
        e2e::passes::focus::wait_for_focus(page, "[data-pin-index='1']", "one digit")
            .await
            .unwrap();
        let shown = after_two_frames(page, SHOWN).await;
        assert!(!shown, "a move between cells showed the rule");

        // Each cell is a tab stop: two Tabs stay inside, the third reaches Next.
        for _ in 0..2 {
            keyboard::press(page, keyboard::TAB).await.unwrap();
        }
        e2e::passes::focus::wait_for_focus(page, "[data-pin-index='3']", "two Tabs")
            .await
            .unwrap();
        let shown = after_two_frames(page, SHOWN).await;
        assert!(!shown, "a Tab between cells showed the rule");
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(page, SHOWN, "the rule once Tab left the cells")
            .await
            .unwrap();
        fixture.console.assert_clean("a pin rule").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `js` once the render a move scheduled has landed: two frames on.
async fn after_two_frames(page: &Page, js: &str) -> bool {
    page.evaluate(format!(
        "new Promise(r => requestAnimationFrame(() => requestAnimationFrame(() => r({js}))))"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// The same on Blitz, whose `focusout` comes after the move and whose Tab fires none.
async fn the_rule_waits_for_focus_to_leave<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const SHOWN: &str = "[aria-invalid='true']";
    d.click("[data-pin-index='0']").await?;
    d.type_text("1").await?;
    eventually_focused(d, "[data-pin-index='1']", "one digit").await?;
    d.settle().await?;
    anyhow::ensure!(
        !d.exists(SHOWN).await?,
        "{:?}: a move between cells showed the rule",
        d.platform()
    );
    // Each cell is a tab stop: two Tabs stay inside, the third reaches Next.
    d.press(keyboard::TAB).await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "[data-pin-index='3']", "two Tabs").await?;
    d.settle().await?;
    anyhow::ensure!(
        !d.exists(SHOWN).await?,
        "{:?}: a Tab between cells showed the rule",
        d.platform()
    );
    d.press(keyboard::TAB).await?;
    eventually(d, "the rule once Tab left the cells", async |d| {
        d.exists(SHOWN).await
    })
    .await
}

e2e::scenario!(
    the_rule_waits_for_focus_to_leave_the_cells,
    "/pin-field/rule",
    the_rule_waits_for_focus_to_leave,
    web: skip("the_rule_waits_until_focus_leaves_the_cells checks it after two frames"),
    desktop: skip("1564: a WebView cannot tell a move inside the field from a blur"),
    android: skip("1564: a WebView cannot tell a move inside the field from a blur")
);

async fn cells_read<D: Driver>(d: &mut D, expected: [&str; 4], during: &str) -> Result<()> {
    eventually(
        d,
        &format!("cells {expected:?} after {during}"),
        async |d| {
            for (index, want) in expected.iter().enumerate() {
                if d.value(&format!("[data-pin-index='{index}']")).await? != *want {
                    return Ok(false);
                }
            }
            Ok(true)
        },
    )
    .await
}

/// Todo 1030: a soft keyboard commits text with no key named, so the cell's raw
/// text is put back even where the edit leaves the pin unchanged (590).
async fn retyped_text_without_a_key<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("[data-pin-index='0']").await?;
    d.insert_text("1").await?;
    eventually_text(d, "#echo", "1", "1 inserted").await?;
    eventually_focused(d, "[data-pin-index='1']", "1 inserted").await?;

    d.focus("[data-pin-index='0']").await?;
    d.insert_text("1").await?;
    cells_read(d, ["1", "", "", ""], "1 inserted again").await?;
    eventually_focused(d, "[data-pin-index='1']", "1 inserted again").await?;

    d.focus("[data-pin-index='0']").await?;
    d.insert_text("23").await?;
    cells_read(d, ["2", "3", "", ""], "23 inserted over the 1").await?;
    eventually_text(d, "#echo", "23", "23 inserted over the 1").await
}

e2e::scenario!(
    a_digit_retyped_without_a_key_leaves_no_drifted_cell,
    "/pin-field/echo",
    retyped_text_without_a_key,
    native: skip("no text without a key on Blitz"),
    desktop: skip("1126: no text without a key (no IME path) under xdotool")
);

/// Todo 507: every cell is named, so axe `label` holds.
#[test]
fn it_meets_the_baseline() {
    Suite::new("pin_field", "/pin-field/error")
        .focusable("[role=group] input[data-pin-index='0']")
        .focusable("[role=group] input[data-pin-index='3']")
        // The frame is the press target: a press on its padding focuses the cell.
        .targets("[role=group] > div")
        .run();
}

/// Todos 507 and 591: a focused cell names its place and carries the error
/// and helper, not only the group.
#[test]
fn every_cell_is_named_and_described() {
    block_on(async {
        let fixture = Fixture::open("/pin-field/error", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let checks: String = page
            .evaluate(
                "[...document.querySelectorAll('[role=group] input')].map((cell, i) => { \
                    const ids = (cell.getAttribute('aria-describedby') || '').split(' '); \
                    const text = ids.map(id => document.getElementById(id)?.textContent).join(' '); \
                    return cell.getAttribute('aria-label') === `Character ${i + 1} of 4` \
                        && text.includes('That code is wrong.') \
                        && text.includes('It expires in ten minutes.'); \
                }).join()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(checks, "true,true,true,true");

        fixture.console.assert_clean("pin field names").unwrap();
        fixture.close().await.unwrap();
    });
}
/// The cells' text joined by `|`, and the focused cell's index.
const CELLS: &str = "(() => { const c = [...document.querySelectorAll('[role=group] input')]; \
    return c.map(i => i.value).join('|') + ' @' + c.indexOf(document.activeElement); })()";

#[test]
fn a_click_on_the_label_focuses_the_first_cell() {
    crate::select::label_click_focuses("/pin-field", CELL);
}

/// Ctrl/Alt/Meta with an arrow, Home or End leave the focus on its cell; the
/// plain arrow still moves it.
#[test]
fn modifier_chords_leave_the_focus_on_its_cell() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let cell =
            "[...document.querySelectorAll('[role=group] input')].indexOf(document.activeElement)";
        page.evaluate("document.querySelectorAll('[role=group] input')[1].focus()")
            .await
            .unwrap();
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_LEFT,
                keyboard::ARROW_RIGHT,
                keyboard::HOME,
                keyboard::END,
            ],
            cell,
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{cell} === 2"),
            "ArrowRight to the next cell",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("pin field chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Focuses cell `index` with the caret at `caret`.
async fn focus_at(page: &Page, index: usize, caret: usize) {
    page.evaluate(format!(
        "(() => {{ const i = document.querySelectorAll('[role=group] input')[{index}]; \
         i.focus(); i.setSelectionRange({caret}, {caret}); }})()"
    ))
    .await
    .unwrap();
}

/// A digit key, pressed as a keyboard does: `keydown` first, then the text.
async fn press_digit(page: &Page, digit: &'static str) {
    let vk = 48 + digit.parse::<i64>().unwrap();
    let code = [
        "Digit0", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8",
        "Digit9",
    ][(vk - 48) as usize];
    let key = keyboard::Key {
        key: digit,
        code,
        vk,
        text: Some(digit),
    };
    keyboard::press(page, key).await.unwrap();
}

async fn expect_cells(page: &Page, expected: &str, what: &str) {
    wait::for_js_true(
        page,
        &format!("{CELLS} === {}", serde_json::to_string(expected).unwrap()),
        what,
    )
    .await
    .unwrap_or_else(|e| {
        panic!("{what}: {e}");
    });
}

/// A click on a cell's left half puts the caret before its digit: typing there
/// replaces the digit, and the cell holds one character.
#[test]
fn typing_before_a_digit_replaces_it() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1234").await.unwrap();
        expect_cells(page, "1|2|3|4 @3", "1234 typed").await;

        focus_at(page, 1, 0).await;
        press_digit(page, "7").await;
        expect_cells(page, "1|7|3|4 @2", "7 typed before the 2").await;

        fixture.console.assert_clean("pin field retype").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A paste into a filled cell replaces from that cell on, whichever side of
/// its digit the caret was.
#[test]
fn a_paste_into_a_filled_cell_replaces_from_there() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1234").await.unwrap();
        expect_cells(page, "1|2|3|4 @3", "1234 typed").await;

        focus_at(page, 1, 1).await;
        page.execute(InsertTextParams::new("98")).await.unwrap();
        expect_cells(page, "1|9|8|4 @3", "98 pasted after the 2").await;

        focus_at(page, 1, 0).await;
        page.execute(InsertTextParams::new("56")).await.unwrap();
        expect_cells(page, "1|5|6|4 @3", "56 pasted before the 9").await;

        fixture.console.assert_clean("pin field paste").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Holes are not representable: a character typed past the pin lands in the
/// first empty cell, which is what the cells show, and focus moves after it.
#[test]
fn typing_past_the_pin_lands_in_the_first_empty_cell() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1").await.unwrap();
        expect_cells(page, "1||| @1", "1 typed").await;

        focus_at(page, 3, 0).await;
        press_digit(page, "5").await;
        expect_cells(page, "1|5|| @2", "5 typed in the last cell").await;

        fixture
            .console
            .assert_clean("pin field past the end")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Text with no key named, as an Android soft keyboard sends it: a cell whose
/// value the edit leaves unchanged still shows the pin (todo 590).
#[test]
fn text_without_a_key_leaves_no_drifted_cell() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1").await.unwrap();
        expect_cells(page, "1||| @1", "1 typed").await;

        focus_at(page, 3, 0).await;
        page.execute(InsertTextParams::new("5")).await.unwrap();
        expect_cells(page, "1|5|| @2", "5 inserted in the last cell").await;

        focus_at(page, 0, 1).await;
        page.execute(InsertTextParams::new("1")).await.unwrap();
        expect_cells(page, "1|5|| @1", "1 inserted after the 1").await;

        fixture.console.assert_clean("pin field no key").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 1597, 1598, 1600: eight cells and seven dashes fit 288px, a reader
/// skips the dashes, and a phone neither capitalises nor corrects a letter.
#[test]
fn eight_letter_cells_fit_a_phone_and_read_as_cells_only() {
    block_on(async {
        let fixture = Fixture::open("/pin-field/wide", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelectorAll('[role=group] input').length === 8",
            "eight cells",
        )
        .await
        .unwrap();
        let checks: Vec<bool> = page
            .evaluate(
                "(() => { const group = document.querySelector('[role=group]'); \
                 const box = document.getElementById('box').getBoundingClientRect(); \
                 const cells = [...group.querySelectorAll('input')]; \
                 const dashes = [...group.children].filter(c => c.tagName === 'SPAN'); \
                 return [ \
                   group.scrollWidth <= group.clientWidth, \
                   [...group.children].every(c => c.getBoundingClientRect().right <= box.right + 0.5), \
                   dashes.length === 7 && dashes.every(d => d.getAttribute('aria-hidden') === 'true'), \
                   cells.every(c => c.getAttribute('autocapitalize') === 'off' \
                     && c.getAttribute('autocorrect') === 'off' && c.spellcheck === false), \
                 ]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            checks, [true; 4],
            "[no scroll, inside 288px, dashes hidden, no autocapitalize/correct/spellcheck]"
        );
        let tree = ax::snapshot(page, "[role=group]").await.unwrap();
        assert!(!tree.contains("\"-\""), "a dash is read:\n{tree}");

        fixture.console.assert_clean("pin field wide").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `aria-required` is not allowed on a group (axe `aria-allowed-attr`), so
/// the cells carry it, and each cell reports the error too.
#[test]
fn every_cell_reports_required_and_invalid() {
    block_on(async {
        let fixture = Fixture::open("/pin-field/error", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let tree = ax::snapshot(page, "[role=group]").await.unwrap();
        assert_eq!(
            tree.lines()
                .filter(|line| line.contains("textbox") && line.ends_with("[invalid] [required]"))
                .count(),
            4,
            "every cell required and invalid:\n{tree}"
        );
        let on_group: bool = page
            .evaluate("document.querySelector('[role=group]').hasAttribute('aria-required')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!on_group, "aria-required on the group");

        fixture.console.assert_clean("pin field error").unwrap();
        fixture.close().await.unwrap();
    });
}
