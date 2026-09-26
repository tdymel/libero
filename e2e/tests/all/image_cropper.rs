//! `ImageCropper` (1218): the box and its corners move and resize by key and by
//! drag, and an `aspect` holds through both. Starts at `25,25,50,50` percent.
//! Then `FileField { crop }`: a dropped image is cropped in a dialog.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const BOX: &str = "[data-slot=box]";
const SOUTH_EAST: &str = "[aria-label='Bottom right corner']";
const NORTH_WEST: &str = "[aria-label='Top left corner']";

/// 40 x 20 px, red on the left half, blue on the right.
const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 40, 0, 0, 0, 20, 8, 2,
    0, 0, 0, 112, 36, 232, 236, 0, 0, 0, 39, 73, 68, 65, 84, 120, 218, 99, 248, 207, 192, 64, 54,
    162, 64, 235, 127, 134, 81, 139, 71, 45, 30, 181, 120, 212, 226, 81, 139, 71, 45, 30, 181, 120,
    212, 226, 145, 99, 49, 0, 96, 237, 29, 14, 213, 214, 78, 215, 0, 0, 0, 0, 73, 69, 78, 68, 174,
    66, 96, 130,
];

#[test]
fn it_meets_the_baseline() {
    Suite::new("image_cropper", "/image-cropper")
        .focusable(BOX)
        .focusable(SOUTH_EAST)
        .targets(SOUTH_EAST)
        .run();
}

/// A dropped PNG opens the dialog at the largest centred square; Apply hands
/// `onchange` that 20 px square, cut by the canvas and capped by `max_size` at
/// 16 px, still a PNG.
#[test]
fn a_dropped_image_is_cropped_before_the_field_takes_it() {
    let path = std::env::temp_dir().join(format!("e2e-crop-{}.png", std::process::id()));
    std::fs::write(&path, PNG).unwrap();
    block_on(async {
        let fixture = Fixture::open("/file-field/crop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        crate::file_field::drop_files(
            page,
            "[data-fixture-ready] [role=group]",
            vec![path.to_string_lossy().into_owned()],
        )
        .await;
        wait::for_js_true(
            page,
            "!!document.querySelector('[role=dialog] [data-slot=box]')",
            "the crop dialog to open",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "(() => { const apply = [...document.querySelectorAll('[role=dialog] button')]
                .find((button) => button.textContent.trim() === 'Apply');
              if (!apply || apply.disabled) return false;
              apply.click();
              return true; })()",
            "Apply to be pressable",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('rect').textContent === '25,0,50,100'
              && document.getElementById('out').textContent.startsWith('e2e-crop-')
              && document.getElementById('out').textContent.endsWith('.png image/png 16x16')",
            "the field to take the 16 x 16 crop",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "the dialog to close",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the crop").unwrap();
        fixture.close().await.unwrap();
    });
    let _ = std::fs::remove_file(path);
}

/// The same crop through Browse, the PNG handed to the page's canvas and back
/// as base64 in a WebView (1230).
async fn a_picked_image_is_cropped<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#stub-picker").await?;
    eventually_text(d, "#stubbed", "true", "stubbing the picker").await?;
    d.click("#avatar").await?;
    eventually(d, "the crop dialog to open", async |d| {
        d.exists("[role=dialog] [data-slot=box]").await
    })
    .await?;
    // Cancel, then Apply, the footer's last button; enabled once the box settles.
    let apply = "[role=dialog] button:last-of-type:not(:first-of-type)";
    eventually(d, "Apply to be enabled", async |d| {
        Ok(d.attr(apply, "disabled").await?.is_none())
    })
    .await?;
    d.click(apply).await?;
    eventually_text(d, "#rect", "25,0,50,100", "Apply").await?;
    eventually_text(d, "#out", "e2e-crop-picked.png image/png 16x16", "the crop").await?;
    eventually(d, "the dialog to close", async |d| {
        Ok(!d.exists("[role=dialog]").await?)
    })
    .await
}

e2e::scenario!(
    a_picked_image_is_cropped_through_the_page,
    "/file-field/crop",
    a_picked_image_is_cropped,
    native: skip("Blitz has no canvas: the field takes the image uncropped")
);

/// A touch on the image outside the box scrolls the page; only the box takes drags.
async fn only_the_box_claims_touches<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let root = d.style(":has(> [data-slot=image])", "touch-action").await?;
    anyhow::ensure!(root == "auto", "the cropper's touch-action is {root}");
    let frame = d.style("[data-slot=frame]", "touch-action").await?;
    anyhow::ensure!(frame == "none", "the box's touch-action is {frame}");
    Ok(())
}

/// A disabled cropper shows no move or resize cursor over the box and its handles.
async fn a_disabled_cropper_shows_no_grab_cursor<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for part in ["[data-slot=frame]", SOUTH_EAST, "[data-grip=e]"] {
        let cursor = d.style(part, "cursor").await?;
        anyhow::ensure!(cursor == "not-allowed", "{part} shows the {cursor} cursor");
    }
    Ok(())
}

e2e::scenario!(
    only_the_box_claims_touches,
    "/image-cropper",
    only_the_box_claims_touches
);
e2e::scenario!(
    a_disabled_cropper_shows_no_grab_cursor,
    "/image-cropper/disabled",
    a_disabled_cropper_shows_no_grab_cursor
);

async fn the_arrows_move_the_box<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(BOX).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(d, "#crop", "26,25,50,50", "ArrowRight").await?;
    d.press_shift(keyboard::ARROW_DOWN).await?;
    eventually_text(d, "#crop", "26,35,50,50", "Shift+ArrowDown").await?;
    eventually(d, "the box's value text to follow", async |d| {
        Ok(d.attr(BOX, "aria-valuetext").await?.as_deref() == Some("50% by 50%, at 26%, 35%"))
    })
    .await
}

async fn the_arrows_resize_from_a_corner<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SOUTH_EAST).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(
        d,
        "#crop",
        "25,25,51,50",
        "ArrowRight on the bottom right corner",
    )
    .await?;
    d.focus(NORTH_WEST).await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, "#crop", "25,24,51,51", "ArrowUp on the top left corner").await
}

/// A drag by a tenth of the picture, which a phone draws narrower than its 400 px.
async fn tenth_of_the_picture<D: Driver>(d: &mut D) -> Result<(f64, f64)> {
    let picture = d.rect("[data-slot=image]").await?;
    Ok((picture.width / 10.0, picture.height / 10.0))
}

/// A tenth of the picture each way is 10%; the corner takes the focus.
async fn a_drag_resizes_from_a_corner<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#before").await?;
    let (dx, dy) = tenth_of_the_picture(d).await?;
    d.drag(SOUTH_EAST, dx, dy).await?;
    eventually_text(
        d,
        "#crop",
        "25,25,60,60",
        "a drag on the bottom right corner",
    )
    .await?;
    eventually_focused(d, SOUTH_EAST, "the drag").await
}

async fn a_drag_moves_the_box<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (dx, dy) = tenth_of_the_picture(d).await?;
    d.drag("[data-slot=frame]", -dx, -dy).await?;
    eventually_text(d, "#crop", "15,15,50,50", "a drag on the box").await?;
    eventually_focused(d, BOX, "the drag").await
}

/// A square on a 2:1 picture: 200 px each way, and still square once a key
/// shrinks it.
async fn the_aspect_holds<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#pixels", "200x200", "the starting square").await?;
    d.focus(SOUTH_EAST).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_text(d, "#pixels", "196x196", "ArrowLeft on a square crop").await
}

e2e::scenario!(
    the_arrows_move_the_box,
    "/image-cropper",
    the_arrows_move_the_box
);
e2e::scenario!(
    the_arrows_resize_from_a_corner,
    "/image-cropper",
    the_arrows_resize_from_a_corner
);
e2e::scenario!(
    a_drag_on_a_corner_resizes_and_focuses_it,
    "/image-cropper",
    a_drag_resizes_from_a_corner
);
e2e::scenario!(
    a_drag_on_the_box_moves_it,
    "/image-cropper",
    a_drag_moves_the_box
);
e2e::scenario!(
    an_aspect_holds_through_a_key,
    "/image-cropper/square",
    the_aspect_holds
);
