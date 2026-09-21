//! Use-case flows across several components (todo 824): what a user does from
//! the first click to the result, not one component's contract.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, Key};
use e2e::passes::{focus, pointer};
use e2e::{Fixture, Viewport, wait};

const BACKSPACE: Key = Key {
    key: "Backspace",
    code: "Backspace",
    vk: 8,
    text: None,
};

/// Marks the first `css` element whose trimmed text is `text` and returns a
/// selector for it.
async fn by_text(page: &Page, css: &str, text: &str) -> Result<String> {
    let mark = text.replace(|c: char| !c.is_ascii_alphanumeric(), "-");
    wait::for_js_true(
        page,
        &format!(
            "(() => {{ const el = [...document.querySelectorAll({css:?})] \
             .find((el) => el.textContent.trim() === {text:?}); \
             if (el) el.setAttribute('data-e2e', {mark:?}); return !!el; }})()"
        ),
        &format!("a {css} reading {text:?}"),
    )
    .await?;
    Ok(format!("[data-e2e={mark:?}]"))
}

/// `pointer::click`, after scrolling the element into view: a flow's page is
/// taller than the viewport.
async fn click(page: &Page, selector: &str) -> Result<()> {
    page.evaluate(format!(
        "document.querySelector({selector:?})?.scrollIntoView({{ block: 'center' }})"
    ))
    .await?;
    pointer::click(page, selector).await
}

async fn click_text(page: &Page, css: &str, text: &str) -> Result<()> {
    let selector = by_text(page, css, text).await?;
    click(page, &selector).await
}

/// Clicks into `selector`, empties it and types `text`.
async fn fill(page: &Page, selector: &str, text: &str) -> Result<()> {
    click(page, selector).await?;
    let length: usize = page
        .evaluate(format!("document.querySelector({selector:?}).value.length"))
        .await?
        .into_value()?;
    keyboard::press(page, keyboard::END).await?;
    for _ in 0..length {
        keyboard::press(page, BACKSPACE).await?;
    }
    keyboard::type_text(page, text).await
}

async fn texts(page: &Page, css: &str) -> Result<Vec<String>> {
    Ok(page
        .evaluate(format!(
            "[...document.querySelectorAll({css:?})].map((el) => el.textContent.trim())"
        ))
        .await?
        .into_value()?)
}

/// Waits until the `css` elements read `expected`, in order.
async fn wait_texts(page: &Page, css: &str, expected: &[&str], what: &str) {
    let json = serde_json::to_string(expected).unwrap();
    let settled = wait::for_js_true(
        page,
        &format!(
            "JSON.stringify([...document.querySelectorAll({css:?})] \
             .map((el) => el.textContent.trim())) === {json:?}"
        ),
        what,
    )
    .await;
    if settled.is_err() {
        panic!("{what}: {css} read {:?}", texts(page, css).await.unwrap());
    }
}

async fn data(page: &Page, id: &str, key: &str) -> String {
    page.evaluate(format!("document.getElementById({id:?}).dataset.{key}"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

const SUMMARY: &str = "[data-slot=summary] li";

/// A signup form end to end: submit empty, follow a summary link, fill every
/// field type, fix what a second submit still refuses, then submit the value.
#[test]
fn a_signup_form_is_filled_fixed_and_submitted() {
    block_on(async {
        let fixture = Fixture::open("/flows/signup", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        click_text(page, "button", "Sign up").await.unwrap();
        wait_texts(
            page,
            SUMMARY,
            &[
                "Name: Enter your name.",
                "Email: Enter your email.",
                "Age: You must be 18.",
                "Plan: Pick a plan.",
                "Contact me by: Pick a way to reach you.",
                "Street: Enter a street.",
                "I accept the terms: Accept the terms.",
            ],
            "an empty submit lists every required field",
        )
        .await;
        focus::wait_for_focus(page, "[data-slot=summary]", "an empty submit")
            .await
            .unwrap();

        click_text(
            page,
            SUMMARY.replace(" li", " li a").as_str(),
            "Age: You must be 18.",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, "input[name=age]", "following the age link")
            .await
            .unwrap();

        fill(page, "input[name=name]", "Ada Lovelace")
            .await
            .unwrap();
        fill(page, "input[name=email]", "ada@").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "document.body.textContent.includes('That is not an email address.') \
             && document.querySelector('input[name=email]').getAttribute('aria-invalid') === 'true'",
            "a half email to be refused on blur",
        )
        .await
        .unwrap();
        fill(page, "input[name=email]", "ada@example.com")
            .await
            .unwrap();
        fill(page, "input[name=age]", "17").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait_texts(
            page,
            SUMMARY,
            &[
                "Age: You must be 18.",
                "Plan: Pick a plan.",
                "Contact me by: Pick a way to reach you.",
                "Street: Enter a street.",
                "I accept the terms: Accept the terms.",
            ],
            "fixed fields drop out of the summary, a 17 keeps its line",
        )
        .await;
        fill(page, "input[name=age]", "36").await.unwrap();

        click(page, "[role=combobox]").await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        click_text(page, "[role=option]", "Team").await.unwrap();
        wait::for_hidden(page, "[role=option]").await.unwrap();
        click_text(page, "label", "Phone").await.unwrap();

        fill(page, "input[name='address.street']", "1 Analytical Way")
            .await
            .unwrap();
        fill(page, "input[name='address.city']", "London")
            .await
            .unwrap();
        fill(page, "textarea[name=notes]", "Ring twice, then wait")
            .await
            .unwrap();

        click_text(page, "button", "Sign up").await.unwrap();
        wait_texts(
            page,
            SUMMARY,
            &[
                "Notes: Keep notes under 20 characters.",
                "I accept the terms: Accept the terms.",
                "A city needs its zip code.",
            ],
            "the second submit lists what is still wrong",
        )
        .await;
        assert_eq!(data(page, "submits", "submits").await, "0");

        fill(page, "input[name='address.zip']", "N1").await.unwrap();
        fill(page, "textarea[name=notes]", "Ring twice")
            .await
            .unwrap();
        click_text(page, "label", "Newsletter").await.unwrap();
        click_text(page, "label", "I accept the terms")
            .await
            .unwrap();
        wait_texts(page, SUMMARY, &[], "every line fixed").await;

        click_text(page, "button", "Sign up").await.unwrap();
        wait::for_selector(page, "#submitted").await.unwrap();
        let submitted: String = page
            .evaluate("document.getElementById('submitted').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            submitted,
            "Signup { name: \"Ada Lovelace\", email: \"ada@example.com\", age: Some(36), \
             plan: Some(Team), contact: Some(Phone), address: Address { street: \"1 Analytical Way\", \
             zip: \"N1\", city: \"London\" }, notes: \"Ring twice\", newsletter: true, terms: true }"
        );
        assert_eq!(data(page, "submits", "submits").await, "1");

        fixture.console.assert_clean("the signup flow").unwrap();
        fixture.close().await.unwrap();
    });
}

const ROWS: &str = "table tbody tr";

async fn customers(page: &Page) -> Vec<String> {
    page.evaluate(format!(
        "[...document.querySelectorAll({ROWS:?})].map((tr) => tr.children[1]?.textContent.trim() ?? tr.textContent.trim())"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

async fn wait_customers(page: &Page, expected: &[&str], what: &str) {
    let json = serde_json::to_string(expected).unwrap();
    let settled = wait::for_js_true(
        page,
        &format!(
            "JSON.stringify([...document.querySelectorAll({ROWS:?})].map((tr) => \
             tr.children[1]?.textContent.trim() ?? tr.textContent.trim())) === {json:?}"
        ),
        what,
    )
    .await;
    if settled.is_err() {
        panic!("{what}: rows read {:?}", customers(page).await);
    }
}

/// Totals of the page's rows, in drawn order.
async fn totals(page: &Page) -> Vec<u32> {
    let cells: Vec<String> = page
        .evaluate(format!(
            "[...document.querySelectorAll({ROWS:?})].map((tr) => tr.children[2].textContent.trim())"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    cells.iter().map(|cell| cell.parse().unwrap()).collect()
}

/// Filter, page and sort one orders table: a filter goes back to page 1 and
/// shrinks the pager, no match shows the empty row, and a sort holds per page.
#[test]
fn an_orders_table_is_filtered_paged_and_sorted() {
    block_on(async {
        let fixture = Fixture::open("/flows/orders", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let filter = "input";

        wait_customers(
            page,
            &["Ada", "Alan", "Barbara", "Claude", "Dana"],
            "page 1",
        )
        .await;
        click(page, "[aria-label=\"Go to page 3\"]").await.unwrap();
        wait_customers(page, &["Joan", "Ken"], "page 3").await;

        fill(page, filter, "an").await.unwrap();
        wait_customers(
            page,
            &["Alan", "Dana", "Frances", "Ivan", "Joan"],
            "a filter back on page 1",
        )
        .await;
        assert_eq!(data(page, "matching", "matching").await, "5");
        wait::for_js_true(
            page,
            "!document.querySelector('[aria-label=\"Go to page 2\"]') \
             && document.querySelector('[aria-current=page]').textContent === '1'",
            "one page left",
        )
        .await
        .unwrap();

        keyboard::type_text(page, "n").await.unwrap();
        wait_customers(page, &["No orders match"], "no match").await;
        keyboard::press(page, BACKSPACE).await.unwrap();
        wait_customers(
            page,
            &["Alan", "Dana", "Frances", "Ivan", "Joan"],
            "the filter widened again",
        )
        .await;

        fill(page, filter, "").await.unwrap();
        wait_customers(
            page,
            &["Ada", "Alan", "Barbara", "Claude", "Dana"],
            "no filter",
        )
        .await;
        click_text(page, "th[data-sortable] button", "Total")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('th[aria-sort]')?.getAttribute('aria-sort') === 'ascending'",
            "the Total column to sort ascending",
        )
        .await
        .unwrap();
        wait_customers(
            page,
            &["Ada", "Claude", "Alan", "Dana", "Barbara"],
            "page 1 by total",
        )
        .await;
        assert_eq!(totals(page).await, [10, 31, 47, 68, 84]);

        // The sort is the table's own state, so it holds for the next page's rows.
        click(page, "[aria-label=\"Go to page 2\"]").await.unwrap();
        wait_customers(
            page,
            &["Edsger", "Hedy", "Frances", "Ivan", "Grace"],
            "page 2 by total",
        )
        .await;
        assert_eq!(totals(page).await, [15, 36, 52, 73, 89]);

        fixture.console.assert_clean("the orders flow").unwrap();
        fixture.close().await.unwrap();
    });
}

const TRIGGER: &str = "#add-member";
const DIALOG: &str = "[role=dialog]";

/// A form in a dialog: an invalid save keeps it open, a valid one adds the member and
/// returns focus; Escape adds nothing and the next opening starts empty.
#[test]
fn a_dialog_form_refuses_saves_and_dismisses() {
    block_on(async {
        let fixture = Fixture::open("/flows/dialog-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let name = "[role=dialog] input[name=name]";

        click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        click_text(page, "[role=dialog] button", "Save member")
            .await
            .unwrap();
        wait_texts(
            page,
            SUMMARY,
            &["Member name: Enter a name.", "Member plan: Pick a plan."],
            "an empty save",
        )
        .await;
        assert!(
            wait::is_visible(page, DIALOG).await.unwrap(),
            "an invalid save closed the dialog"
        );

        fill(page, name, "Grace").await.unwrap();
        click(page, "[role=dialog] [role=combobox]").await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        click_text(page, "[role=option]", "Enterprise")
            .await
            .unwrap();
        wait::for_hidden(page, "[role=option]").await.unwrap();
        click_text(page, "[role=dialog] button", "Save member")
            .await
            .unwrap();
        wait::for_hidden(page, DIALOG).await.unwrap();
        wait_texts(
            page,
            "#members li",
            &["Grace (Enterprise)"],
            "the saved member",
        )
        .await;
        focus::wait_for_focus(page, TRIGGER, "saving the dialog")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        let empty: String = page
            .evaluate(format!("document.querySelector({name:?}).value"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(empty, "", "a new opening starts with an empty form");
        fill(page, name, "Hedy").await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, DIALOG).await.unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('dismissed').dataset.dismissed === '1'",
            "Escape to dismiss",
        )
        .await
        .unwrap();
        wait_texts(
            page,
            "#members li",
            &["Grace (Enterprise)"],
            "a dismissal adds nothing",
        )
        .await;
        focus::wait_for_focus(page, TRIGGER, "dismissing the dialog")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("the dialog form flow")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
