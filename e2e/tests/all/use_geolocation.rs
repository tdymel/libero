//! `use_geolocation`: mounting asks nothing; on the web a granted fix, a
//! watch that follows it and a denial, from Chromium's emulated position.
//! Android locates from the emulator's GPS, Linux desktop is denied, Blitz unsupported.

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::browser::{
    PermissionDescriptor, PermissionSetting, SetPermissionParams,
};
use chromiumoxide::cdp::browser_protocol::emulation::SetGeolocationOverrideParams;
use e2e::browser::{PERMISSIONS, block_on};
use e2e::driver::{Driver, Platform, eventually, eventually_text};
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

/// Nothing prompts or locates until asked; Blitz has no Geolocation API.
async fn mounting_asks_nothing<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let supported = (d.platform() != Platform::Native).to_string();
    eventually_text(d, "#supported", &supported, "mount").await?;
    d.settle().await?;
    for (selector, expected) in [
        ("#position", "none"),
        ("#error", "None"),
        ("#pending", "false"),
        ("#watching", "false"),
    ] {
        eventually_text(d, selector, expected, "mount").await?;
    }
    match d.platform() {
        Platform::Native => {
            eventually_text(d, "#permission", "Unsupported", "mount").await?;
            d.click("#locate").await?;
            eventually_text(d, "#error", "Some(Unsupported)", "a request").await?;
        }
        // wry gives WebKitGTK no permission handler, so it denies every request.
        Platform::Desktop => {
            eventually_text(d, "#permission", "Prompt", "mount").await?;
            d.click("#locate").await?;
            eventually_text(d, "#error", "Some(Denied)", "a request").await?;
            eventually_text(d, "#permission", "Denied", "a denial").await?;
        }
        // The runner granted the permission; wry's prompt handler passes it on.
        // The WebView's Permissions API gives no answer, so a fix is what grants.
        Platform::Android => {
            eventually_text(d, "#permission", "Unknown", "mount").await?;
            d.click("#locate").await?;
            eventually(d, "a fix from the emulator's GPS", async |d| {
                fake_fix_on_android();
                Ok(d.text("#position").await?.starts_with("52.520, 13.405"))
            })
            .await?;
            eventually_text(d, "#permission", "Granted", "a fix").await?;
        }
        Platform::Web => {}
    }
    Ok(())
}

e2e::scenario!(
    mounting_asks_nothing_until_a_request,
    "/use-geolocation",
    mounting_asks_nothing
);

/// Feeds the emulator's GPS provider, the one a high-accuracy request reads.
fn fake_fix_on_android() {
    if let Ok(serial) = std::env::var("E2E_ANDROID_SERIAL") {
        let _ = std::process::Command::new("adb")
            .args(["-s", &serial, "emu", "geo", "fix", "13.405", "52.52"])
            .output();
    }
}

async fn set_permission(page: &Page, setting: PermissionSetting) -> Result<()> {
    let descriptor = PermissionDescriptor::new("geolocation");
    page.execute(SetPermissionParams::new(descriptor, setting))
        .await?;
    Ok(())
}

async fn move_to(page: &Page, latitude: f64, longitude: f64) -> Result<()> {
    page.execute(
        SetGeolocationOverrideParams::builder()
            .latitude(latitude)
            .longitude(longitude)
            .accuracy(20.0)
            .build(),
    )
    .await?;
    Ok(())
}

async fn reads(page: &Page, selector: &str, text: &str) -> Result<()> {
    let expression = format!("document.querySelector({selector:?})?.textContent === {text:?}");
    wait::for_js_true(page, &expression, &format!("{selector} to read {text}")).await
}

#[test]
fn the_web_locates_watches_and_reports_a_denial() {
    block_on(async {
        let _permissions = PERMISSIONS.lock().await;
        let fixture = Fixture::open("/use-geolocation", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        set_permission(page, PermissionSetting::Granted)
            .await
            .unwrap();
        move_to(page, 52.52, 13.405).await.unwrap();
        reads(page, "#permission", "Granted").await.unwrap();

        pointer::click(page, "#locate").await.unwrap();
        reads(page, "#position", "52.520, 13.405 ±20")
            .await
            .unwrap();
        reads(page, "#pending", "false").await.unwrap();

        pointer::click(page, "#watch").await.unwrap();
        reads(page, "#watching", "true").await.unwrap();
        move_to(page, 48.857, 2.352).await.unwrap();
        reads(page, "#position", "48.857, 2.352 ±20").await.unwrap();
        pointer::click(page, "#watch").await.unwrap();
        reads(page, "#watching", "false").await.unwrap();

        // The Permissions API reports the revoke; a request then fails as denied.
        set_permission(page, PermissionSetting::Denied)
            .await
            .unwrap();
        reads(page, "#permission", "Denied").await.unwrap();
        pointer::click(page, "#locate").await.unwrap();
        reads(page, "#error", "Some(Denied)").await.unwrap();
        reads(page, "#position", "48.857, 2.352 ±20").await.unwrap();

        set_permission(page, PermissionSetting::Prompt)
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("the geolocation hook")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
