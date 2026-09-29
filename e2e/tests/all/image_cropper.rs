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

/// Forced colours drop a box-shadow: the dim mask opts out, so the crop still
/// stands out from the rest of the image (1278).
#[test]
fn the_dim_mask_stays_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/image-cropper", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        let shadow: String = page
            .evaluate("getComputedStyle(document.querySelector('[data-slot=mask] > *')).boxShadow")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            shadow.contains("9999px"),
            "the mask lost its shadow: {shadow}"
        );
        fixture.close().await.unwrap();
    });
}

/// A dropped PNG opens the dialog at 80% of the largest centred square; Apply
/// hands `onchange` that 16 px square, cut by the canvas and held by `max_size`
/// at 16 px, still a PNG.
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
            "document.getElementById('rect').textContent === '30,10,40,80'
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

/// A dropped file that is no image: the dialog says so and Apply stays off (1265).
#[test]
fn a_broken_image_in_the_crop_dialog_says_so() {
    let path = std::env::temp_dir().join(format!("e2e-broken-{}.png", std::process::id()));
    std::fs::write(&path, b"not an image").unwrap();
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
            "(() => { const alert = document.querySelector('[role=dialog] [role=alert]');
              const apply = [...document.querySelectorAll('[role=dialog] button')]
                .find((button) => button.textContent.trim() === 'Apply');
              return !!alert && alert.textContent.includes('could not be loaded')
                && !!apply && apply.disabled
                && !document.querySelector('[role=dialog] [data-slot=box]'); })()",
            "the dialog to report the broken image",
        )
        .await
        .unwrap();
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
    eventually_text(d, "#rect", "30,10,40,80", "Apply").await?;
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
    // An id, not `:has(...)`, which Blitz cannot parse.
    let root = d.style("#cropper", "touch-action").await?;
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

/// An uncontrolled box refits to a new aspect or picture, a moved one too (1263).
async fn the_box_refits_a_new_picture<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#crop", "30,10,40,80", "the starting square").await?;
    d.focus(BOX).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(d, "#crop", "31,10,40,80", "ArrowRight").await?;
    d.click("#widen").await?;
    eventually_text(d, "#crop", "10,10,80,80", "a 2:1 aspect").await?;
    d.click("#widen").await?;
    eventually_text(d, "#crop", "30,10,40,80", "back to square").await?;
    d.click("#swap").await?;
    eventually_text(d, "#crop", "10,30,80,40", "a tall picture").await
}

/// At `min_size` the handles' hit areas meet; a drag on the box's centre still moves it (1264).
async fn a_tiny_box_still_moves<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (dx, dy) = tenth_of_the_picture(d).await?;
    d.drag("[data-slot=frame]", -dx, -dy).await?;
    eventually_text(d, "#crop", "30,30,5,5", "a drag on the tiny box").await?;
    eventually_focused(d, BOX, "the drag").await
}

/// A finger drags the box without scrolling the page; two pinch it around its
/// centre (1368).
#[test]
fn a_touch_drags_the_box_and_two_pinch_it() {
    use e2e::passes::pointer::{self, Point};
    block_on(async {
        let fixture = Fixture::open("/image-cropper", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let crop = "document.getElementById('crop').textContent";
        wait::for_js_true(page, &format!("{crop} === '25,25,50,50'"), "the box")
            .await
            .unwrap();
        let (left, top, width, height, scroll): (f64, f64, f64, f64, f64) = page
            .evaluate(
                "(() => { const r = document.querySelector('[data-slot=image]').getBoundingClientRect();
                  return [r.left, r.top, r.width, r.height, scrollY]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let centre = Point {
            x: left + width / 2.0,
            y: top + height / 2.0,
        };
        let to = Point {
            x: centre.x - width / 10.0,
            y: centre.y - height / 10.0,
        };
        pointer::touch_drag(page, centre, to, 8).await.unwrap();
        wait::for_js_true(page, &format!("{crop} === '15,15,50,50'"), "a touch drag")
            .await
            .unwrap();
        let after: f64 = page
            .evaluate("scrollY")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(after, scroll, "the touch drag scrolled the page");

        // Spread three times as far: the box grows to the whole image and no further.
        let middle = Point {
            x: left + width * 0.4,
            y: top + height * 0.4,
        };
        pointer::pinch(page, middle, width * 0.2, width * 0.6, 8)
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{crop} === '0,0,100,100'"), "a pinch out")
            .await
            .unwrap();
        // Back to a fifth of the spread: about a fifth of the size, centred.
        let centre_box = |text: String| -> Vec<f64> {
            text.split(',').map(|part| part.parse().unwrap()).collect()
        };
        pointer::pinch(page, centre, width * 0.5, width * 0.1, 8)
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{crop} !== '0,0,100,100'"), "a pinch in")
            .await
            .unwrap();
        let shrunk: String = page.evaluate(crop).await.unwrap().into_value().unwrap();
        let [x, y, w, h] = centre_box(shrunk.clone())[..] else {
            panic!("{shrunk}");
        };
        assert!((w - 20.0).abs() <= 2.0 && (h - w).abs() <= 1.0, "{shrunk}");
        assert!(
            (x + w / 2.0 - 50.0).abs() <= 1.0 && (y + h / 2.0 - 50.0).abs() <= 1.0,
            "{shrunk}"
        );
        fixture.console.assert_clean("the touches").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A `src` that fails calls `onerror` once, warns, and draws no box over the alt text (1265).
#[test]
fn a_broken_src_reports_and_draws_no_box() {
    block_on(async {
        let fixture = Fixture::open("/image-cropper/broken", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "document.getElementById('error').textContent === '1'
              && !document.querySelector('[data-slot=box], [data-slot=frame]')",
            "onerror, and no box",
        )
        .await
        .unwrap();
        let said = fixture.console.drain();
        assert!(
            said.iter()
                .any(|message| message.contains("`src` failed to load")),
            "no warning: {said:?}"
        );
        fixture.close().await.unwrap();
    });
}

e2e::scenario!(
    the_box_refits_a_new_picture,
    "/image-cropper/refit",
    the_box_refits_a_new_picture
);
e2e::scenario!(
    a_tiny_box_still_moves,
    "/image-cropper/tiny",
    a_tiny_box_still_moves
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

/// A square on a 2:1 picture starts at 160 px each way, and stays square once
/// a key shrinks it.
async fn the_aspect_holds<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#pixels", "160x160", "the starting square").await?;
    d.focus(SOUTH_EAST).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_text(d, "#pixels", "156x156", "ArrowLeft on a square crop").await
}

/// An unset free box starts short of the edges, so a handle drag shrinks it and
/// a box drag moves it straight away; the whole image could not move (1343).
async fn the_starting_box_resizes_and_moves<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#crop", "10,10,80,80", "the starting box").await?;
    let (dx, dy) = tenth_of_the_picture(d).await?;
    d.drag(SOUTH_EAST, -dx, -dy).await?;
    eventually_text(
        d,
        "#crop",
        "10,10,70,70",
        "a drag on the bottom right corner",
    )
    .await?;
    d.drag("[data-slot=frame]", dx, dy).await?;
    eventually_text(d, "#crop", "20,20,70,70", "a drag on the box").await
}

e2e::scenario!(
    the_starting_box_takes_a_handle_drag_and_a_move,
    "/image-cropper/start",
    the_starting_box_resizes_and_moves
);

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

const PLUS: keyboard::Key = keyboard::Key {
    key: "+",
    code: "Equal",
    vk: 187,
    text: Some("+"),
};
const MINUS: keyboard::Key = keyboard::Key {
    key: "-",
    code: "Minus",
    vk: 189,
    text: Some("-"),
};

/// Pan mode: the arrows move the crop, + and - zoom around the still box (1368).
async fn the_keys_pan_and_zoom<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#crop", "30,10,40,80", "the starting square").await?;
    anyhow::ensure!(
        !d.exists(SOUTH_EAST).await?,
        "pan mode drew a resize handle"
    );
    d.focus(BOX).await?;
    d.press(PLUS).await?;
    eventually_text(d, "#crop", "32,14,36,73", "+").await?;
    eventually(d, "the box to speak the zoom", async |d| {
        Ok(d.attr(BOX, "aria-valuetext").await?.as_deref()
            == Some("36% by 73%, at 32%, 14%, zoom 110%"))
    })
    .await?;
    let (root, image) = (
        d.rect("#cropper").await?,
        d.rect("[data-slot=image]").await?,
    );
    anyhow::ensure!(
        (image.width / root.width - 1.1).abs() < 0.02,
        "the image did not zoom: {} of {}",
        image.width,
        root.width
    );
    d.press(MINUS).await?;
    eventually_text(d, "#crop", "30,10,40,80", "-").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(d, "#crop", "31,10,40,80", "ArrowRight").await?;
    // The image dragged a tenth of the cropper right: the crop a tenth left.
    d.drag("[data-slot=frame]", root.width / 10.0, 0.0).await?;
    eventually_text(d, "#crop", "21,10,40,80", "a drag").await
}

e2e::scenario!(
    pan_mode_keys_move_and_zoom,
    "/image-cropper/pan",
    the_keys_pan_and_zoom
);

/// Pan mode: the box holds still while a finger pans the image under it and
/// two zoom it around their midpoint; the whole cropper takes touches (1368).
#[test]
fn a_touch_pans_and_pinch_zooms_the_image_under_the_box() {
    use e2e::passes::pointer::{self, Point};
    block_on(async {
        let fixture = Fixture::open("/image-cropper/pan", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let crop = "document.getElementById('crop').textContent";
        wait::for_js_true(page, &format!("{crop} === '30,10,40,80'"), "the square")
            .await
            .unwrap();
        // Root left, top, width, height; frame width, height; image width; box left, width.
        let rects = "(() => { const r = (s) => document.querySelector(s).getBoundingClientRect();
            const [root, frame, image, box] = ['#cropper', '[data-slot=frame]', '[data-slot=image]', '[data-slot=box]'].map(r);
            return [root.left, root.top, root.width, root.height, frame.width, frame.height,
                image.width, box.left - root.left, box.width]; })()";
        let before: Vec<f64> = page.evaluate(rects).await.unwrap().into_value().unwrap();
        let (left, top, width, height) = (before[0], before[1], before[2], before[3]);
        assert!(
            (before[4] - width).abs() < 1.0 && (before[5] - height).abs() < 1.0,
            "the frame does not cover the cropper: {before:?}"
        );
        let overflow: String = page
            .evaluate("getComputedStyle(document.getElementById('cropper')).overflow")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(overflow, "hidden");

        // A tenth of the cropper right and up: the crop moves a tenth left and down.
        let centre = Point {
            x: left + width / 2.0,
            y: top + height / 2.0,
        };
        let to = Point {
            x: centre.x + width / 10.0,
            y: centre.y - height / 10.0,
        };
        pointer::touch_drag(page, centre, to, 8).await.unwrap();
        wait::for_js_true(page, &format!("{crop} === '20,20,40,80'"), "a touch pan")
            .await
            .unwrap();

        // Twice the spread around the centre: half the crop around the point there.
        pointer::pinch(page, centre, width * 0.2, width * 0.4, 8)
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{crop} === '30,40,20,40'"), "a pinch out")
            .await
            .unwrap();
        let after: Vec<f64> = page.evaluate(rects).await.unwrap().into_value().unwrap();
        assert!(
            (after[6] / width - 2.0).abs() < 0.02,
            "the image did not double: {after:?}"
        );
        assert!(
            (after[7] - before[7]).abs() < 1.0 && (after[8] - before[8]).abs() < 1.0,
            "the box moved: {before:?} -> {after:?}"
        );
        fixture.console.assert_clean("the touches").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Pan mode: a wheel notch zooms by the + key's step around the pointer, a trackpad
/// pinch (ctrl+wheel) by its spread (1456).
#[test]
fn the_wheel_and_a_trackpad_pinch_zoom_the_image() {
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchMouseEventParams, DispatchMouseEventType,
    };
    block_on(async {
        let fixture = Fixture::open("/image-cropper/pan", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let crop = "document.getElementById('crop').textContent";
        wait::for_js_true(page, &format!("{crop} === '30,10,40,80'"), "the square")
            .await
            .unwrap();
        let centre = e2e::passes::pointer::centre_of(page, "#cropper")
            .await
            .unwrap();
        let wheel = async |delta_y: f64, ctrl: bool| {
            let event = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseWheel)
                .x(centre.x)
                .y(centre.y)
                .delta_x(0.0)
                .delta_y(delta_y)
                .modifiers(if ctrl { 2 } else { 0 })
                .build()
                .unwrap();
            page.execute(event).await.unwrap();
        };

        wheel(-100.0, false).await;
        wait::for_js_true(page, &format!("{crop} === '32,14,36,73'"), "a notch in")
            .await
            .unwrap();
        wheel(100.0, false).await;
        wait::for_js_true(page, &format!("{crop} === '30,10,40,80'"), "a notch out")
            .await
            .unwrap();
        // A pinch to twice the spread: half the crop around the centre.
        wheel(-100.0 * 2f64.ln(), true).await;
        wait::for_js_true(page, &format!("{crop} === '40,30,20,40'"), "a pinch")
            .await
            .unwrap();
        fixture.console.assert_clean("the wheel").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Pan mode: a zoom slider is the single-pointer and keyboard zoom (WCAG 2.5.1, 1458).
#[test]
fn the_zoom_slider_zooms_with_one_pointer() {
    block_on(async {
        let fixture = Fixture::open("/image-cropper/pan", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let crop = "document.getElementById('crop').textContent";
        let thumb = "#cropper [data-slot=zoom] [role=slider]";
        let spoken = format!("document.querySelector('{thumb}').getAttribute('aria-valuetext')");
        wait::for_js_true(
            page,
            &format!(
                "{crop} === '30,10,40,80' && {spoken} === '100%' \
                 && document.querySelector('{thumb}').getAttribute('aria-label') === 'Zoom'"
            ),
            "the slider at 100%",
        )
        .await
        .unwrap();
        page.evaluate(format!("document.querySelector('{thumb}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::HOME).await.unwrap();
        // The largest crop of the box's shape.
        wait::for_js_true(
            page,
            &format!("{crop} === '25,0,50,100' && {spoken} === '80%'"),
            "Home to the widest zoom",
        )
        .await
        .unwrap();
        // One click on the track's middle zooms in, around the box's centre.
        let track = e2e::passes::pointer::centre_of(page, "#cropper [data-slot=zoom]")
            .await
            .unwrap();
        e2e::passes::pointer::click_at(page, track).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const [x, y, w, h] = {crop}.split(',').map(Number); \
                 return w < 40 && Math.abs(x + w / 2 - 50) <= 1 && Math.abs(y + h / 2 - 50) <= 1; }})()"
            ),
            "a click to zoom in",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the slider").unwrap();
        fixture.close().await.unwrap();
    });
}
