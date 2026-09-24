//! `Rating`: click, drag, keys, hover, clear and read-only, on every platform.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, Platform, eventually, linger};
use e2e::passes::keyboard;

const STARS: &str = "#stars";

async fn value_of<D: Driver>(d: &mut D, rating: &str) -> Result<f64> {
    Ok(d.attr(rating, "aria-valuenow")
        .await?
        .and_then(|v| v.parse().ok())
        .unwrap_or(-1.0))
}

async fn becomes<D: Driver>(d: &mut D, rating: &str, want: f64, what: &str) -> Result<()> {
    let label = format!("{what}: {rating} to read {want}");
    eventually(d, &label, async |d| Ok(value_of(d, rating).await? == want)).await
}

/// Clicks `rating`'s symbol `nth` (from 1) at `along` its width, from the left.
async fn click_symbol<D: Driver>(d: &mut D, rating: &str, nth: u8, along: f64) -> Result<()> {
    let at = d
        .rect(&format!("{rating} > [data-slot=symbol]:nth-child({nth})"))
        .await?;
    d.click_at(at.x + at.width * along, at.y + at.height / 2.0)
        .await
}

/// The fourth symbol's right half is 4, the second's left half 1.5.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    click_symbol(d, STARS, 4, 0.75).await?;
    becomes(d, STARS, 4.0, "a click on the fourth's right half").await?;
    click_symbol(d, STARS, 2, 0.25).await?;
    becomes(d, STARS, 1.5, "a click on the second's left half").await?;
    let text = d.attr(STARS, "aria-valuetext").await?;
    ensure!(text.as_deref() == Some("1.5 of 5"), "valuetext {text:?}");
    Ok(())
}

e2e::scenario!(a_click_picks_the_half_it_lands_on, "/rating", clicks);

/// Arrows step a half, Shift a whole symbol, End and Home go to the ends.
/// Without `clearable` the floor is the first step.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(STARS).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    linger(d, 4).await;
    ensure!(
        value_of(d, STARS).await? == 0.0,
        "ArrowLeft raised an unrated value"
    );
    d.press(keyboard::ARROW_RIGHT).await?;
    becomes(d, STARS, 0.5, "ArrowRight").await?;
    d.press_shift(keyboard::ARROW_RIGHT).await?;
    becomes(d, STARS, 1.5, "Shift+ArrowRight").await?;
    d.press(keyboard::END).await?;
    becomes(d, STARS, 5.0, "End").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    becomes(d, STARS, 4.5, "ArrowDown").await?;
    d.press(keyboard::HOME).await?;
    becomes(d, STARS, 0.5, "Home").await
}

e2e::scenario!(the_keys_step_by_the_fraction, "/rating", keys);

/// A press on the second symbol's centre (42px in) slid 56px right ends 98px
/// along the 140px row: 3.5.
async fn scrub<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.drag(
        &format!("{STARS} > [data-slot=symbol]:nth-child(2)"),
        56.0,
        0.0,
    )
    .await?;
    let scrubbed = eventually(d, "a sideways drag to scrub", async |d| {
        Ok((3.0..=4.0).contains(&value_of(d, STARS).await?))
    })
    .await;
    ensure!(
        scrubbed.is_ok(),
        "the drag left {}",
        value_of(d, STARS).await?
    );
    Ok(())
}

e2e::scenario!(a_sideways_drag_scrubs_across_the_symbols, "/rating", scrub);

/// Pressing the current value of a clearable rating resets it to 0; a vertical
/// swipe on a phone scrolls and leaves it.
async fn clears<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Android {
        d.drag("#clear", 0.0, -120.0).await?;
        linger(d, 8).await;
        ensure!(value_of(d, "#clear").await? == 3.0, "a swipe changed it");
    }
    click_symbol(d, "#clear", 3, 0.5).await?;
    becomes(d, "#clear", 0.0, "a click on the current value").await?;
    click_symbol(d, "#clear", 2, 0.5).await?;
    becomes(d, "#clear", 2.0, "a click on the second").await
}

e2e::scenario!(
    a_clearable_rating_clears_on_its_own_value,
    "/rating",
    clears
);

/// Hover draws the value under the mouse and reports it; `aria-valuenow` keeps
/// the picked one.
async fn hovers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover(&format!("{STARS} > [data-slot=symbol]:nth-child(3)"))
        .await?;
    eventually(d, "the hover read-out", async |d| {
        Ok(!d.text("#stars-hover").await?.is_empty())
    })
    .await?;
    let hovered: f64 = d.text("#stars-hover").await?.parse()?;
    ensure!((2.5..=3.0).contains(&hovered), "hovered {hovered}");
    ensure!(
        d.exists(&format!("{STARS} [data-slot=fill]")).await?,
        "nothing drawn filled"
    );
    ensure!(value_of(d, STARS).await? == 0.0, "hover changed the value");
    Ok(())
}

e2e::scenario!(
    hovering_previews_without_changing_the_value,
    "/rating",
    hovers,
    android: skip("a touch screen has no hover")
);

/// Read-only takes focus but neither clicks nor keys; display-only takes neither.
async fn fixed<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    click_symbol(d, "#fixed", 5, 0.5).await?;
    d.focus("#fixed").await?;
    ensure!(d.is_focused("#fixed").await?, "read-only lost its tab stop");
    d.press(keyboard::END).await?;
    linger(d, 4).await;
    ensure!(value_of(d, "#fixed").await? == 2.0, "read-only changed");
    ensure!(
        d.attr("#average", "role").await?.as_deref() == Some("img"),
        "display-only is not an image"
    );
    ensure!(
        d.attr("#average", "tabindex").await?.is_none(),
        "display-only is a tab stop"
    );
    Ok(())
}

e2e::scenario!(read_only_ignores_presses_and_keys, "/rating", fixed);

/// Todo 1169: a bound rating posts its picked value through the hidden input.
async fn posts<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    click_symbol(d, "#review", 4, 0.5).await?;
    becomes(d, "#review", 4.0, "a click on the fourth").await?;
    d.click("#send").await?;
    eventually(d, "the submit's read-out", async |d| {
        Ok(d.text("#posted").await? == r#"stars=Text("4")"#)
    })
    .await
}

e2e::scenario!(a_bound_rating_posts_the_picked_value, "/rating/form", posts);

/// Right to left, the rightmost symbol is the first, and ArrowLeft raises.
async fn rtl<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let first = d.rect("#rtl > [data-slot=symbol]:nth-child(1)").await?;
    let second = d.rect("#rtl > [data-slot=symbol]:nth-child(2)").await?;
    ensure!(first.x > second.x, "the first symbol is not rightmost");
    click_symbol(d, "#rtl", 2, 0.5).await?;
    becomes(d, "#rtl", 2.0, "a click on the second").await?;
    d.focus("#rtl").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    becomes(d, "#rtl", 3.0, "ArrowLeft under RTL").await
}

e2e::scenario!(
    right_to_left_mirrors_the_row_and_the_arrows,
    "/rating/rtl",
    rtl
);
