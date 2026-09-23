//! `Sortable` and `use_sortable`: a handle drag reorders, a handle click stays a click (1095).

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text, linger};
use e2e::passes::keyboard;

/// The distance between two neighbours' tops (or lefts, in a row).
async fn pitch<D: Driver>(d: &mut D, horizontal: bool) -> Result<f64> {
    let (a, b) = (d.rect("#Alpha").await?, d.rect("#Beta").await?);
    Ok(if horizontal { b.x - a.x } else { b.y - a.y })
}

async fn a_drag_down_moves_it<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#before").await?;
    let pitch = pitch(d, false).await?;
    d.drag("#Alpha button", 0.0, pitch * 2.0).await?;
    eventually_text(
        d,
        "#order",
        "Beta Gamma Alpha Delta",
        "a drag two slots down",
    )
    .await?;
    ensure!(d.text("#moves").await? == "0>2 ", "one move expected");
    eventually_focused(d, "#Alpha button", "a drag").await?;
    // No offset is left behind: the item sits one slot below Gamma.
    let (alpha, gamma) = (d.rect("#Alpha").await?, d.rect("#Gamma").await?);
    ensure!(
        (alpha.y - gamma.y - pitch).abs() < 1.0,
        "Alpha at {}, Gamma at {}, pitch {pitch}",
        alpha.y,
        gamma.y
    );
    Ok(())
}

async fn a_drag_up_moves_it<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let pitch = pitch(d, false).await?;
    d.drag("#Delta button", 0.0, -pitch * 3.0).await?;
    eventually_text(d, "#order", "Delta Alpha Beta Gamma", "a drag to the top").await?;
    Ok(())
}

async fn a_short_drag_moves_nothing<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let pitch = pitch(d, false).await?;
    d.drag("#Beta button", 0.0, pitch * 0.3).await?;
    linger(d, 4).await;
    ensure!(
        d.text("#order").await? == "Alpha Beta Gamma Delta",
        "a drag short of the next middle reordered"
    );
    ensure!(d.text("#moves").await?.is_empty(), "a move was reported");
    Ok(())
}

async fn a_row_reorders_sideways<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (alpha, gamma) = (d.rect("#Alpha").await?, d.rect("#Gamma").await?);
    d.drag("#Alpha button", gamma.x - alpha.x, 0.0).await?;
    eventually_text(
        d,
        "#order",
        "Beta Gamma Alpha Delta",
        "a drag two slots right",
    )
    .await?;
    Ok(())
}

async fn a_click_stays_a_click<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#Beta .handle").await?;
    eventually_text(d, "#clicks", "1", "a click on the handle").await?;
    ensure!(
        d.text("#order").await? == "Alpha Beta Gamma Delta",
        "a click reordered"
    );
    let pitch = pitch(d, false).await?;
    d.drag("#Alpha .handle", 0.0, pitch * 2.0).await?;
    eventually_text(d, "#order", "Beta Gamma Alpha Delta", "a hook drag").await?;
    ensure!(
        d.text("#clicks").await? == "1",
        "the drag clicked the handle"
    );
    Ok(())
}

const STATUS: &str = "[role=status]";

async fn the_keyboard_moves_it<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#Alpha button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Alpha, position 1 of 4.", "Space to lift").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, STATUS, "Alpha moved to position 2 of 4.", "ArrowDown").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, STATUS, "Alpha moved to position 3 of 4.", "ArrowDown").await?;
    ensure!(
        d.text("#order").await? == "Alpha Beta Gamma Delta",
        "a keyboard move reordered before the drop"
    );
    d.press(keyboard::SPACE).await?;
    eventually_text(
        d,
        "#order",
        "Beta Gamma Alpha Delta",
        "Space to drop two slots down",
    )
    .await?;
    ensure!(
        d.text(STATUS).await? == "Dropped Alpha at position 3 of 4.",
        "the drop went unsaid"
    );
    eventually_focused(d, "#Alpha button", "a keyboard drop").await?;
    Ok(())
}

async fn escape_puts_it_back<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#Beta button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Beta, position 2 of 4.", "Space to lift").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, STATUS, "Beta moved to position 1 of 4.", "ArrowUp").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually_text(
        d,
        STATUS,
        "Cancelled. Beta back at position 2 of 4.",
        "Escape to cancel",
    )
    .await?;
    linger(d, 4).await;
    ensure!(
        d.text("#order").await? == "Alpha Beta Gamma Delta",
        "a cancelled move reordered"
    );
    ensure!(d.text("#moves").await?.is_empty(), "a move was reported");
    eventually_focused(d, "#Beta button", "a cancel").await?;
    Ok(())
}

async fn the_move_buttons_move_it<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.attr("#Alpha [data-slot=move-earlier]", "disabled")
            .await?
            .is_some(),
        "the first item's move up is enabled"
    );
    d.click("#Beta [data-slot=move-later]").await?;
    eventually_text(d, "#order", "Alpha Gamma Beta Delta", "Move down").await?;
    eventually_focused(d, "#Beta [data-slot=move-later]", "a move down").await?;
    // Now last: its move down turns disabled and hands the focus to move up.
    d.click("#Beta [data-slot=move-later]").await?;
    eventually_text(d, "#order", "Alpha Gamma Delta Beta", "a second Move down").await?;
    eventually_focused(d, "#Beta [data-slot=move-earlier]", "a move to the end").await?;
    ensure!(
        d.text(STATUS).await? == "Beta moved to position 4 of 4.",
        "the move went unsaid"
    );
    Ok(())
}

async fn a_row_moves_by_the_side_arrows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#Alpha button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(
        d,
        STATUS,
        "Lifted Item 1, position 1 of 4.",
        "Space to lift",
    )
    .await?;
    // A row's keys are the side arrows; ArrowDown moves nothing.
    d.press(keyboard::ARROW_DOWN).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(d, STATUS, "Item 1 moved to position 2 of 4.", "ArrowRight").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#order", "Beta Alpha Gamma Delta", "Enter to drop").await?;
    Ok(())
}

async fn a_dropped_item_settles_into_its_slot<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let pitch = pitch(d, false).await?;
    d.drag("#Alpha button", 0.0, pitch * 1.4).await?;
    eventually_text(
        d,
        "#order",
        "Beta Alpha Gamma Delta",
        "a drag one slot down",
    )
    .await?;
    let style = d.attr("#Alpha", "style").await?.unwrap_or_default();
    ensure!(
        style.contains("lsx-sortable-settle"),
        "no settle on the dropped item: {style}"
    );
    // The next lift ends it.
    d.focus("#Beta button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Beta, position 1 of 4.", "Space to lift").await?;
    let style = d.attr("#Alpha", "style").await?.unwrap_or_default();
    ensure!(!style.contains("animation"), "the settle stayed: {style}");
    d.press(keyboard::ESCAPE).await?;
    Ok(())
}

/// A swipe on an item's text scrolls the page: only the handle drags (touch-action).
#[cfg_attr(not(feature = "android"), allow(dead_code))]
async fn a_swipe_off_the_handle_scrolls<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let order = d.text("#order").await?;
    let before = d.rect("#Item12").await?.y;
    d.drag("#Item12-text", 0.0, -200.0).await?;
    eventually(d, "the swipe to scroll the page", async |d| {
        Ok(d.rect("#Item12").await?.y < before - 50.0)
    })
    .await?;
    linger(d, 4).await;
    ensure!(
        d.text("#order").await? == order,
        "a swipe on the text reordered"
    );
    Ok(())
}

/// A long touch drag on a handle, down a 24 item list.
async fn a_long_handle_drag_moves_it_far<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (a, b) = (d.rect("#Item2").await?, d.rect("#Item3").await?);
    let pitch = b.y - a.y;
    d.drag("#Item2 button", 0.0, pitch * 5.0).await?;
    eventually(d, "a drag five slots down", async |d| {
        Ok(d.text("#order")
            .await?
            .starts_with("Item1 Item3 Item4 Item5 Item6 Item7 Item2 "))
    })
    .await?;
    eventually_focused(d, "#Item2 button", "a long drag").await?;
    Ok(())
}

e2e::scenario!(
    a_handle_drag_down_moves_the_item_and_keeps_its_handle_focused,
    "/sortable",
    a_drag_down_moves_it
);
e2e::scenario!(
    a_handle_drag_up_moves_the_item_to_the_top,
    "/sortable",
    a_drag_up_moves_it
);
e2e::scenario!(
    a_drag_short_of_the_next_middle_moves_nothing,
    "/sortable",
    a_short_drag_moves_nothing
);
e2e::scenario!(
    a_horizontal_list_reorders_by_a_sideways_drag,
    "/sortable/horizontal",
    a_row_reorders_sideways
);
e2e::scenario!(
    a_click_on_a_hook_handle_clicks_and_a_drag_does_not,
    "/sortable/hook",
    a_click_stays_a_click
);
e2e::scenario!(
    space_lifts_the_arrows_move_and_space_drops_with_each_step_announced,
    "/sortable",
    the_keyboard_moves_it
);
e2e::scenario!(
    escape_puts_a_lifted_item_back_and_says_so,
    "/sortable",
    escape_puts_it_back
);
e2e::scenario!(
    the_move_buttons_move_an_item_one_slot_and_keep_the_focus,
    "/sortable",
    the_move_buttons_move_it
);
e2e::scenario!(
    a_row_moves_by_the_side_arrows_and_names_an_unlabelled_item_by_position,
    "/sortable/horizontal",
    a_row_moves_by_the_side_arrows
);
e2e::scenario!(
    a_dropped_item_settles_into_its_slot_until_the_next_lift,
    "/sortable",
    a_dropped_item_settles_into_its_slot
);
/// Touch only: a mouse drag on text selects it.
mod a_swipe_on_an_items_text_scrolls_the_page_and_moves_nothing {
    #[cfg(feature = "android")]
    #[test]
    fn android() {
        e2e::android::block_on(async {
            let route = "/sortable/long";
            let mut driver = e2e::driver::Android::open(route).await.unwrap();
            super::a_swipe_off_the_handle_scrolls(&mut driver, route)
                .await
                .unwrap();
            driver
                .finish("a_swipe_off_the_handle_scrolls")
                .await
                .unwrap();
        });
    }
}
e2e::scenario!(
    a_long_handle_drag_moves_an_item_five_slots_down_a_long_list,
    "/sortable/long",
    a_long_handle_drag_moves_it_far
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("sortable", "/sortable")
        .focusable("#Alpha button")
        .run();
}
