//! `Sortable` and `use_sortable`: a handle drag reorders, a handle click stays a click (1095).

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

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
    d.settle().await?;
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
    d.settle().await?;
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

async fn an_rtl_row_drags_leftwards<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (alpha, gamma) = (d.rect("#Alpha").await?, d.rect("#Gamma").await?);
    ensure!(gamma.x < alpha.x, "Alpha is not the rightmost item");
    d.drag("#Alpha button", gamma.x - alpha.x, 0.0).await?;
    eventually_text(
        d,
        "#order",
        "Beta Gamma Alpha Delta",
        "a drag two slots left",
    )
    .await?;
    let (alpha, gamma) = (d.rect("#Alpha").await?, d.rect("#Gamma").await?);
    ensure!(
        alpha.x < gamma.x,
        "Alpha at {}, Gamma at {}",
        alpha.x,
        gamma.x
    );
    Ok(())
}

async fn an_rtl_row_moves_by_the_mirrored_arrows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#Alpha button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Alpha, position 1 of 4.", "Space to lift").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_text(d, STATUS, "Alpha moved to position 3 of 4.", "ArrowLeft").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#order", "Beta Gamma Alpha Delta", "Space to drop").await?;
    // The move forward button points the way the row runs.
    d.click("#Alpha [data-slot=move-later]").await?;
    eventually_text(d, "#order", "Beta Gamma Delta Alpha", "Move forward").await?;
    Ok(())
}

/// The neighbours slide aside while an item is lifted; the lifted one follows the pointer at once.
async fn neighbours_slide_the_lifted_item_does_not<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    ensure!(
        d.style("#Beta", "transition-duration").await? == "0s",
        "an idle item animates"
    );
    d.focus("#Alpha button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Alpha, position 1 of 4.", "Space to lift").await?;
    ensure!(
        d.style("#Beta", "transition-duration").await? == "0.15s",
        "a neighbour has no slide: {}",
        d.style("#Beta", "transition-duration").await?
    );
    ensure!(
        d.style("#Alpha", "transition-duration").await? == "0s",
        "the lifted item lags behind"
    );
    d.press(keyboard::ESCAPE).await?;
    Ok(())
}

/// A key past the list's end says so, and again on a repeat (a changed text).
async fn a_move_past_the_end_is_announced<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const AT_END: &str = "Delta is already at position 4 of 4.";
    d.focus("#Delta button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Delta, position 4 of 4.", "Space to lift").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, STATUS, AT_END, "ArrowDown on the last item").await?;
    d.press(keyboard::END).await?;
    eventually(d, "a repeat to change the region's text", async |d| {
        let text = d.text(STATUS).await?;
        Ok(text != AT_END && text.trim_end_matches('\u{200B}') == AT_END)
    })
    .await?;
    d.press(keyboard::HOME).await?;
    eventually_text(d, STATUS, "Delta moved to position 1 of 4.", "Home").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(
        d,
        STATUS,
        "Delta is already at position 1 of 4.",
        "ArrowUp at the top",
    )
    .await?;
    d.press(keyboard::ESCAPE).await?;
    Ok(())
}

/// A move `onreorder` ignored must not pull the focus back on a later, unrelated reorder.
async fn an_ignored_move_leaves_no_refocus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#Delta button").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, STATUS, "Lifted Delta, position 4 of 4.", "Space to lift").await?;
    d.press(keyboard::ARROW_UP).await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(
        d,
        STATUS,
        "Dropped Delta at position 3 of 4.",
        "Space to drop",
    )
    .await?;
    d.focus("#rotate").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#order", "Delta Alpha Beta Gamma", "the rotate").await?;
    d.settle().await?;
    eventually_focused(d, "#rotate", "a reorder after an ignored move").await?;
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
    d.settle().await?;
    ensure!(
        d.text("#order").await? == order,
        "a swipe on the text reordered"
    );
    Ok(())
}

/// A long touch drag on a handle, down a 40 item list.
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
    the_neighbours_of_a_lifted_item_slide_and_the_lifted_item_does_not,
    "/sortable",
    neighbours_slide_the_lifted_item_does_not
);
e2e::scenario!(
    a_right_to_left_row_reorders_by_a_leftward_drag,
    "/sortable/rtl",
    an_rtl_row_drags_leftwards
);
e2e::scenario!(
    a_right_to_left_row_moves_by_the_mirrored_arrows_and_buttons,
    "/sortable/rtl",
    an_rtl_row_moves_by_the_mirrored_arrows
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

e2e::scenario!(
    a_key_past_the_lists_end_announces_the_item_stays,
    "/sortable",
    a_move_past_the_end_is_announced
);
e2e::scenario!(
    a_move_onreorder_ignored_does_not_take_the_focus_on_a_later_reorder,
    "/sortable/ignored",
    an_ignored_move_leaves_no_refocus
);

/// 1.4.10: a row of four is wider than 320px; it scrolls inside itself, rings unclipped.
#[test]
fn a_row_scrolls_inside_itself_at_320px() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    block_on(async {
        let fixture = Fixture::open("/sortable/horizontal", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(SetDeviceMetricsOverrideParams::new(320, 640, 1.0, true))
            .await
            .unwrap();
        wait::for_visible(page, "#Alpha").await.unwrap();
        let widths: Vec<f64> = page
            .evaluate(
                "(() => {
                    const list = document.querySelector('#list');
                    const handle = document.querySelector('#Alpha button');
                    return [document.documentElement.scrollWidth, innerWidth,
                            list.scrollWidth, list.clientWidth,
                            handle.getBoundingClientRect().left - list.getBoundingClientRect().left];
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [page_width, viewport, scroll, client, inset] = widths[..] else {
            panic!("{widths:?}");
        };
        assert!(
            page_width <= viewport,
            "the page scrolls sideways: {widths:?}"
        );
        assert!(scroll > client, "the row does not scroll: {widths:?}");
        // The ring reaches offset + stripe + halo, 6px by default.
        assert!(
            inset >= 6.0,
            "the first handle's ring is clipped: {widths:?}"
        );
        fixture.console.assert_clean("a row at 320px").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A second drag still shows its preview: the neighbour steps aside before the drop (todo 1438).
#[test]
fn a_second_drag_still_moves_the_neighbours_before_the_drop() {
    use e2e::archetypes::{Round, second_drag};
    use e2e::passes::pointer;
    block_on(async {
        let fixture = Fixture::open("/sortable", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let outcome = async {
            wait::for_visible(page, "#Alpha").await?;
            let (gamma, delta) = (
                pointer::centre_of(page, "#Gamma").await?,
                pointer::centre_of(page, "#Delta").await?,
            );
            let rounds = [
                Round {
                    handle: "#Alpha button",
                    passed: "#Beta",
                    order: "Beta Alpha Gamma Delta",
                },
                Round {
                    handle: "#Beta button",
                    passed: "#Alpha",
                    order: "Alpha Beta Gamma Delta",
                },
            ];
            // In front: behind, each move waits about a second for a frame.
            e2e::frames::in_front(
                page,
                second_drag(page, &rounds, "#Gamma", delta.y - gamma.y),
            )
            .await
        }
        .await;
        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

/// On a touch screen the handle's description names the move buttons, not Space.
#[test]
fn a_touch_screen_describes_the_handle_by_the_move_buttons() {
    const DESCRIPTION: &str = "document.getElementById(document.querySelector('#Alpha button')\
         .getAttribute('aria-describedby')).textContent";
    block_on(async {
        let fixture = Fixture::open("/sortable", Viewport::Desktop).await.unwrap();
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
                &format!("{DESCRIPTION} === 'Use the move buttons to reorder the item.'"),
                "the touch description",
            )
            .await
        }
        .await;
        fixture.close().await.unwrap();
        outcome.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("sortable", "/sortable")
        .focusable("#Alpha button")
        .run();
}

/// Lifted by the keyboard, and the other layouts (todo 1773).
#[test]
fn the_lifted_item_meets_the_baseline() {
    Suite::new("sortable-lifted", "/sortable")
        .state(
            "lifted",
            &[Step::TabTo("#Alpha button"), Step::Press(keyboard::SPACE)],
            "#Alpha[data-state~=dragging]",
        )
        .run();
}

#[test]
fn the_horizontal_list_meets_the_baseline() {
    Suite::new("sortable-horizontal", "/sortable/horizontal").run();
}

#[test]
fn the_rtl_list_meets_the_baseline() {
    Suite::new("sortable-rtl", "/sortable/rtl").run();
}
