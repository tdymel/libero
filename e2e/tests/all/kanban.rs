//! `Kanban`: cards reorder in a column by drag, keys or buttons, and move across
//! columns by their Move to menu (1216).

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually_focused, eventually_text, linger};
use e2e::passes::keyboard;

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
    linger(d, 4).await;
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

e2e::scenario!(
    a_handle_drag_reorders_a_column_and_keeps_the_handle_focused,
    "/kanban",
    a_drag_reorders_a_column
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
    the_menu_moves_a_card_across
);
e2e::scenario!(
    the_move_to_menu_lists_every_column_with_the_cards_own_disabled,
    "/kanban",
    the_own_column_is_disabled
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("kanban", "/kanban")
        .focusable("#Alpha [data-slot=handle]")
        .run();
}
