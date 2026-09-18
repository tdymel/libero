//! `PhoneField`: the country picker, its searchable list and the `tel` input.

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::{Fixture, Viewport, ax, passes::keyboard, passes::pointer, wait};

const PICKER: &str = "button[aria-haspopup=listbox]";
const SEARCH: &str = "input[role=combobox]";
const TEL: &str = "input[type=tel]";

/// The highlighted row's text, or `None`.
async fn highlighted(page: &Page) -> Option<String> {
    page.evaluate(format!(
        "(s => {{ const id = s && s.getAttribute('aria-activedescendant'); \
         return id ? document.getElementById(id).textContent : null; }})(document.querySelector({SEARCH:?}))"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

async fn wait_open(page: &Page, how: &str) {
    wait::for_js_true(
        page,
        &format!("document.activeElement === document.querySelector({SEARCH:?})"),
        &format!("{how} to open the list and focus its search box"),
    )
    .await
    .unwrap();
}

async fn wait_closed(page: &Page) {
    wait::for_js_true(
        page,
        &format!("document.activeElement === document.querySelector({PICKER:?})"),
        "Escape to close the list and return the focus to the button",
    )
    .await
    .unwrap();
}

/// Todo 763: the country button was 82x21 px, under WCAG 2.5.8's 24x24.
#[test]
fn the_country_button_is_at_least_24px_square() {
    block_on(async {
        let fixture = Fixture::open("/phone-field", Viewport::Desktop)
            .await
            .unwrap();
        e2e::passes::target_size::assert_minimum(&fixture.page, PICKER)
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// APG: the list opens on the current country, by key or by click, however it
/// was left the last time.
#[test]
fn the_list_opens_on_the_current_country() {
    block_on(async {
        let fixture = Fixture::open("/phone-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, PICKER, 10).await.unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_open(page, "Enter").await;
        assert_eq!(
            highlighted(page).await.as_deref(),
            Some("Germany+49"),
            "Enter"
        );
        // Leave the highlight elsewhere before closing.
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait_closed(page).await;

        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait_open(page, "ArrowDown").await;
        assert_eq!(
            highlighted(page).await.as_deref(),
            Some("Germany+49"),
            "ArrowDown"
        );
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait_closed(page).await;

        pointer::click(page, PICKER).await.unwrap();
        wait_open(page, "a click").await;
        assert_eq!(
            highlighted(page).await.as_deref(),
            Some("Germany+49"),
            "click"
        );

        fixture.console.assert_clean("/phone-field").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 4.1.2: the listbox has a name; 2.5.3: the button's name holds what it shows.
#[test]
fn the_button_and_its_listbox_are_named() {
    block_on(async {
        let fixture = Fixture::open("/phone-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, PICKER, 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_open(page, "Enter").await;

        let tree = ax::snapshot(page, "body").await.unwrap();
        assert!(
            tree.contains(r#"button "Country: Germany, DE +49""#),
            "the button's name lacks its visible text:\n{tree}"
        );
        assert!(
            tree.contains(r#"listbox "Country: Germany, DE +49""#),
            "the listbox is not named by the button:\n{tree}"
        );

        fixture.close().await.unwrap();
    });
}

/// Read-only: the picker stays a tab stop, is announced as unavailable, and no
/// key or click opens it (todo 558).
#[test]
fn a_read_only_picker_is_focusable_but_opens_nothing() {
    block_on(async {
        let fixture = Fixture::open("/phone-field/readonly", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, PICKER, 10).await.unwrap();
        for key in [keyboard::ENTER, keyboard::SPACE, keyboard::ARROW_DOWN] {
            keyboard::press(page, key).await.unwrap();
        }
        pointer::click(page, PICKER).await.unwrap();

        let tree = ax::snapshot(page, "body").await.unwrap();
        assert!(
            tree.contains(r#"button "Country: Germany, DE +49" [disabled]"#),
            "the read-only picker is not announced as unavailable:\n{tree}"
        );
        assert!(
            tree.contains("[expanded=false]"),
            "the list opened:\n{tree}"
        );
        assert!(!tree.contains("listbox \""), "the list opened:\n{tree}");
        let focused: bool = page
            .evaluate(format!(
                "document.activeElement === document.querySelector({PICKER:?})"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(focused, "the picker lost the focus");

        fixture.close().await.unwrap();
    });
}

/// A query ranks the names starting with it first, so Enter picks France for
/// "fr", not the Central African Republic.
#[test]
fn a_query_ranks_names_starting_with_it_first() {
    block_on(async {
        let fixture = Fixture::open("/phone-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, PICKER, 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_open(page, "Enter").await;
        keyboard::type_text(page, "fr").await.unwrap();
        assert_eq!(highlighted(page).await.as_deref(), Some("France+33"));

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.activeElement === document.querySelector({PICKER:?}) \
                 && document.activeElement.getAttribute('aria-label').startsWith('Country: France')"
            ),
            "Enter to pick France and return the focus",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("/phone-field").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 793: the list sorts by the shown, German name, an umlaut with its base
/// letter: Deutschland among the D's, Österreich among the O's.
#[test]
fn the_list_sorts_by_the_localized_name() {
    block_on(async {
        let fixture = Fixture::open("/phone-field/german", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, PICKER, 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_open(page, "Enter").await;
        let names: Vec<String> = page
            .evaluate("[...document.querySelectorAll('[role=option] [data-slot=name]')].map(n => n.textContent)")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let initial = |name: &str| {
            let c = name.chars().next().unwrap().to_lowercase().next().unwrap();
            match c {
                'ä' | 'å' => 'a',
                'ö' => 'o',
                'ü' => 'u',
                c => c,
            }
        };
        let initials: Vec<char> = names.iter().map(|name| initial(name)).collect();
        assert!(names.len() > 200, "the whole list: {}", names.len());
        assert!(
            initials.windows(2).all(|pair| pair[0] <= pair[1]),
            "initials in order: {names:?}"
        );
        for (name, letter) in [("Deutschland", 'd'), ("Österreich", 'o'), ("Ägypten", 'a')] {
            let at = names.iter().position(|n| n == name).expect(name);
            assert_eq!(initials[at], letter, "{name}");
        }

        fixture.console.assert_clean("/phone-field/german").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 547: the country search's Home and End edit the query.
#[test]
fn home_and_end_edit_the_country_search() {
    crate::select::search_home_end_edit_the_query("/phone-field", PICKER, "fr");
}

/// The caret stays where the user typed: nothing regroups while typing, and a
/// fixed plan groups on blur. The E.164 follows every keystroke.
#[test]
fn typing_keeps_the_caret_and_blur_groups() {
    block_on(async {
        let fixture = Fixture::open("/phone-field/error", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TEL, 10).await.unwrap();
        keyboard::type_text(page, "213734253").await.unwrap();
        page.evaluate(format!(
            "document.querySelector({TEL:?}).setSelectionRange(3, 3)"
        ))
        .await
        .unwrap();
        keyboard::type_text(page, "3").await.unwrap();
        let read = format!("(t => [t.value, t.selectionStart])(document.querySelector({TEL:?}))");
        let typed: (String, usize) = page
            .evaluate(read.as_str())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            typed,
            ("2133734253".to_string(), 4),
            "a digit typed mid-number"
        );

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({TEL:?}).value === '213 373 4253'"),
            "blur to group a US number",
        )
        .await
        .unwrap();

        let wiring: Vec<Option<String>> = page
            .evaluate(format!(
                "(t => ['type', 'inputmode', 'autocomplete', 'aria-invalid', 'aria-required', 'required'] \
                 .map(name => t.getAttribute(name)))(document.querySelector({TEL:?}))"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            wiring,
            [
                Some("tel".to_string()),
                Some("tel".to_string()),
                Some("tel-national".to_string()),
                Some("true".to_string()),
                Some("true".to_string()),
                Some("true".to_string()),
            ],
            "type, inputmode, autocomplete, aria-invalid, aria-required, required"
        );
        let tree = ax::snapshot(page, "body").await.unwrap();
        assert!(
            tree.contains(r#"textbox "Mobile" = 213 373 4253 [invalid] [required]"#),
            "the tel input's name and states:\n{tree}"
        );

        fixture.console.assert_clean("/phone-field/error").unwrap();
        fixture.close().await.unwrap();
    });
}
