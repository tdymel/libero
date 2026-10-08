//! `Kanban`: cards reorder in a column by drag, keys or buttons, and move across
//! columns by their Move to menu (1216).

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::suite::Step;

const STATUS: &str = "[role=status]";
const START: &str = "Alpha Beta Gamma | Delta | ";

async fn a_drag_reorders_a_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (alpha, beta) = (d.rect("#Alpha").await?, d.rect("#Beta").await?);
    d.drag("#Alpha [data-slot=handle]", 0.0, (beta.y - alpha.y) * 2.0)
        .await?;
    eventually_text(
        d,
        "#order",
        "Beta Gamma Alpha | Delta | ",
        "a drag two slots down",
    )
    .await?;
    ensure!(
        d.text("#moves").await? == "0.0>0.2 ",
        "one move in column 0"
    );
    eventually_focused(d, "#Alpha [data-slot=handle]", "a drag").await?;
    Ok(())
}

/// `card`'s handle dragged by the gap between column 0 and `column`, and `slots` cards down.
async fn drag_across<D: Driver>(d: &mut D, card: &str, column: usize, slots: f64) -> Result<()> {
    let (from, to) = (
        d.rect("#column-0").await?,
        d.rect(&format!("#column-{column}")).await?,
    );
    let (alpha, beta) = (d.rect("#Alpha").await?, d.rect("#Beta").await?);
    let handle = format!("#{card} [data-slot=handle]");
    d.drag(&handle, to.x - from.x, (beta.y - alpha.y) * slots)
        .await
}

async fn a_drag_moves_a_card_across<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // Beta, second in To do, level with the slot below Delta in Doing.
    drag_across(d, "Beta", 1, 0.0).await?;
    eventually_text(
        d,
        "#order",
        "Alpha Gamma | Delta Beta | ",
        "a drag to Doing",
    )
    .await?;
    ensure!(
        d.text("#moves").await? == "0.1>1.1 ",
        "one move to column 1"
    );
    eventually_text(
        d,
        STATUS,
        "Beta moved to Doing, position 2 of 2.",
        "the drop across",
    )
    .await?;
    eventually_focused(d, "#Beta [data-slot=handle]", "a drag across").await?;

    // Gamma, now second in To do, two slots up: first in Doing.
    drag_across(d, "Gamma", 1, -2.0).await?;
    eventually_text(
        d,
        "#order",
        "Alpha | Gamma Delta Beta | ",
        "a drag above Delta",
    )
    .await?;
    Ok(())
}

async fn a_drag_fills_an_empty_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    drag_across(d, "Gamma", 2, -2.0).await?;
    eventually_text(d, "#order", "Alpha Beta | Delta | Gamma", "a drag to Done").await?;
    eventually_text(
        d,
        STATUS,
        "Gamma moved to Done, position 1 of 1.",
        "the drop in an empty column",
    )
    .await?;
    eventually_focused(d, "#Gamma [data-slot=handle]", "a drag to Done").await?;
    Ok(())
}

async fn a_drop_off_the_board_puts_it_back<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (alpha, board) = (d.rect("#Alpha").await?, d.rect("#board").await?);
    d.drag(
        "#Alpha [data-slot=handle]",
        0.0,
        board.y + board.height - alpha.y + alpha.height * 2.0,
    )
    .await?;
    eventually_text(
        d,
        STATUS,
        "Cancelled. Alpha back at position 1 of 3.",
        "a drop below the board",
    )
    .await?;
    d.settle().await?;
    ensure!(
        d.text("#order").await? == START,
        "a drop off the board moved a card"
    );
    ensure!(d.text("#moves").await?.is_empty(), "a move was reported");
    Ok(())
}

async fn the_keyboard_reorders_a_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#Beta [data-slot=handle]").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Beta, position 2 of 3.", "Space to lift").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, STATUS, "Beta moved to position 1 of 3.", "ArrowUp").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#order", "Beta Alpha Gamma | Delta | ", "Space to drop").await?;
    eventually_text(d, STATUS, "Dropped Beta at position 1 of 3.", "the drop").await?;
    eventually_focused(d, "#Beta [data-slot=handle]", "a keyboard drop").await?;
    Ok(())
}

async fn escape_puts_it_back<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#Gamma [data-slot=handle]").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Gamma, position 3 of 3.", "Space to lift").await?;
    d.press(keyboard::HOME).await?;
    d.press(keyboard::ESCAPE).await?;
    eventually_text(
        d,
        STATUS,
        "Cancelled. Gamma back at position 3 of 3.",
        "Escape to cancel",
    )
    .await?;
    d.settle().await?;
    ensure!(d.text("#order").await? == START, "a cancel reordered");
    ensure!(d.text("#moves").await?.is_empty(), "a move was reported");
    Ok(())
}

async fn the_move_buttons_reorder_a_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.attr("#Delta [data-slot=move-earlier]", "disabled")
            .await?
            .is_some()
            && d.attr("#Delta [data-slot=move-later]", "disabled")
                .await?
                .is_some(),
        "a lone card's move buttons are enabled"
    );
    d.click("#Alpha [data-slot=move-later]").await?;
    eventually_text(d, "#order", "Beta Alpha Gamma | Delta | ", "Move down").await?;
    eventually_focused(d, "#Alpha [data-slot=move-later]", "a move down").await?;
    ensure!(
        d.text(STATUS).await? == "Alpha moved to position 2 of 3.",
        "the move went unsaid"
    );
    Ok(())
}

async fn the_menu_moves_a_card_across<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // ArrowUp opens the menu on its last item: Done.
    d.focus("#Beta [data-slot=move-to]").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_focused(d, "[role=menuitem]", "ArrowUp on the trigger").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#order", "Alpha Gamma | Delta | Beta", "Move to Done").await?;
    ensure!(
        d.text("#moves").await? == "0.1>2.0 ",
        "one move to column 2"
    );
    eventually_text(
        d,
        STATUS,
        "Beta moved to Done, position 1 of 1.",
        "the move across",
    )
    .await?;
    eventually_focused(d, "#Beta [data-slot=move-to]", "a move across").await?;

    // Back to To do: it lands last there.
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_focused(d, "[role=menuitem]", "ArrowDown on the trigger").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#order", "Alpha Gamma Beta | Delta | ", "Move to To do").await?;
    eventually_text(
        d,
        STATUS,
        "Beta moved to To do, position 3 of 3.",
        "the move back",
    )
    .await?;
    eventually_focused(d, "#Beta [data-slot=move-to]", "a move back").await?;
    Ok(())
}

async fn the_own_column_is_disabled<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#Delta [data-slot=move-to]").await?;
    eventually_focused(d, "[role=menuitem]", "a click on the trigger").await?;
    ensure!(
        d.exists("[role=menuitem]:nth-child(3)").await?
            && !d.exists("[role=menuitem]:nth-child(4)").await?,
        "not one menu item per column"
    );
    let doing = d
        .attr("[role=menuitem]:nth-child(2)", "aria-disabled")
        .await?;
    ensure!(
        doing.as_deref() == Some("true"),
        "the card's own column is enabled: {doing:?}"
    );
    d.press(keyboard::ESCAPE).await?;
    eventually_focused(d, "#Delta [data-slot=move-to]", "Escape").await?;
    ensure!(d.text("#order").await? == START, "Escape moved a card");
    Ok(())
}

/// 2519, 2580: a refused move is void once the app moves its card by data, whether
/// that renders the board or only a column.
async fn a_refused_move_is_void_after_a_data_change<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    d.click("#refuse").await?;
    d.focus("#Beta [data-slot=move-to]").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_focused(d, "[role=menuitem]", "ArrowUp on the trigger").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#moves", "0.1>2.0 ", "the refused move to Done").await?;
    d.focus("#remove").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#order", "Alpha Gamma | Delta | ", "Beta removed").await?;
    d.focus("#add").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#order", "Alpha Gamma | Delta | Omega", "Omega added").await?;
    d.settle().await?;
    ensure!(
        d.is_focused("#add").await?,
        "a card added at the refused slot took the focus: {}",
        d.focus_owner().await?
    );
    Ok(())
}

/// 2517: with Alpha hidden, To do shows indices 1 and 2; a drag and Move to still land.
async fn a_filtered_column_drags_and_takes_a_card_at_its_end<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    d.click("#hide").await?;
    eventually_text(d, "#order", START, "Alpha hidden, not removed").await?;
    let (beta, gamma) = (d.rect("#Beta").await?, d.rect("#Gamma").await?);
    d.drag("#Gamma [data-slot=handle]", 0.0, beta.y - gamma.y)
        .await?;
    eventually_text(
        d,
        "#order",
        "Alpha Gamma Beta | Delta | ",
        "a drag one slot up",
    )
    .await?;
    ensure!(
        d.text("#moves").await? == "0.2>0.1 ",
        "the move speaks the data's indices"
    );
    d.focus("#Delta [data-slot=move-to]").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_focused(d, "[role=menuitem]", "ArrowDown on the trigger").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#order", "Alpha Gamma Beta Delta |  | ", "Move to To do").await?;
    eventually_text(
        d,
        STATUS,
        "Delta moved to To do, position 3 of 3.",
        "the move into the filtered column",
    )
    .await?;
    eventually_focused(d, "#Delta [data-slot=move-to]", "a move to the end").await?;
    Ok(())
}

/// 2579: with Alpha hidden, To do shows indices 1 and 2; keys and buttons go by the shown cards.
async fn a_filtered_column_moves_by_keys_and_buttons<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    d.click("#hide").await?;
    eventually(d, "Beta first of the shown cards", async |d| {
        let earlier = d.attr("#Beta [data-slot=move-earlier]", "disabled").await?;
        Ok(earlier.is_some())
    })
    .await?;
    d.focus("#Gamma [data-slot=handle]").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Gamma, position 2 of 2.", "Space to lift").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, STATUS, "Gamma moved to position 1 of 2.", "ArrowUp").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#order", "Alpha Gamma Beta | Delta | ", "Space to drop").await?;
    eventually_focused(d, "#Gamma [data-slot=handle]", "a keyboard drop").await?;
    ensure!(
        d.text("#moves").await? == "0.2>0.1 ",
        "the key move speaks the data's indices"
    );

    d.click("#Gamma [data-slot=move-later]").await?;
    eventually_text(d, "#order", "Alpha Beta Gamma | Delta | ", "Move down").await?;
    eventually_focused(d, "#Gamma [data-slot=move-earlier]", "a move down").await?;
    eventually_text(d, STATUS, "Gamma moved to position 2 of 2.", "Move down").await?;
    ensure!(
        d.text("#moves").await? == "0.2>0.1 0.1>0.2 ",
        "the button move speaks the data's indices"
    );
    Ok(())
}

/// 2635: To do shows Alpha, Beta and Zeta with Gamma hidden between the last two.
async fn gamma_hidden_before_zeta<D: Driver>(d: &mut D) -> Result<()> {
    d.click("#grow").await?;
    d.click("#hide-gamma").await?;
    eventually(d, "Gamma hidden, Zeta shown", async |d| {
        Ok(d.exists("#Zeta").await? && !d.exists("#Gamma").await?)
    })
    .await
}

/// Alpha passes Beta and lands before Zeta, behind the hidden Gamma, as a drag would.
async fn alpha_landed_before_zeta<D: Driver>(d: &mut D, how: &str) -> Result<()> {
    eventually_text(d, "#order", "Beta Gamma Alpha Zeta | Delta | ", how).await?;
    ensure!(
        d.text("#moves").await? == "0.0>0.2 ",
        "{how} speaks the data's indices"
    );
    Ok(())
}

async fn a_filtered_key_move_lands_before_the_next_shown_card<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    gamma_hidden_before_zeta(d).await?;
    d.focus("#Alpha [data-slot=handle]").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Alpha, position 1 of 3.", "Space to lift").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, STATUS, "Alpha moved to position 2 of 3.", "ArrowDown").await?;
    d.press(keyboard::SPACE).await?;
    alpha_landed_before_zeta(d, "the key move").await
}

async fn a_filtered_button_move_lands_before_the_next_shown_card<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    gamma_hidden_before_zeta(d).await?;
    d.click("#Alpha [data-slot=move-later]").await?;
    alpha_landed_before_zeta(d, "the button move").await
}

async fn a_filtered_drag_lands_before_the_next_shown_card<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    gamma_hidden_before_zeta(d).await?;
    let (alpha, beta) = (d.rect("#Alpha").await?, d.rect("#Beta").await?);
    d.drag("#Alpha [data-slot=handle]", 0.0, beta.y - alpha.y)
        .await?;
    alpha_landed_before_zeta(d, "the drag").await
}

/// 2451: a move the app ignores arms no focus jump for the next card at its slot.
async fn a_refused_move_moves_no_focus_later<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#refuse").await?;
    d.focus("#Beta [data-slot=move-to]").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_focused(d, "[role=menuitem]", "ArrowUp on the trigger").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#moves", "0.1>2.0 ", "the refused move to Done").await?;
    ensure!(d.text("#order").await? == START, "a refused move moved");
    d.focus("#add").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(
        d,
        "#order",
        "Alpha Beta Gamma | Delta | Omega",
        "Omega added",
    )
    .await?;
    d.settle().await?;
    ensure!(
        d.is_focused("#add").await?,
        "a card added at the refused slot took the focus: {}",
        d.focus_owner().await?
    );
    Ok(())
}

e2e::scenario!(
    a_move_the_app_refuses_leaves_no_focus_jump_for_a_later_card,
    "/kanban",
    a_refused_move_moves_no_focus_later,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_refused_move_is_void_once_the_app_removes_its_card,
    "/kanban/data",
    a_refused_move_is_void_after_a_data_change,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_refused_move_is_void_when_the_data_change_renders_only_a_column,
    "/kanban/columns",
    a_refused_move_is_void_after_a_data_change,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_filtered_column_moves_by_keys_and_buttons_over_the_shown_cards,
    "/kanban/data",
    a_filtered_column_moves_by_keys_and_buttons,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_filtered_key_move_lands_before_the_next_shown_card_behind_a_hidden_one,
    "/kanban/data",
    a_filtered_key_move_lands_before_the_next_shown_card,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_filtered_button_move_lands_before_the_next_shown_card_behind_a_hidden_one,
    "/kanban/data",
    a_filtered_button_move_lands_before_the_next_shown_card,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_filtered_drag_lands_before_the_next_shown_card_behind_a_hidden_one,
    "/kanban/data",
    a_filtered_drag_lands_before_the_next_shown_card,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_filtered_column_still_drags_and_move_to_lands_after_its_last_card,
    "/kanban/data",
    a_filtered_column_drags_and_takes_a_card_at_its_end,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_handle_drag_reorders_a_column_and_keeps_the_handle_focused,
    "/kanban",
    a_drag_reorders_a_column
);
e2e::scenario!(
    a_handle_drag_moves_a_card_into_another_column_at_the_slot_it_was_let_go,
    "/kanban",
    a_drag_moves_a_card_across
);
e2e::scenario!(
    a_handle_drag_moves_a_card_into_an_empty_column,
    "/kanban",
    a_drag_fills_an_empty_column
);
e2e::scenario!(
    a_handle_drag_let_go_off_the_board_puts_the_card_back_and_says_so,
    "/kanban",
    a_drop_off_the_board_puts_it_back
);
e2e::scenario!(
    space_lifts_an_arrow_moves_and_space_drops_in_a_column_each_step_announced,
    "/kanban",
    the_keyboard_reorders_a_column
);
e2e::scenario!(
    escape_puts_a_lifted_card_back_and_says_so,
    "/kanban",
    escape_puts_it_back
);
e2e::scenario!(
    the_move_buttons_move_a_card_one_slot_in_its_column,
    "/kanban",
    the_move_buttons_reorder_a_column
);
e2e::scenario!(
    the_move_to_menu_moves_a_card_to_the_end_of_another_column_and_keeps_the_focus,
    "/kanban",
    the_menu_moves_a_card_across,
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    the_move_to_menu_lists_every_column_with_the_cards_own_disabled,
    "/kanban",
    the_own_column_is_disabled,
    desktop: skip("958: element identity on the WebView")
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("kanban", "/kanban")
        .focusable("#Alpha [data-slot=handle]")
        .targets("#Beta [data-slot=handle]")
        .targets("#Beta [data-slot=move-earlier]")
        .targets("#Beta [data-slot=move-later]")
        .targets("#Beta [data-slot=move-to]")
        .state(
            "menu_open",
            &[
                Step::TabTo("#Alpha [data-slot=move-to]"),
                Step::Press(keyboard::ARROW_DOWN),
            ],
            "[role=menu]",
        )
        .state(
            "lifted",
            &[
                Step::TabTo("#Alpha [data-slot=handle]"),
                Step::Press(keyboard::SPACE),
            ],
            "#Alpha[data-state~=dragging]",
        )
        .run();
}

/// At a 220px column the move buttons wrap below the content together, end-aligned;
/// the content keeps 8rem or more.
#[test]
fn a_narrow_column_wraps_the_move_buttons_below_the_content() {
    use e2e::browser::{Fixture, Viewport, block_on};
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Mobile).await.unwrap();
        let measured: String = fixture
            .page
            .evaluate(
                "(() => { const r = s => document.querySelector('#Beta ' + s).getBoundingClientRect(); \
                 return [document.querySelector('#column-0').getBoundingClientRect().width, \
                 r('[data-slot=content]').width, r('[data-slot=handle]').bottom, \
                 r('[data-slot=move-earlier]').top, r('[data-slot=move-later]').top, \
                 r('[data-slot=move-to]').top, \
                 document.querySelector('#Beta').getBoundingClientRect().right - r('[data-slot=move-to]').right] \
                 .map(Math.round).join(' '); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [
            column,
            content,
            handle_bottom,
            earlier,
            later,
            move_to,
            end_gap,
        ] = measured
            .split(' ')
            .map(|n| n.parse::<i64>().unwrap())
            .collect::<Vec<_>>()[..]
        else {
            panic!("{measured}")
        };
        assert!(column <= 240, "the column is not narrow: {measured}");
        assert!(content >= 128, "the content is under 8rem: {measured}");
        assert!(
            earlier >= handle_bottom && earlier == later && (later - move_to).abs() <= 2,
            "the buttons did not wrap together: {measured}"
        );
        assert!(end_gap <= 12, "the buttons are not end-aligned: {measured}");
        // Room for 8rem and the buttons: one line.
        let one_line: bool = fixture
            .page
            .evaluate(
                "(() => { const c = document.querySelector('#Beta'); c.parentElement.style.width = '420px'; \
                 const r = s => c.querySelector(s).getBoundingClientRect(); \
                 return ['move-earlier', 'move-later', 'move-to'].every(s => r(`[data-slot=${s}]`).top < r('[data-slot=handle]').bottom); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(one_line, "a wide card wrapped its buttons");
        fixture.close().await.unwrap();
    });
}

/// The client rect's centre of `selector`'s first match, on `page`.
async fn centre(page: &chromiumoxide::Page, selector: &str) -> e2e::passes::pointer::Point {
    e2e::passes::pointer::centre_of(page, selector)
        .await
        .unwrap()
}

/// One left-button mouse event at `at`, the button held when `held`.
async fn mouse_at(
    page: &chromiumoxide::Page,
    kind: chromiumoxide::cdp::browser_protocol::input::DispatchMouseEventType,
    at: e2e::passes::pointer::Point,
    held: bool,
) {
    use chromiumoxide::cdp::browser_protocol::input::{DispatchMouseEventParams, MouseButton};
    let event = DispatchMouseEventParams::builder()
        .r#type(kind)
        .x(at.x)
        .y(at.y)
        .button(MouseButton::Left)
        .buttons(i64::from(held))
        .click_count(1)
        .build()
        .unwrap();
    page.execute(event).await.unwrap();
}

/// The board's auto-scroll tick (libero's kanban `AUTO_SCROLL_MS`).
const AUTO_SCROLL_MS: u32 = 40;

/// Runs held auto-scroll ticks, each with the scroll read it spawned, until `done` or
/// `most` ticks, in the page: a round trip per tick took longer than the real clock.
async fn ticks(page: &chromiumoxide::Page, done: &str, most: u32) -> bool {
    page.evaluate(format!(
        "(async () => {{
            for (let n = 0; !({done}) && n < {most}; n++) {{
                const armed = window.__heldClock.fireAll({AUTO_SCROLL_MS});
                if (armed !== 1) throw new Error(`${{armed}} auto-scroll ticks armed`);
                await new Promise((r) => setTimeout(() => setTimeout(r, 0), 0));
            }}
            return {done};
        }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// A card held still at the board's end edge scrolls the board, and drops in the column it brought in (1364).
#[test]
fn a_drag_held_at_the_edge_scrolls_the_board_to_a_hidden_column() {
    drag_held_at_the_end_edge(false);
}

/// Right to left the end edge is the left one, and `scrollLeft` runs negative (1436).
#[test]
fn a_drag_held_at_the_left_edge_scrolls_a_right_to_left_board() {
    drag_held_at_the_end_edge(true);
}

fn drag_held_at_the_end_edge(rtl: bool) {
    use chromiumoxide::cdp::browser_protocol::input::DispatchMouseEventType as Kind;
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::frames::in_front;
    use e2e::passes::pointer::Point;
    use e2e::wait;
    // The board's end side; Done wholly past it, and scrolled in past it.
    let (end, hidden, past) = match rtl {
        true => ("left", "d.right <= b.left", "d.left > b.left + 2"),
        false => ("right", "d.left >= b.right", "d.right < b.right - 2"),
    };
    let done = |test: &str| {
        format!(
            "(() => {{ const b = document.querySelector('#board').getBoundingClientRect(); \
             const d = document.querySelector('#column-2').getBoundingClientRect(); \
             return {test}; }})()"
        )
    };
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        if rtl {
            page.evaluate("document.documentElement.dir = 'rtl'")
                .await
                .unwrap();
        }
        e2e::clock::hold(page, &[AUTO_SCROLL_MS]).await.unwrap();
        let hidden: bool = page
            .evaluate(done(hidden))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(hidden, "Done starts in view: nothing to scroll to");
        let side: f64 = page
            .evaluate(format!(
                "document.querySelector('#board').getBoundingClientRect().{end}"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let (handle, alpha) = (
            centre(page, "#Gamma [data-slot=handle]").await,
            centre(page, "#Alpha [data-slot=handle]").await,
        );
        let edge = Point {
            x: if rtl { side + 8.0 } else { side - 8.0 },
            y: alpha.y,
        };
        // In front: a tab behind holds each move for its next frame, about 1 s.
        in_front(page, async {
            mouse_at(page, Kind::MouseMoved, handle, false).await;
            mouse_at(page, Kind::MousePressed, handle, true).await;
            for step in 1..=10 {
                let t = f64::from(step) / 10.0;
                let at = Point {
                    x: handle.x + (edge.x - handle.x) * t,
                    y: handle.y + (edge.y - handle.y) * t,
                };
                mouse_at(page, Kind::MouseMoved, at, true).await;
            }
            Ok(())
        })
        .await
        .unwrap();
        // Held still: only the ticks scroll, up to Done's end edge and no further.
        let at_end = done(&format!("Math.abs(d.{end} - b.{end}) < 2"));
        assert!(
            ticks(page, &at_end, 200).await,
            "a card held at the end edge did not scroll the board to its end"
        );
        // A few more ticks, held at the edge.
        ticks(page, "false", 8).await;
        let past: bool = page
            .evaluate(done(past))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            !past,
            "the board scrolled past its end into the dragged card's overflow"
        );
        let done = centre(page, "#column-2").await;
        let over = Point {
            x: done.x,
            y: alpha.y,
        };
        in_front(page, async {
            mouse_at(page, Kind::MouseMoved, over, true).await;
            mouse_at(page, Kind::MouseReleased, over, false).await;
            Ok(())
        })
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#order').textContent === 'Alpha Beta | Delta | Gamma'",
            "a drop in the column the scroll brought in",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("an edge drag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The Move to menu, keyboard only, scrolls the board to a hidden column it moves the card to.
#[test]
fn a_menu_move_to_a_hidden_column_scrolls_it_into_view() {
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::wait;
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        page.evaluate("document.querySelector('#Beta [data-slot=move-to]').focus()")
            .await
            .unwrap();
        // ArrowUp opens the menu on its last item: Done.
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.getAttribute('role') === 'menuitem'",
            "ArrowUp on the Move to trigger",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const b = document.querySelector('#board').getBoundingClientRect(); \
             const c = document.querySelector('#Beta').getBoundingClientRect(); \
             return document.querySelector('#order').textContent === 'Alpha Gamma | Delta | Beta' \
             && c.left >= b.left - 1 && c.right <= b.right + 1; })()",
            "Beta moved to Done and in view",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A finger on a handle drags the card to the next column; on a card's text it scrolls the board.
#[test]
fn a_touch_on_the_handle_drags_across_and_elsewhere_scrolls_the_board() {
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::frames::in_front;
    use e2e::passes::pointer::{Point, touch_drag};
    use e2e::wait;
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        let handle = centre(page, "#Beta [data-slot=handle]").await;
        let (from, to) = (
            centre(page, "#column-0").await,
            centre(page, "#column-1").await,
        );
        let at = Point {
            x: handle.x + (to.x - from.x),
            y: handle.y,
        };
        // In front: a tab behind holds each touch event for its next frame, about 1 s.
        in_front(page, touch_drag(page, handle, at, 12))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#order').textContent === 'Alpha Gamma | Delta Beta | '",
            "a touch drag on the handle to move Beta to Doing",
        )
        .await
        .unwrap();

        // From the text's end edge leftwards, inside the viewport.
        let text = centre(page, "#Alpha [data-slot=content]").await;
        let (start, end) = (
            Point {
                x: text.x + 60.0,
                y: text.y,
            },
            Point {
                x: text.x - 90.0,
                y: text.y,
            },
        );
        in_front(page, touch_drag(page, start, end, 12))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#board').scrollLeft > 50",
            "a swipe on a card's text to scroll the board",
        )
        .await
        .unwrap();
        let order: String = page
            .evaluate("document.querySelector('#order').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(order, "Alpha Gamma | Delta Beta | ", "a swipe moved a card");
        fixture.console.assert_clean("touch drags").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 2452, 2516: a touch screen's handle description points to the move buttons and the
/// Move to menu.
#[test]
fn a_touch_screen_describes_the_handle_by_the_move_buttons() {
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::wait;
    const DESCRIPTION: &str = "document.getElementById(document.querySelector('#Alpha [data-slot=handle]')\
         .getAttribute('aria-describedby')).textContent";
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let outcome = async {
            e2e::browser::fake_pointer(page).await?;
            wait::for_visible(page, "#Alpha").await?;
            wait::for_js_true(
                page,
                &format!("{DESCRIPTION}.startsWith('Press Space')"),
                "the keyboard description",
            )
            .await?;
            e2e::browser::set_coarse_pointer(page, true).await?;
            wait::for_js_true(
                page,
                &format!(
                    "{DESCRIPTION} === 'Use the move buttons to reorder the card, or the Move to \
                     column button to put it in another column.'"
                ),
                "the touch description",
            )
            .await
        }
        .await;
        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// 2454: a card dragged below the columns adds no vertical scrollbar, so the board keeps its width.
#[test]
fn a_card_dragged_below_the_columns_keeps_the_board_width() {
    use e2e::browser::{Fixture, Viewport, block_on};
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Desktop).await.unwrap();
        drag_below_the_columns(fixture).await;
    });
}

/// 2518: as above where the page draws a classic scrollbar, which headless Chromium hides.
#[test]
fn a_card_dragged_below_the_columns_keeps_the_board_width_with_a_classic_scrollbar() {
    use e2e::browser::{Fixture, Viewport, block_on};
    block_on(async {
        let fixture = Fixture::open_with_scrollbars("/kanban", Viewport::Desktop)
            .await
            .unwrap();
        drag_below_the_columns(fixture).await;
    });
}

async fn drag_below_the_columns(fixture: e2e::browser::Fixture) {
    use chromiumoxide::cdp::browser_protocol::input::DispatchMouseEventType as Kind;
    use e2e::frames::in_front;
    use e2e::passes::pointer::Point;
    use e2e::wait;
    const WIDTH: &str = "document.querySelector('#board').clientWidth";
    let page = &fixture.page;
    let width: f64 = page.evaluate(WIDTH).await.unwrap().into_value().unwrap();
    let bottom: f64 = page
        .evaluate("document.querySelector('#board').getBoundingClientRect().bottom")
        .await
        .unwrap()
        .into_value()
        .unwrap();
    let handle = centre(page, "#Gamma [data-slot=handle]").await;
    let below = Point {
        x: handle.x,
        y: bottom + 80.0,
    };
    in_front(page, async {
        mouse_at(page, Kind::MouseMoved, handle, false).await;
        mouse_at(page, Kind::MousePressed, handle, true).await;
        for step in 1..=10 {
            let t = f64::from(step) / 10.0;
            let at = Point {
                x: handle.x,
                y: handle.y + (below.y - handle.y) * t,
            };
            mouse_at(page, Kind::MouseMoved, at, true).await;
        }
        Ok(())
    })
    .await
    .unwrap();
    wait::for_js_true(
        page,
        "document.querySelector('#Gamma')?.dataset.state?.includes('dragging')",
        "Gamma lifted",
    )
    .await
    .unwrap();
    let held: (f64, String, f64) = page
        .evaluate(format!(
            "(() => {{ const b = document.querySelector('#board'); \
             return [{WIDTH}, getComputedStyle(b).overflowY, b.scrollTop]; }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(
        held,
        (width, "hidden".to_string(), 0.0),
        "the drag changed the board"
    );
    in_front(page, async {
        mouse_at(page, Kind::MouseReleased, below, false).await;
        Ok(())
    })
    .await
    .unwrap();
    fixture.close().await.unwrap();
}

/// A card is a bordered paper surface apart from its column's fill, light and dark (1437).
#[test]
fn a_card_stands_apart_from_its_column_in_both_schemes() {
    use e2e::browser::{Fixture, Scheme, Viewport, block_on, emulate_media};
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for scheme in [Scheme::Light, Scheme::Dark] {
            emulate_media(page, scheme, None).await.unwrap();
            e2e::wait::for_js_true(
                page,
                &format!(
                    "matchMedia('(prefers-color-scheme: {})').matches",
                    scheme.name()
                ),
                "the scheme to apply",
            )
            .await
            .unwrap();
            let [card, column, border]: [String; 3] = page
                .evaluate(
                    "(() => { const card = getComputedStyle(document.querySelector('#Alpha')); \
                     const column = getComputedStyle(document.querySelector('#column-0')); \
                     return [card.backgroundColor, column.backgroundColor, \
                             card.borderTopWidth + ' ' + card.borderTopStyle + ' ' + card.borderTopColor]; })()",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            let name = scheme.name();
            assert!(
                card != column,
                "{name}: the card shares its column's {card}"
            );
            assert!(!card.contains("rgba(0, 0, 0, 0)"), "{name}: a clear card");
            // The edge parts the card from the column it sits on (todo 1775).
            assert!(
                border.starts_with("1px solid ") && border != format!("1px solid {column}"),
                "{name}: no visible edge, {border} on the column's {column}"
            );
        }
        fixture.close().await.unwrap();
    });
}

/// A second drag still slides the neighbours before the drop, as the first did (1438).
#[test]
fn a_second_drag_still_slides_the_neighbours() {
    use e2e::archetypes::{Round, second_drag};
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::passes::pointer;
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let outcome = async {
            e2e::wait::for_visible(page, "#Gamma").await?;
            let (beta, gamma) = (
                pointer::centre_of(page, "#Beta").await?,
                pointer::centre_of(page, "#Gamma").await?,
            );
            let rounds = [
                Round {
                    handle: "#Alpha [data-slot=handle]",
                    passed: "#Beta",
                    order: "Beta Alpha Gamma | Delta |",
                },
                Round {
                    handle: "#Beta [data-slot=handle]",
                    passed: "#Alpha",
                    order: "Alpha Beta Gamma | Delta |",
                },
            ];
            second_drag(page, &rounds, "#Gamma", gamma.y - beta.y).await
        }
        .await;
        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// Forced colours paint the column's fill as `Canvas`, so only a border draws its edge.
#[test]
fn a_column_keeps_an_edge_in_forced_colours() {
    use e2e::browser::{Fixture, Viewport, block_on};
    block_on(async {
        let fixture = Fixture::open("/kanban", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        e2e::browser::force_colours(page).await.unwrap();
        let edge: String = page
            .evaluate(
                "(() => { const s = getComputedStyle(document.querySelector('#column-0')); \
                 return s.borderTopWidth + ' ' + s.borderTopStyle + ' ' + s.borderTopColor; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            edge.starts_with("1px solid ") && !edge.contains("rgba(0, 0, 0, 0)"),
            "{edge}"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 2018: the board measures at the press, so a finger's drag moves the card soon.
mod a_touch_drag_moves_the_card_soon_on_a_slow_phone {
    #[cfg(feature = "android")]
    #[test]
    fn android() {
        e2e::android::block_on(async {
            e2e::driver::Android::drag_starts_without_a_read(
                "/kanban",
                "#Alpha [data-slot=handle]",
                (0.0, 140.0),
            )
            .await
            .unwrap();
        });
    }
}
