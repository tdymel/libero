//! `use_fcm` (todo 3404): on the web it is unsupported; on Android the Kotlin
//! service's events reach the handle over JNI, and Firebase answers a register.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// No Firebase bridge on the web: unsupported, and a register says so.
#[test]
fn the_web_has_no_firebase_bridge() {
    block_on(async {
        let fixture = Fixture::open("/fcm", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::click(page, "#register").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#error').textContent === 'Some(Unsupported)'",
            "the register to be refused",
        )
        .await
        .unwrap();
        let supported: String = page
            .evaluate_bounded("document.querySelector('#supported').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(supported, "false");

        fixture
            .console
            .assert_clean("an unsupported register")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A token and a message sent down the service's path reach the handle.
async fn the_service_events_reach_the_handle<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the bridge to mount", async |d| {
        Ok(d.text("#supported").await? == "true")
    })
    .await?;
    d.click("#probe").await?;
    eventually(d, "the probe's token and message", async |d| {
        Ok(d.text("#token").await? == "probe-token" && d.text("#message").await? == "Probe hello")
    })
    .await?;
    Ok(())
}

/// Placeholder options: Firebase starts, refuses the key, and the error reaches the handle.
async fn firebase_answers_a_register<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the bridge to mount", async |d| {
        Ok(d.text("#supported").await? == "true")
    })
    .await?;
    d.click("#register").await?;
    eventually(d, "Firebase's refusal", async |d| {
        Ok(d.text("#error").await?.contains("API key") && d.text("#pending").await? == "false")
    })
    .await?;
    Ok(())
}

e2e::scenario!(
    android_the_service_events_reach_the_handle,
    "/fcm",
    the_service_events_reach_the_handle,
    android_only("3404: the Kotlin service is Android's")
);
e2e::scenario!(
    android_firebase_answers_a_register,
    "/fcm",
    firebase_answers_a_register,
    android_only("3404: the Kotlin service is Android's")
);
