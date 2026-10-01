//! Clearable fields (620): Tab out past the x must close the list, via the control's blur,
//! unlike `ColorField`'s old input-only settle.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, ax, wait};

const CONTROL: &str = "[role=combobox]";
const CLEAR: &str = "[aria-label=Clear]";

fn control(route: &str) -> &'static str {
    match route.ends_with("tags-field") {
        true => "[data-frame] input",
        false => CONTROL,
    }
}

async fn the_x_is_gone<D: Driver>(d: &mut D) -> Result<()> {
    eventually(d, "the x to go with the value", async |d| {
        Ok(!d.exists(CLEAR).await?)
    })
    .await
}

/// A press on the x empties the field and hands the focus to its control.
async fn a_press_clears_and_refocuses<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    d.click(CLEAR).await?;
    the_x_is_gone(d).await?;
    eventually_focused(d, control(route), "a press on the x").await
}

/// Enter on the focused x does the same.
async fn enter_clears_and_refocuses<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    d.focus(control(route)).await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, CLEAR, "Tab from the control").await?;
    d.press(keyboard::ENTER).await?;
    the_x_is_gone(d).await?;
    eventually_focused(d, control(route), "Enter on the x").await
}

e2e::scenario!(
    a_press_on_the_select_x_clears_it,
    "/trailing-button/select",
    a_press_clears_and_refocuses
);
e2e::scenario!(
    a_press_on_the_autocomplete_x_clears_it,
    "/trailing-button/autocomplete",
    a_press_clears_and_refocuses
);
e2e::scenario!(
    a_press_on_the_tags_x_clears_it,
    "/trailing-button/tags-field",
    a_press_clears_and_refocuses
);
e2e::scenario!(
    enter_on_the_select_x_clears_it,
    "/trailing-button/select",
    enter_clears_and_refocuses
);
e2e::scenario!(
    enter_on_the_cascader_x_clears_it,
    "/trailing-button/cascader",
    enter_clears_and_refocuses
);

#[test]
fn tab_out_past_the_clear_button_closes_the_list() {
    const ROUTES: [&str; 5] = [
        "/trailing-button/select",
        "/trailing-button/multi-select",
        "/trailing-button/autocomplete",
        "/trailing-button/cascader",
        "/trailing-button/tags-field",
    ];
    // The pages run at once, as `Suite`'s do.
    block_on(async {
        let _pages = e2e::browser::page_permit(ROUTES.len()).await;
        futures::future::join_all(ROUTES.map(async |route| {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, CONTROL, 5).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            expect(
                page,
                "!!document.querySelector('[aria-expanded=true]')",
                &format!("{route}: Arrow Down to open the list"),
            )
            .await;
            keyboard::tab_to(page, CLEAR, 3).await.unwrap();
            keyboard::press(page, keyboard::TAB).await.unwrap();
            expect(
                page,
                "document.activeElement.id === 'after' \
                 && !document.querySelector('[aria-expanded=true]')",
                &format!("{route}: Tab from the x on to the button after, the list closed"),
            )
            .await;

            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }))
        .await;
    });
}

/// Todo 1498: the x borrows the field's label, "Clear Fruit"; with no label it stays "Clear".
#[test]
fn the_x_is_named_with_its_fields_label() {
    block_on(async {
        for (route, name) in [
            ("/trailing-button/select", "Clear Fruit"),
            ("/trailing-button/multi-select", "Clear Fruit"),
            ("/trailing-button/autocomplete", "Clear City"),
            ("/trailing-button/cascader", "Clear Place"),
            ("/trailing-button/tags-field", "Clear Topics"),
            ("/trailing-button/unlabelled", "Clear"),
        ] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            wait::for_visible(page, CLEAR).await.unwrap();
            let tree = ax::snapshot(page, CLEAR).await.unwrap();
            let first = tree.lines().next().unwrap_or_default();
            assert_eq!(first, format!("button {name:?}"), "{route}: {tree}");
            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

async fn expect(page: &chromiumoxide::Page, check: &str, what: &str) {
    if let Err(error) = wait::for_js_true(page, check, what).await {
        let focus: String = page
            .evaluate("document.activeElement.outerHTML.slice(0, 160)")
            .await
            .and_then(|value| Ok(value.into_value()?))
            .unwrap_or_default();
        panic!("{what}: {error}; focus={focus}");
    }
}
