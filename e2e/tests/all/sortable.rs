//! `Sortable` and `use_sortable`: a handle drag reorders, a handle click stays a click (1095).

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually_focused, eventually_text, linger};

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

e2e::scenario!(
    a_handle_drag_down_moves_the_item_and_keeps_its_handle_focused,
    "/sortable",
    a_drag_down_moves_it,
    android: skip("1095 slice 3: Android")
);
e2e::scenario!(
    a_handle_drag_up_moves_the_item_to_the_top,
    "/sortable",
    a_drag_up_moves_it,
    android: skip("1095 slice 3: Android")
);
e2e::scenario!(
    a_drag_short_of_the_next_middle_moves_nothing,
    "/sortable",
    a_short_drag_moves_nothing,
    android: skip("1095 slice 3: Android")
);
e2e::scenario!(
    a_horizontal_list_reorders_by_a_sideways_drag,
    "/sortable/horizontal",
    a_row_reorders_sideways,
    android: skip("1095 slice 3: Android")
);
e2e::scenario!(
    a_click_on_a_hook_handle_clicks_and_a_drag_does_not,
    "/sortable/hook",
    a_click_stays_a_click,
    android: skip("1095 slice 3: Android")
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("sortable", "/sortable")
        .focusable("#Alpha button")
        .run();
}
