//! `TagsField`: the combobox archetype.

use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, ax, passes::keyboard, wait};

pub const TRIGGER: &str = "[role=combobox]";
const LISTBOX: &str = "[role=listbox]";
const OPTION_COUNT: usize = 4;

#[test]
fn it_meets_the_baseline() {
    Suite::new("tags_field", "/tags-field")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            LISTBOX,
        )
        .run();
}

#[test]
fn it_honours_the_combobox_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/tags-field", viewport).await.unwrap();

            Combobox {
                trigger: TRIGGER,
                option_count: OPTION_COUNT,
                tab_budget: 10,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the combobox contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The suggestion list is named by the field's label, like Autocomplete's
/// (todo 449).
#[test]
fn its_listbox_is_named_by_the_label() {
    block_on(async {
        let fixture = Fixture::open("/tags-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_visible(page, LISTBOX).await.unwrap();
        let tree = ax::snapshot(page, "body").await.unwrap();
        assert!(
            tree.contains(r#"listbox "Topics""#),
            "the suggestion list is not named by the label:\n{tree}"
        );
        fixture.close().await.unwrap();
    });
}

#[test]
fn typing_keeps_home_end_and_expanded_honest() {
    super::autocomplete::editable_combobox_typing("/tags-field", "s");
}

const CURSOR: &str = "#cursor";
const DELETE: keyboard::Key = keyboard::Key {
    key: "Delete",
    code: "Delete",
    vk: 46,
    text: None,
};
const BACKSPACE: keyboard::Key = keyboard::Key {
    key: "Backspace",
    code: "Backspace",
    vk: 8,
    text: None,
};

fn chip(label: &str) -> String {
    format!("button[aria-label='Remove {label}']")
}

async fn tags(page: &chromiumoxide::Page) -> Vec<String> {
    page.evaluate(
        "[...document.querySelectorAll(\"[data-slot='tag'] [data-slot='label']\")].map(e => e.textContent)",
    )
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// ArrowLeft from the empty input walks the chips, Delete/Backspace remove the
/// focused one and the focus moves on before it goes (todo 546).
#[test]
fn the_chip_cursor_walks_and_removes_tags() {
    use e2e::passes::focus::wait_for_focus;
    block_on(async {
        let fixture = Fixture::open("/tags-field/cursor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, CURSOR, 10).await.unwrap();

        let steps = [
            (keyboard::ARROW_LEFT, chip("css")),
            (keyboard::ARROW_LEFT, chip("wasm")),
            (keyboard::ARROW_LEFT, chip("dioxus")),
            (keyboard::ARROW_LEFT, chip("rust")),
            // The first chip is the end of the walk.
            (keyboard::ARROW_LEFT, chip("rust")),
            (keyboard::ARROW_RIGHT, chip("dioxus")),
        ];
        for (key, target) in &steps {
            keyboard::press(page, *key).await.unwrap();
            wait_for_focus(page, target, key.key).await.unwrap();
        }

        // The next tag slides into the removed one's place.
        keyboard::press(page, DELETE).await.unwrap();
        wait_for_focus(page, &chip("wasm"), "Delete").await.unwrap();
        assert_eq!(tags(page).await, ["rust", "wasm", "css"]);

        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        keyboard::press(page, BACKSPACE).await.unwrap();
        // The last tag gone, the one before it takes the cursor.
        wait_for_focus(page, &chip("wasm"), "Backspace on the last")
            .await
            .unwrap();
        assert_eq!(tags(page).await, ["rust", "wasm"]);

        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait_for_focus(page, CURSOR, "ArrowRight past the last")
            .await
            .unwrap();

        // Enter and Space are the x's own click: the focus still moves first.
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_for_focus(page, &chip("rust"), "Enter").await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait_for_focus(page, CURSOR, "Space on the only tag")
            .await
            .unwrap();
        assert!(tags(page).await.is_empty());

        fixture.console.assert_clean("the chip cursor").unwrap();
        fixture.close().await.unwrap();
    });
}
