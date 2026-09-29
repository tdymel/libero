//! `use_system_notification` and `use_push_subscription`: mounting asks nothing;
//! on the web a granted show and close, a denial, and push registering its worker.
//! Android's dialog, posted notification and tap (1348); push is web only.

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::browser::{
    PermissionDescriptor, PermissionSetting, SetPermissionParams,
};
use e2e::browser::{PERMISSIONS, block_on};
use e2e::driver::{Driver, Platform, eventually, eventually_text, linger};
use e2e::{Fixture, Viewport, wait};

/// Nothing prompts until asked; elsewhere than the web, push is unsupported.
async fn mounting_asks_nothing<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let web = d.platform() == Platform::Web;
    eventually_text(d, "#push-supported", &web.to_string(), "mount").await?;
    linger(d, 3).await;
    for (selector, expected) in [
        ("#error", "None"),
        ("#pending", "false"),
        ("#push-error", "None"),
        ("#subscription", "false"),
    ] {
        eventually_text(d, selector, expected, "mount").await?;
    }
    match d.platform() {
        // Todo 1348: the system's notifications over JNI; the runner revokes the permission.
        Platform::Android => {
            eventually_text(d, "#supported", "true", "mount").await?;
            eventually_text(d, "#permission", "Prompt", "mount").await?;
            d.allow_permission("#request").await?;
            eventually_text(d, "#permission", "Granted", "the dialog's answer").await?;
            d.click("#show").await?;
            eventually(d, "the notification to post", async |d| {
                Ok(d.posted_notifications().await?.contains("tag=fixture"))
            })
            .await?;
            eventually_text(d, "#error", "None", "a show").await?;
            d.tap_notification("Fixture").await?;
            eventually_text(d, "#clicks", "1", "a tap on the notification").await?;
            // The first tap after the tap's relaunch of the activity can go nowhere.
            eventually(d, "the second notification to post", async |d| {
                d.click("#show").await?;
                linger(d, 20).await;
                Ok(d.posted_notifications().await?.contains("tag=fixture"))
            })
            .await?;
            d.click("#close").await?;
            eventually(d, "the close to cancel it", async |d| {
                Ok(!d.posted_notifications().await?.contains("tag=fixture"))
            })
            .await?;
            d.click("#subscribe").await?;
            eventually_text(d, "#push-error", "Some(Unsupported)", "a subscribe").await?;
        }
        // Blitz on Linux asks the session's notification server; without one it is unsupported.
        Platform::Native => {
            eventually(d, "the notification server's answer", async |d| {
                Ok(d.text("#permission").await? != "Unknown")
            })
            .await?;
            if d.text("#supported").await? == "true" {
                eventually_text(d, "#permission", "Granted", "mount").await?;
                return Ok(());
            }
            eventually_text(d, "#supported", "false", "mount").await?;
            eventually_text(d, "#permission", "Unsupported", "mount").await?;
            d.click("#show").await?;
            eventually_text(d, "#error", "Some(Unsupported)", "a show").await?;
            d.click("#subscribe").await?;
            eventually_text(d, "#push-error", "Some(Unsupported)", "a subscribe").await?;
        }
        // WebKitGTK has a `Notification`, but wry answers no permission request.
        Platform::Desktop => {
            eventually_text(d, "#supported", "true", "mount").await?;
            d.click("#request").await?;
            eventually_text(d, "#error", "Some(Denied)", "a request").await?;
            eventually_text(d, "#permission", "Denied", "a denial").await?;
        }
        Platform::Web => {
            eventually_text(d, "#supported", "true", "mount").await?;
        }
    }
    Ok(())
}

e2e::scenario!(
    mounting_asks_nothing_until_a_request,
    "/use-system-notification",
    mounting_asks_nothing
);

async fn set_permission(page: &Page, setting: PermissionSetting) -> Result<()> {
    let descriptor = PermissionDescriptor::new("notifications");
    page.execute(SetPermissionParams::new(descriptor, setting))
        .await?;
    Ok(())
}

async fn reads(page: &Page, selector: &str, text: &str) -> Result<()> {
    let expression = format!("document.querySelector({selector:?})?.textContent === {text:?}");
    wait::for_js_true(page, &expression, &format!("{selector} to read {text}")).await
}

async fn click(page: &Page, selector: &str) {
    page.find_element(selector)
        .await
        .unwrap()
        .click()
        .await
        .unwrap();
}

#[test]
fn the_web_shows_closes_and_reports_a_denial() {
    block_on(async {
        let _permissions = PERMISSIONS.lock().await;
        let fixture = Fixture::open("/use-system-notification", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        set_permission(page, PermissionSetting::Granted)
            .await
            .unwrap();
        reads(page, "#permission", "Granted").await.unwrap();

        click(page, "#request").await;
        reads(page, "#pending", "false").await.unwrap();
        reads(page, "#error", "None").await.unwrap();
        // Keeps each shown notification so a click can be dispatched on it.
        page.evaluate(
            "window.shown = []; window.focused = 0; window.focus = () => window.focused++;
            window.Notification = class extends Notification {
                constructor(title, options) { super(title, options); window.shown.push(this); }
            };",
        )
        .await
        .unwrap();
        click(page, "#show").await;
        // A failed show would land in `#error` once its promise settles.
        page.evaluate("new Promise((done) => setTimeout(done, 500))")
            .await
            .unwrap();
        reads(page, "#error", "None").await.unwrap();
        page.evaluate("window.shown.at(-1).dispatchEvent(new Event('click'))")
            .await
            .unwrap();
        reads(page, "#clicks", "1").await.unwrap();
        let focused = page
            .evaluate("window.focused")
            .await
            .unwrap()
            .into_value::<u32>()
            .unwrap();
        assert_eq!(focused, 1, "a click focuses the window");
        click(page, "#close").await;

        // The Permissions API reports the revoke; a show then fails as denied.
        set_permission(page, PermissionSetting::Denied)
            .await
            .unwrap();
        reads(page, "#permission", "Denied").await.unwrap();
        click(page, "#show").await;
        reads(page, "#error", "Some(Denied)").await.unwrap();

        // The worker registers; headless Chromium then refuses the subscription itself.
        click(page, "#subscribe").await;
        reads(page, "#push-error", "Some(Denied)").await.unwrap();
        reads(page, "#push-pending", "false").await.unwrap();
        let registered = page
            .evaluate("navigator.serviceWorker.getRegistration().then((r) => !!r)")
            .await
            .unwrap()
            .into_value::<bool>()
            .unwrap();
        assert!(registered, "the worker registers before the subscription");

        // Todo 1338: without a `Notification` constructor the worker shows it, and
        // the click the worker posts back runs `on_click`; another token is ignored.
        set_permission(page, PermissionSetting::Granted)
            .await
            .unwrap();
        reads(page, "#permission", "Granted").await.unwrap();
        // As in Chrome on Android, where the constructor throws.
        page.evaluate(
            "window.focused = 0; window.focus = () => window.focused++;
            window.Notification = class extends Notification {
                constructor() { throw new TypeError('Illegal constructor'); }
            };",
        )
        .await
        .unwrap();
        click(page, "#show").await;
        let latest = "navigator.serviceWorker.getRegistration()
            .then((r) => r.getNotifications())
            .then((shown) => shown.find((one) => one.data?.libero)?.data.libero ?? '')";
        let expression = format!("{latest}.then((token) => token !== '')");
        // Polling `getNotifications` at once never sees it in headless Chromium (cause unknown).
        page.evaluate("new Promise((done) => setTimeout(done, 1000))")
            .await
            .unwrap();
        wait::for_js_true(page, &expression, "the worker's notification")
            .await
            .unwrap();
        reads(page, "#error", "None").await.unwrap();

        // What the sample `sw.js` posts on a click.
        let post = |libero: &str| {
            format!(
                "{latest}.then((token) => navigator.serviceWorker.dispatchEvent(
                    new MessageEvent('message', {{ data: {{ libero: {libero}, event: 'click' }} }})))"
            )
        };
        page.evaluate(post("'another'")).await.unwrap();
        page.evaluate(post("token")).await.unwrap();
        reads(page, "#clicks", "2").await.unwrap();
        let focused = page
            .evaluate("window.focused")
            .await
            .unwrap()
            .into_value::<u32>()
            .unwrap();
        assert_eq!(focused, 1, "a worker click focuses the window");

        click(page, "#close").await;
        let gone = "navigator.serviceWorker.getRegistration()
            .then((r) => r.getNotifications()).then((shown) => shown.length === 0)";
        wait::for_js_true(page, gone, "the close to reach the worker's notification")
            .await
            .unwrap();

        set_permission(page, PermissionSetting::Prompt)
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("the system notification hooks")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
