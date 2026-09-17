//! `TextField`, `PasswordField` and `Textarea`: what assistive technology hears
//! of the label, the captions and the status, and the reveal toggle.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{contrast, focus, keyboard};
use e2e::{Fixture, Suite, Viewport, ax, wait};

/// The caller's description first, then the captions in order; a rule's
/// message shows on blur and goes once the text passes it.
#[test]
fn the_captions_and_the_status_reach_assistive_technology() {
    block_on(async {
        let fixture = Fixture::open("/text-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        contrast::assert_clean(page, "[data-fixture-ready]")
            .await
            .unwrap();

        let wired = "[data-case=wired] input";
        assert_eq!(
            ax::description(page, wired).await.unwrap(),
            "Shown on your profile. How other people see you. Letters and digits. That name is taken."
        );
        let snapshot = ax::snapshot(page, "[data-case=wired]").await.unwrap();
        assert!(
            snapshot.contains(r#"textbox "Username" = ada [invalid] [required]"#),
            "{snapshot}"
        );

        let email = "[data-case=validated] input";
        keyboard::tab_to(page, email, 8).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        invalid(page, email, "Email needed").await.unwrap();
        keyboard::tab_to(page, email, 8).await.unwrap();
        keyboard::type_text(page, "a").await.unwrap();
        invalid(page, email, "").await.unwrap();

        // Uncontrolled: the rule judges what was typed.
        let bio = "[data-case=textarea] textarea";
        keyboard::tab_to(page, bio, 16).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        invalid(page, bio, "Markdown works. Bio needed")
            .await
            .unwrap();
        keyboard::tab_to(page, bio, 16).await.unwrap();
        keyboard::type_text(page, "hi").await.unwrap();
        invalid(page, bio, "Markdown works.").await.unwrap();

        fixture
            .console
            .assert_clean("describing text fields")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Enter and Space flip the secret's visibility, the button's `aria-pressed`
/// follows, and the focus stays on the button.
#[test]
fn the_reveal_button_toggles_by_keyboard() {
    block_on(async {
        let fixture = Fixture::open("/text-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let input = "[data-case=password] input";
        let button = "[data-case=password] button";

        focus::assert_focus_ring(page, input, 12).await.unwrap();
        keyboard::type_text(page, "hunter2").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::assert_focus_ring(page, button, 12).await.unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        revealed(page, "text", "true").await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        revealed(page, "password", "false").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({input:?}).value === 'hunter2'"),
            "the secret to survive the toggles",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("revealing a password")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A submit and a reset of the `Form` each hide a revealed secret again.
#[test]
fn a_submit_or_a_reset_hides_the_secret_again() {
    block_on(async {
        let fixture = Fixture::open("/text-field/form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let case = "[data-case=password-form]";
        let input = format!("{case} input");
        let reveal = format!("{case} [data-slot=trailing] button");

        keyboard::tab_to(page, &reveal, 32).await.unwrap();
        for trigger in ["[type=submit]", "[type=reset]"] {
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            secret_type(page, &input, "text").await.unwrap();
            let target = format!("{case} {trigger}");
            keyboard::tab_to(page, &target, 4).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            secret_type(page, &input, "password").await.unwrap();
            // Back from the submit button to the reveal button.
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        }

        fixture
            .console
            .assert_clean("hiding a password on submit and reset")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Axe, contrast, focus rings and the reveal button's target, light and dark,
/// desktop and mobile.
#[test]
fn it_meets_the_baseline() {
    Suite::new("text_field", "/text-field")
        .focusable("[data-case=wired] input")
        .focusable("[data-case=password] [data-slot=trailing] button")
        .focusable("[data-case=textarea] textarea")
        .targets("[data-case=password] [data-slot=trailing] button")
        .tab_budget(20)
        .run();
}

async fn secret_type(page: &Page, input: &str, kind: &str) -> Result<()> {
    wait::for_js_true(
        page,
        &format!("document.querySelector({input:?}).type === {kind:?}"),
        &format!("{input} type={kind}"),
    )
    .await
}

/// `aria-invalid` is set exactly when `description` names an error, and the
/// description reads `description`.
async fn invalid(page: &Page, selector: &str, description: &str) -> Result<()> {
    let error = !description.is_empty() && description.ends_with("needed");
    wait::for_js_true(
        page,
        &format!(
            "(document.querySelector({selector:?}).getAttribute('aria-invalid') === 'true') === {error}"
        ),
        &format!("{selector} invalid: {error}"),
    )
    .await?;
    let got = ax::description(page, selector).await?;
    anyhow::ensure!(got == description, "{selector} describes {got:?}");
    Ok(())
}

/// The input's type, and the focused button's static name and `aria-pressed`.
async fn revealed(page: &Page, kind: &str, pressed: &str) -> Result<()> {
    wait::for_js_true(
        page,
        &format!(
            "(() => {{ const el = document.activeElement; \
             return document.querySelector('[data-case=password] input').type === {kind:?} \
             && el.tagName === 'BUTTON' && el.getAttribute('aria-label') === 'Show password' \
             && el.getAttribute('aria-pressed') === {pressed:?}; }})()"
        ),
        &format!("type={kind} with the focus on the toggle, aria-pressed={pressed}"),
    )
    .await
}
