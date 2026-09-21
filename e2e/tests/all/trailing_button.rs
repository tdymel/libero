//! Clearable fields (620): Tab out past the x must close the list, via the control's blur,
//! unlike `ColorField`'s old input-only settle.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const CONTROL: &str = "[role=combobox]";
const CLEAR: &str = "[aria-label=Clear]";

#[test]
fn tab_out_past_the_clear_button_closes_the_list() {
    block_on(async {
        for route in [
            "/trailing-button/select",
            "/trailing-button/multi-select",
            "/trailing-button/autocomplete",
            "/trailing-button/cascader",
            "/trailing-button/tags-field",
        ] {
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
