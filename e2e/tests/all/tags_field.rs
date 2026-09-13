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
