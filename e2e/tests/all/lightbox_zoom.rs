//! `Lightbox` zoom: the `+`/`-` keys, the zoom buttons, clicks and double-clicks about a
//! spot, pans clamped to the picture's edge, the resize clamp, the tap and the pinch.

use anyhow::{Result, bail, ensure};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::{
    Fixture, Viewport, passes::focus, passes::keyboard, passes::motion, passes::pointer, wait,
};

use super::lightbox::{DIALOG, PICTURE, TRIGGER, wait_showing};

/// `+` and `-` step like the wheel, so the keyboard reaches `max_zoom` (the
/// theme's 8x) and back in fine steps (WCAG 2.1.1).
#[test]
fn plus_and_minus_zoom_to_max_zoom_and_back() {
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
    const STYLE: &str =
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText";
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        wait_showing(page, 0).await.unwrap();
        keyboard::tab_to(page, PICTURE, 12).await.unwrap();
        // 1.25^10 passes 8, so the tenth press clamps.
        for _ in 0..10 {
            keyboard::press(page, PLUS).await.unwrap();
        }
        if wait::for_js_true(
            page,
            &format!("{STYLE}.includes('scale(8)')"),
            "the picture at 8x",
        )
        .await
        .is_err()
        {
            let style: String = page.evaluate(STYLE).await.unwrap().into_value().unwrap();
            panic!("ten presses of + left the picture at {style:?}, not the theme's max_zoom 8x");
        }
        for _ in 0..10 {
            keyboard::press(page, MINUS).await.unwrap();
        }
        wait::for_js_true(
            page,
            &format!("!{STYLE}.includes('scale(')"),
            "the picture fitted again",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("the lightbox keyboard zoom")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 864: the zoom buttons step like `+`/`-` and turn `aria-disabled` at a limit, keeping
/// focus; a double-click steps 2x, 4x, 8x, then back to fit.
#[test]
fn the_zoom_buttons_and_double_click_step_the_zoom() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        zoom_buttons(&fixture.page).await.unwrap();
        fixture
            .console
            .assert_clean("the lightbox zoom buttons")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn zoom_buttons(page: &Page) -> Result<()> {
    const ZOOM_IN: &str = "[role=dialog] button[aria-label='Zoom in']";
    const ZOOM_OUT: &str = "[role=dialog] button[aria-label='Zoom out']";
    const STYLE: &str =
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText";
    // Enabled drops the attribute rather than writing `false`.
    let disabled = |selector: &str, value: bool| {
        let expected = if value { "'true'" } else { "null" };
        format!(
            "document.querySelector({selector:?})?.getAttribute('aria-disabled') === {expected}"
        )
    };
    let wait_style = async |check: String, what: &str| -> Result<()> {
        if wait::for_js_true(page, &check, what).await.is_err() {
            let style: String = page.evaluate(STYLE).await?.into_value()?;
            bail!("waiting for {what}, the picture's style is {style:?}");
        }
        Ok(())
    };
    motion::set_reduced_motion(page, true).await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_OUT, true),
        "zoom out disabled when fitted",
    )
    .await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_IN, false),
        "zoom in enabled when fitted",
    )
    .await?;
    // Disabled looks it, and a hover does not light it up.
    let look = format!(
        "(() => {{ const s = getComputedStyle(document.querySelector({ZOOM_OUT:?})); return [s.opacity, s.backgroundColor, s.color].join(' '); }})()"
    );
    let rest: String = page.evaluate(look.as_str()).await?.into_value()?;
    pointer::hover(page, ZOOM_OUT).await?;
    let hovered: String = page.evaluate(look.as_str()).await?.into_value()?;
    if !rest.starts_with("0.5 ") || hovered != rest {
        bail!("disabled zoom out: {rest:?} at rest, {hovered:?} hovered");
    }

    pointer::click(page, ZOOM_IN).await?;
    wait_style(
        format!("{STYLE}.includes('scale(1.25)')"),
        "one wheel step in",
    )
    .await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_OUT, false),
        "zoom out enabled once zoomed",
    )
    .await?;

    // 1.25^10 passes 8: the button clamps, disables, and keeps the focus.
    keyboard::tab_to(page, ZOOM_IN, 12).await?;
    for _ in 0..10 {
        keyboard::press(page, keyboard::ENTER).await?;
    }
    wait_style(format!("{STYLE}.includes('scale(8)')"), "the picture at 8x").await?;
    wait::for_js_true(page, &disabled(ZOOM_IN, true), "zoom in disabled at 8x").await?;
    focus::assert_focused(page, ZOOM_IN, "zoom in at its limit").await?;

    keyboard::tab_to(page, ZOOM_OUT, 12).await?;
    for _ in 0..12 {
        keyboard::press(page, keyboard::ENTER).await?;
    }
    wait_style(format!("!{STYLE}.includes('scale(')"), "the picture fitted").await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_OUT, true),
        "zoom out disabled when fitted again",
    )
    .await?;
    focus::assert_focused(page, ZOOM_OUT, "zoom out at its limit").await?;

    let picture = "[role=dialog] [data-lightbox-frame=\"0\"] img";
    for scale in ["scale(2)", "scale(4)", "scale(8)"] {
        pointer::double_click(page, picture).await?;
        wait_style(
            format!("{STYLE}.includes('{scale}')"),
            &format!("a double-click to {scale}"),
        )
        .await?;
    }
    pointer::double_click(page, picture).await?;
    wait_style(
        format!("!{STYLE}.includes('scale(')"),
        "a double-click back to fit",
    )
    .await
}

/// Todo 421: zoom, pan to the edge, then shrink to a phone: the pan clamps to the new bounds.
#[test]
fn a_resize_keeps_a_zoomed_picture_in_its_frame() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        zoomed_resize(&fixture.page).await.unwrap();
        fixture.console.assert_clean("the lightbox resize").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The shown picture's centre offset from its frame's, and its bound, per axis:
/// `[dx, dy, x_bound, y_bound]`.
const PAN_JS: &str = r#"(() => {
    const frame = document.querySelector('[role=dialog] [data-lightbox-frame="0"]');
    const img = frame.querySelector('img');
    const f = frame.getBoundingClientRect(), i = img.getBoundingClientRect();
    const s = i.width / f.width;
    const r = Math.min(f.width / img.naturalWidth, f.height / img.naturalHeight, 1);
    const bound = (natural, side) => Math.max(0, (natural * r * s - side) / 2);
    return [
        i.left + i.width / 2 - (f.left + f.width / 2),
        i.top + i.height / 2 - (f.top + f.height / 2),
        bound(img.naturalWidth, f.width),
        bound(img.naturalHeight, f.height),
    ];
})()"#;

async fn zoomed_resize(page: &Page) -> Result<()> {
    const Z: keyboard::Key = keyboard::Key {
        key: "z",
        code: "KeyZ",
        vk: 90,
        text: Some("z"),
    };
    // No transform transition, so every rect read is the resting one.
    motion::set_reduced_motion(page, true).await?;
    // Short, so the 400x260 picture at 2x overhangs the 70vh stage (350px)
    // and has somewhere to pan; still over the phone layout's 30rem.
    page.execute(SetDeviceMetricsOverrideParams::new(900, 500, 1.0, false))
        .await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    // Decoded first: without a natural size the bounds are the whole frame.
    wait::for_js_true(
        page,
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img')?.naturalWidth > 0",
        "the first picture to decode",
    )
    .await?;
    keyboard::tab_to(page, PICTURE, 12).await?;
    keyboard::press(page, Z).await?;
    wait::for_js_true(
        page,
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText.includes('scale(2)')",
        "the picture to zoom",
    )
    .await?;
    // At picture 0 an arrow past the edge changes nothing.
    for _ in 0..40 {
        keyboard::press(page, keyboard::ARROW_LEFT).await?;
        keyboard::press(page, keyboard::ARROW_UP).await?;
    }
    // Resting on its edge on both axes, and moved on at least one.
    let at_edge = format!(
        "(() => {{ const [dx, dy, bx, by] = {PAN_JS}; return Math.abs(dx) >= bx - 1 && Math.abs(dy) >= by - 1 && (bx >= 1 || by >= 1); }})()"
    );
    if wait::for_js_true(page, &at_edge, "the pan to reach the picture's edge")
        .await
        .is_err()
    {
        let style: String = page
            .evaluate("document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText")
            .await?
            .into_value()?;
        let actual = focus::active_element(page).await?;
        bail!("the pan never reached the picture's edge: style {style:?}, focus {actual:?}");
    }
    let [dx, dy, ..]: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;

    let (width, height) = Viewport::Mobile.size();
    page.execute(SetDeviceMetricsOverrideParams::new(
        width, height, 1.0, true,
    ))
    .await?;
    let settled = format!(
        "(() => {{ const [dx, dy, bx, by] = {PAN_JS}; return Math.abs(dx) <= bx + 1 && Math.abs(dy) <= by + 1; }})()"
    );
    if wait::for_js_true(page, &settled, "the pan clamped to the resized frame")
        .await
        .is_err()
    {
        let seen: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
        bail!(
            "after the resize the picture sits at {seen:?} ([dx, dy, x_bound, y_bound]); it left its frame"
        );
    }
    // The clamp had to act: the old pan lies outside the new bounds.
    let [.., bx, by]: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
    if dx.abs() <= bx + 1.0 && dy.abs() <= by + 1.0 {
        bail!(
            "the old pan ({dx}, {dy}) still fits the new bounds ({bx}, {by}); the resize tested nothing"
        );
    }
    Ok(())
}

/// Todo 565: a click on a zoomed picture centres the spot clicked, while a
/// drag stays a pan; the zoom is announced and the keys are described.
#[test]
fn a_click_pans_a_zoomed_picture_and_the_zoom_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        click_pan(&fixture.page).await.unwrap();
        fixture
            .console
            .assert_clean("the lightbox click pan")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn click_pan(page: &Page) -> Result<()> {
    const Z: keyboard::Key = keyboard::Key {
        key: "z",
        code: "KeyZ",
        vk: 90,
        text: Some("z"),
    };
    const STATUS: &str = "[...document.querySelectorAll('[role=dialog] [role=status]')].map(s => s.textContent).join('|')";
    motion::set_reduced_motion(page, true).await?;
    // As in `zoomed_resize`: the picture at 2x overhangs the stage vertically.
    page.execute(SetDeviceMetricsOverrideParams::new(900, 500, 1.0, false))
        .await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    wait::for_js_true(
        page,
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img')?.naturalWidth > 0",
        "the first picture to decode",
    )
    .await?;
    keyboard::tab_to(page, PICTURE, 12).await?;
    let described: bool = page
        .evaluate(format!(
            "(() => {{ const ids = document.querySelector({PICTURE:?}).getAttribute('aria-describedby') || ''; \
             return ids.split(' ').some(id => document.getElementById(id)?.textContent.startsWith('Z, plus or minus to zoom')); }})()"
        ))
        .await?
        .into_value()?;
    if !described {
        bail!("the picture's aria-describedby does not reach the keys hint");
    }
    keyboard::press(page, Z).await?;
    wait::for_js_true(
        page,
        &format!("{STATUS}.includes('Zoomed to 200%')"),
        "the zoom to be announced",
    )
    .await?;

    let frame = pointer::centre_of(page, "[role=dialog] [data-lightbox-frame=\"0\"]").await?;
    let below = pointer::Point {
        x: frame.x,
        y: frame.y + 60.0,
    };
    pointer::drag(page, below, below, 1).await?;
    let dy_is = |target: f64| {
        format!("(() => {{ const [, dy] = {PAN_JS}; return Math.abs(dy - ({target})) <= 1; }})()")
    };
    if wait::for_js_true(page, &dy_is(-60.0), "a click to centre the spot clicked")
        .await
        .is_err()
    {
        let seen: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
        bail!("a click 60px below the centre left the picture at {seen:?}, not dy -60");
    }
    // A drag's release is no click: the pan stays where the drag put it.
    let up = pointer::Point {
        x: frame.x,
        y: frame.y + 30.0,
    };
    pointer::drag(page, frame, up, 5).await?;
    if wait::for_js_true(page, &dy_is(-30.0), "a drag to pan by its own distance")
        .await
        .is_err()
    {
        let seen: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
        bail!("a 30px drag down from dy -60 left the picture at {seen:?}, not dy -30");
    }

    // Todo 864: `z` steps on to 4x and 8x, then back to fit.
    for said in ["Zoomed to 400%", "Zoomed to 800%", "Zoom reset"] {
        keyboard::press(page, Z).await?;
        wait::for_js_true(
            page,
            &format!("{STATUS}.includes('{said}')"),
            &format!("z to announce {said:?}"),
        )
        .await?;
    }
    Ok(())
}

/// Todo 2283: a double-click on a zoomed picture off its centre zooms about the spot
/// clicked, its two clicks pan nothing. Todo 2284: a right press leaves no `dragging`.
#[test]
fn a_double_click_off_centre_keeps_the_spot_under_the_pointer() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        double_click_off_centre(&fixture.page).await.unwrap();
        fixture
            .console
            .assert_clean("the lightbox double-click off centre")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A press and release of `button` at `at`; `clicks` 2 on the second makes a `dblclick`.
async fn press_at(page: &Page, at: pointer::Point, button: MouseButton, clicks: i64) -> Result<()> {
    let held = if button == MouseButton::Right { 2 } else { 1 };
    for (kind, buttons) in [
        (DispatchMouseEventType::MousePressed, held),
        (DispatchMouseEventType::MouseReleased, 0),
    ] {
        page.execute(
            DispatchMouseEventParams::builder()
                .r#type(kind)
                .x(at.x)
                .y(at.y)
                .button(button.clone())
                .buttons(buttons)
                .click_count(clicks)
                .build()
                .map_err(anyhow::Error::msg)?,
        )
        .await?;
    }
    Ok(())
}

async fn double_click_off_centre(page: &Page) -> Result<()> {
    const IMG: &str = "[role=dialog] [data-lightbox-frame=\"0\"] img";
    motion::set_reduced_motion(page, true).await?;
    // As in `click_pan`: the picture at 2x overhangs the stage vertically only.
    page.execute(SetDeviceMetricsOverrideParams::new(900, 500, 1.0, false))
        .await?;
    pointer::click(page, TRIGGER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    wait::for_js_true(
        page,
        &format!("document.querySelector({IMG:?})?.naturalWidth > 0"),
        "the first picture to decode",
    )
    .await?;
    let frame = pointer::centre_of(page, "[role=dialog] [data-lightbox-frame=\"0\"]").await?;
    pointer::double_click(page, IMG).await?;
    let style = format!("document.querySelector({IMG:?}).style.cssText");
    wait::for_js_true(
        page,
        &format!("{style}.includes('scale(2)')"),
        "the picture at 2x",
    )
    .await?;

    // 40px below the centre at 2x is 20px of picture; at 4x it stays under the pointer
    // only with the picture's centre at dy -40 (the two clicks' pans would make it -200).
    let below = pointer::Point {
        x: frame.x,
        y: frame.y + 40.0,
    };
    for clicks in 1..=2 {
        press_at(page, below, MouseButton::Left, clicks).await?;
    }
    let at_4x = format!(
        "(() => {{ const [, dy] = {PAN_JS}; return {style}.includes('scale(4)') && Math.abs(dy + 40) <= 1; }})()"
    );
    if wait::for_js_true(page, &at_4x, "a double-click to 4x about the pointer")
        .await
        .is_err()
    {
        let seen: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
        let style: String = page.evaluate(style.as_str()).await?.into_value()?;
        bail!(
            "a double-click 40px below the centre at 2x left {seen:?}, style {style:?}, not dy -40 at 4x"
        );
    }

    // The zoom-in click renders after the right press, so its state is read settled.
    press_at(page, below, MouseButton::Right, 1).await?;
    pointer::click(page, ZOOM_IN).await?;
    wait::for_js_true(
        page,
        &format!("{style}.includes('scale(5)')"),
        "one wheel step in",
    )
    .await?;
    let state: String = page
        .evaluate(format!(
            "document.querySelector({IMG:?}).dataset.state ?? ''"
        ))
        .await?
        .into_value()?;
    if state.split(' ').any(|token| token == "dragging") {
        bail!("a right press left the picture dragging: data-state {state:?}");
    }
    Ok(())
}

const ZOOM_IN: &str = "[role=dialog] button[aria-label='Zoom in']";

/// The current picture's scale, read off its inline transform; 1 when fitted.
async fn scale<D: Driver>(d: &mut D) -> Result<f64> {
    let style = d.attr(PICTURE, "style").await?.unwrap_or_default();
    let Some((_, after)) = style.split_once("scale(") else {
        return Ok(1.0);
    };
    let number = after.split(')').next().unwrap_or_default();
    Ok(number.parse()?)
}

/// Opens the gallery and waits for its picture to take presses.
async fn open<D: Driver>(d: &mut D) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "the lightbox's picture", async |d| {
        d.exists(PICTURE).await
    })
    .await?;
    // The stage's first scroll settles before a gesture lands on it: the
    // picture is centred and holds still from one idle to the next.
    // The page's width: a classic scrollbar narrows it below the viewport's.
    let width = d.rect("html").await?.width;
    eventually(d, "the stage's first scroll to settle", async |d| {
        let before = d.rect(PICTURE).await?;
        d.idle().await;
        let after = d.rect(PICTURE).await?;
        let centre = after.x + after.width / 2.0;
        Ok(after.x == before.x && (centre - width / 2.0).abs() < 1.0)
    })
    .await
}

/// Todo 1068: a tap on "Zoom in" zooms (a WebView answered the frame query
/// with nothing).
async fn zoom_in_tapped<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    // Todo 1069: the toolbar is `sm`, `Button`'s 30px.
    let height = d.rect(ZOOM_IN).await?.height;
    if (height - 30.0).abs() > 0.5 {
        bail!("{:?}: zoom in is {height}px tall, not 30px", d.platform());
    }
    d.click(ZOOM_IN).await?;
    eventually(d, "one zoom step in", async |d| Ok(scale(d).await? == 1.25)).await
}

e2e::scenario!(
    a_tap_on_zoom_in_zooms_the_picture,
    "/lightbox",
    zoom_in_tapped,
    macos: skip("3432: WKWebView: the stage's first scroll never settles"),
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);

/// Todo 961: a zoomed picture pans only as far as its own edge, which needs its
/// natural size (a WebView answered none, so the bounds were the frame's).
async fn pan_stops_at_the_edge<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    eventually(d, "the first picture to decode", async |d| {
        let decoded = "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img')?.naturalWidth > 0";
        Ok(d.evaluate(decoded).await? == serde_json::json!(true))
    })
    .await?;
    for _ in 0..4 {
        d.click(ZOOM_IN).await?;
    }
    eventually(d, "four zoom steps in", async |d| {
        Ok((scale(d).await? - 1.25f64.powi(4)).abs() < 0.01)
    })
    .await?;
    d.drag(PICTURE, -300.0, -300.0).await?;
    let mut seen = [0.0f64; 4];
    eventually(d, "the drag to pan the picture", async |d| {
        seen = serde_json::from_value(d.evaluate(PAN_JS).await?)?;
        // A wide frame leaves no room on one axis; a pan on either will do.
        Ok(seen[0] != 0.0 || seen[1] != 0.0)
    })
    .await?;
    let [dx, dy, bx, by] = seen;
    ensure!(
        dx.abs() <= bx + 1.0 && dy.abs() <= by + 1.0,
        "{:?}: panned to ({dx}, {dy}), past the picture's edge ({bx}, {by})",
        d.platform()
    );
    Ok(())
}

e2e::scenario!(
    a_zoomed_picture_pans_only_to_its_edge,
    "/lightbox",
    pan_stops_at_the_edge,
    native: skip("3386: reads the picture's size and pan by script, which Blitz has none of"),
    macos: skip("3432: WKWebView: the stage's first scroll never settles"),
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);

/// Todo 1067: two fingers spread zoom the picture and do not close the viewer;
/// pinched back together it fits again.
async fn pinch_zooms<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    // 2.5x: no power of the wheel's 1.25, so no page pinch turned into wheel steps.
    d.pinch(PICTURE, 80.0, 200.0).await?;
    eventually(d, "the pinch to 2.5x", async |d| {
        Ok((scale(d).await? - 2.5).abs() < 0.01)
    })
    .await?;
    ensure_open(d).await?;
    d.pinch(PICTURE, 200.0, 40.0).await?;
    eventually(d, "the pinch back to fit", async |d| {
        Ok(scale(d).await? == 1.0)
    })
    .await?;
    ensure_open(d).await
}

async fn ensure_open<D: Driver>(d: &mut D) -> Result<()> {
    match d.exists(DIALOG).await? {
        true => Ok(()),
        false => bail!("{:?}: the pinch closed the lightbox", d.platform()),
    }
}

e2e::scenario!(
    a_pinch_zooms_the_picture,
    "/lightbox",
    pinch_zooms,
    native: skip("Blitz takes no multi-touch input"),
    desktop: skip("1126: no touch input under Xvfb"),
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);
