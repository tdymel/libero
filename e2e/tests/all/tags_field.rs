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
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
    });
}

/// The browser draws the `placeholder` itself; the frame's native stand-in
/// (todo 734) never reaches the web DOM.
#[test]
fn the_web_draws_its_own_placeholder() {
    block_on(async {
        let fixture = Fixture::open("/tags-field", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!(
                "!!document.querySelector('{TRIGGER}') \
                 && !document.querySelector('[data-lsx-placeholder]')"
            ),
            "the field and no stand-in span",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
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

/// Todo 1947: a paste that adds one tag and rejects the next hands the reason
/// after the change, so a handler that clears on change keeps the rejection.
#[test]
fn a_rejection_arrives_after_the_change_with_its_reason() {
    block_on(async {
        let fixture = Fixture::open("/tags-field/reject", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#reject", 10).await.unwrap();
        keyboard::insert_text(page, "rust,wasm,").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector(\"[data-slot='tag'] [data-slot='label']\")?.textContent === 'rust' \
             && document.querySelector('#rejected').textContent === 'wasm:Full'",
            "rust added and wasm rejected",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a mixed paste").unwrap();
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

/// Todo 545: Enter on a refused tag keeps the draft and says why.
#[test]
fn a_refused_tag_stays_in_the_draft_and_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/tags-field/cursor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, CURSOR, 10).await.unwrap();
        keyboard::type_text(page, "WASM").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('[role=status]')].some(e => e.textContent === 'Already added: WASM')",
            "the refusal to be announced",
        )
        .await
        .unwrap();
        let draft: String = page
            .evaluate(format!("document.querySelector({CURSOR:?}).value"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(draft, "WASM", "the refused text stays in the draft");
        assert_eq!(tags(page).await, ["rust", "dioxus", "wasm", "css"]);

        fixture.console.assert_clean("a refused tag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 482: a draft matching no suggestion shows "No results" and says it.
#[test]
fn a_draft_matching_no_suggestion_is_shown_and_said() {
    block_on(async {
        let fixture = Fixture::open("/tags-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::type_text(page, "zzz").await.unwrap();
        crate::autocomplete::wait_for_nothing_found(page, "/tags-field").await;
        fixture.console.assert_clean("nothing found").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 666: ArrowLeft with the caret at the start of a draft enters the chips
/// and leaves the draft uncommitted; ArrowRight past the last chip returns to it.
#[test]
fn the_chip_cursor_enters_from_the_start_of_a_draft() {
    use e2e::passes::focus::wait_for_focus;
    block_on(async {
        let fixture = Fixture::open("/tags-field/cursor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, CURSOR, 10).await.unwrap();
        keyboard::type_text(page, "go").await.unwrap();
        let draft = || async {
            page.evaluate(format!("document.querySelector({CURSOR:?}).value"))
                .await
                .unwrap()
                .into_value::<String>()
                .unwrap()
        };

        // Two presses walk the caret to the start; the third leaves.
        for _ in 0..2 {
            keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        }
        wait_for_focus(page, CURSOR, "ArrowLeft inside the draft")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait_for_focus(page, &chip("css"), "ArrowLeft at the draft's start")
            .await
            .unwrap();
        assert_eq!(draft().await, "go", "the draft was committed or dropped");
        assert_eq!(tags(page).await, ["rust", "dioxus", "wasm", "css"]);

        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait_for_focus(page, CURSOR, "ArrowRight past the last")
            .await
            .unwrap();
        assert_eq!(draft().await, "go");

        fixture
            .console
            .assert_clean("entering from a draft")
            .unwrap();
        fixture.close().await.unwrap();
    });
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
