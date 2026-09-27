//! `use_user_media`: mounting asks nothing; on the web Chromium's fake camera
//! opens, shows, snapshots, records, stops and is refused. Blitz is unsupported.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::browser::{
    PermissionDescriptor, PermissionSetting, SetPermissionParams,
};
use e2e::browser::{PERMISSIONS, block_on};
use e2e::driver::{Driver, Platform, eventually, eventually_text, linger};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

/// Nothing prompts or opens until asked; Blitz has no capture API.
async fn mounting_asks_nothing<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let supported = (d.platform() != Platform::Native).to_string();
    eventually_text(d, "#supported", &supported, "mount").await?;
    linger(d, 3).await;
    for (selector, expected) in [
        ("#live", "false"),
        ("#pending", "false"),
        ("#error", "None"),
        ("#photo", "none"),
    ] {
        eventually_text(d, selector, expected, "mount").await?;
    }
    match d.platform() {
        Platform::Native => {
            eventually_text(d, "#camera", "Unsupported", "mount").await?;
            d.click("#start").await?;
            eventually_text(d, "#error", "Some(Unsupported)", "a start").await?;
        }
        // wry gives WebKitGTK no permission handler, so it denies every request.
        Platform::Desktop => {
            d.click("#start").await?;
            eventually_text(d, "#error", "Some(Denied)", "a start").await?;
            eventually_text(d, "#camera", "Denied", "a denial").await?;
            fullscreen_beside_the_capture(d).await?;
            d.press(keyboard::ESCAPE).await?;
            eventually_text(d, "#is-fullscreen", "false", "Escape").await?;
        }
        // The runner revoked the permission: the first start asks, and opens once
        // allowed, with no reload (1346). The emulator's virtual camera answers.
        Platform::Android => {
            d.click("#audio").await?;
            d.allow_permission("#start").await?;
            eventually_text(d, "#live", "true", "the first start after a grant").await?;
            eventually_text(d, "#camera", "Granted", "a grant").await?;
            d.click("#snapshot").await?;
            eventually_text(d, "#photo", "photo.png true", "a snapshot").await?;
            d.click("#record").await?;
            eventually_text(d, "#recording", "true", "a recording").await?;
            // At least one 1 s chunk crosses the IPC before the end.
            std::thread::sleep(std::time::Duration::from_millis(1500));
            d.click("#finish").await?;
            eventually_text(d, "#recorded", "recording.webm true", "a finish").await?;
            switch_cameras(d).await?;
            d.click("#stop").await?;
            eventually_text(d, "#live", "false", "a stop").await?;
            record_audio_only(d).await?;
            // Native here, found by the watch's selector; the watch leaves it on Escape (1256).
            fullscreen_beside_the_capture(d).await?;
            eventually(d, "native fullscreen", async |d| {
                Ok(d.attr("#preview", "data-fullscreen").await?.as_deref() == Some("native"))
            })
            .await?;
            d.press(keyboard::ESCAPE).await?;
            eventually_text(d, "#is-fullscreen", "false", "Escape").await?;
        }
        Platform::Web => {}
    }
    Ok(())
}

/// The grant lists the emulator's two cameras with no refresh; a switch reopens
/// the stream on the other one, and back (1347).
async fn switch_cameras<D: Driver>(d: &mut D) -> Result<()> {
    eventually(d, "two listed cameras", async |d| {
        Ok(d.text("#cameras").await?.parse::<u32>().unwrap_or(0) >= 2)
    })
    .await?;
    let first = d.text("#camera-id").await?;
    if first.is_empty() {
        bail!("{:?}: no live camera id", d.platform());
    }
    // A running recording finishes into a clip before the switch.
    d.click("#record").await?;
    eventually_text(d, "#recorded", "none", "a recording start").await?;
    std::thread::sleep(std::time::Duration::from_millis(1500));
    d.click("#switch").await?;
    eventually(d, "the other camera", async |d| {
        let id = d.text("#camera-id").await?;
        Ok(!id.is_empty() && id != first && d.text("#live").await? == "true")
    })
    .await?;
    eventually_text(d, "#recording", "false", "a switch").await?;
    eventually_text(d, "#recorded", "recording.webm true", "a switch").await?;
    d.click("#switch").await?;
    eventually_text(d, "#camera-id", &first, "a switch back").await
}

/// The microphone alone records an audio clip (1321); the camera comes back on after.
async fn record_audio_only<D: Driver>(d: &mut D) -> Result<()> {
    d.click("#camera-toggle").await?;
    d.click("#start").await?;
    eventually_text(d, "#live", "true", "an audio-only start").await?;
    d.click("#record").await?;
    eventually_text(d, "#recording", "true", "an audio recording").await?;
    std::thread::sleep(std::time::Duration::from_millis(1500));
    d.click("#finish").await?;
    eventually_text(d, "#recording", "false", "an audio finish").await?;
    eventually(d, "an audio clip", async |d| {
        Ok(d.text("#recorded-type").await?.starts_with("audio/"))
    })
    .await?;
    d.click("#stop").await?;
    eventually_text(d, "#live", "false", "an audio stop").await?;
    d.click("#camera-toggle").await
}

/// A WebView finds each hook's element by its own attribute: `use_fullscreen`'s
/// tag on the preview leaves the capture's in place (1237). The video then
/// covers the page, the toggle included.
async fn fullscreen_beside_the_capture<D: Driver>(d: &mut D) -> Result<()> {
    for name in ["data-lsx-capture", "data-lsx-observe"] {
        if d.attr("#preview", name).await?.is_none() {
            bail!("{:?}: the preview lost its {name}", d.platform());
        }
    }
    d.click("#fullscreen").await?;
    eventually_text(d, "#is-fullscreen", "true", "entering fullscreen").await
}

e2e::scenario!(
    mounting_asks_nothing_until_a_start,
    "/use-user-media",
    mounting_asks_nothing
);

async fn set_permission(page: &Page, setting: PermissionSetting) -> Result<()> {
    for name in ["camera", "microphone"] {
        page.execute(SetPermissionParams::new(
            PermissionDescriptor::new(name),
            setting.clone(),
        ))
        .await?;
    }
    Ok(())
}

async fn reads(page: &Page, selector: &str, text: &str) -> Result<()> {
    let expression = format!("document.querySelector({selector:?})?.textContent === {text:?}");
    let read = wait::for_js_true(page, &expression, &format!("{selector} to read {text}")).await;
    if read.is_ok() {
        return read;
    }
    let state: String = page
        .evaluate("[...document.querySelectorAll('p[id]')].map((p) => p.id + '=' + p.textContent).join(' ')")
        .await?
        .into_value()?;
    read.map_err(|error| error.context(format!("the fixture reads {state}")))
}

async fn click(page: &Page, selector: &str) -> Result<()> {
    page.find_element(selector).await?.click().await?;
    Ok(())
}

#[test]
fn the_web_shows_snapshots_records_and_stops_the_camera() {
    block_on(async {
        let _permissions = PERMISSIONS.lock().await;
        let fixture = Fixture::open("/use-user-media", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        set_permission(page, PermissionSetting::Granted)
            .await
            .unwrap();
        reads(page, "#camera", "Granted").await.unwrap();

        click(page, "#audio").await.unwrap();
        click(page, "#start").await.unwrap();
        reads(page, "#live", "true").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#preview').videoWidth > 0",
            "the preview to play",
        )
        .await
        .unwrap();

        click(page, "#snapshot").await.unwrap();
        reads(page, "#photo", "photo.png true").await.unwrap();

        click(page, "#record").await.unwrap();
        reads(page, "#recording", "true").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        click(page, "#finish").await.unwrap();
        reads(page, "#recording", "false").await.unwrap();
        reads(page, "#recorded", "recording.webm true")
            .await
            .unwrap();

        // The grant fills in the device labels, with no refresh.
        reads(page, "#devices-supported", "true").await.unwrap();
        wait::for_js_true(
            page,
            "Number(document.querySelector('#labelled').textContent) > 0",
            "a labelled fake camera",
        )
        .await
        .unwrap();

        // Stopping ends every track and empties the preview.
        reads(page, "#live", "true").await.unwrap();
        wait::for_js_true(
            page,
            "!!(window.__tracks = document.querySelector('#preview').srcObject?.getTracks())",
            "the preview to hold the stream",
        )
        .await
        .unwrap();
        click(page, "#stop").await.unwrap();
        reads(page, "#live", "false").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#preview').srcObject === null \
             && window.__tracks.every((track) => track.readyState === 'ended')",
            "every track to end",
        )
        .await
        .unwrap();

        // The microphone alone records an audio clip (1321).
        click(page, "#camera-toggle").await.unwrap();
        click(page, "#start").await.unwrap();
        reads(page, "#live", "true").await.unwrap();
        click(page, "#record").await.unwrap();
        reads(page, "#recording", "true").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        click(page, "#finish").await.unwrap();
        reads(page, "#recording", "false").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#recorded-type').textContent.startsWith('audio/')",
            "an audio clip",
        )
        .await
        .unwrap();
        click(page, "#stop").await.unwrap();
        click(page, "#camera-toggle").await.unwrap();

        set_permission(page, PermissionSetting::Denied)
            .await
            .unwrap();
        reads(page, "#camera", "Denied").await.unwrap();
        click(page, "#start").await.unwrap();
        reads(page, "#error", "Some(Denied)").await.unwrap();
        reads(page, "#live", "false").await.unwrap();

        set_permission(page, PermissionSetting::Prompt)
            .await
            .unwrap();
        fixture.console.assert_clean("the user media hook").unwrap();
        fixture.close().await.unwrap();
    });
}
