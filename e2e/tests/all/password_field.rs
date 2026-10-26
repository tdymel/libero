//! `PasswordField`: the revealed input stays out of a browser's speller (4309), and a plain
//! `<form>` reset hides it again (4295).

use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const INPUT: &str = "[data-case=password] input";
const REVEAL: &str = "[data-case=password] [data-slot=trailing] button";

/// Todo 4309: a cloud speller must not see the secret once it is readable text.
#[test]
fn a_revealed_secret_keeps_spellcheck_correction_and_capitals_off() {
    block_on(async {
        let fixture = Fixture::open("/text-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        pointer::click(page, REVEAL).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({INPUT:?}).type === 'text'"),
            "the reveal click",
        )
        .await
        .unwrap();
        let attributes: String = page
            .evaluate_bounded(format!(
                "(() => {{ const el = document.querySelector({INPUT:?}); \
                 return ['spellcheck', 'autocorrect', 'autocapitalize'] \
                 .map(name => el.getAttribute(name)).join(','); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            attributes, "false,off,off",
            "the revealed input's attributes"
        );

        fixture.console.assert_clean("revealing a secret").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4295: with no libero `Form`, the owner `<form>`'s own reset hides the secret.
#[test]
fn a_plain_form_reset_hides_the_secret_again() {
    block_on(async {
        let fixture = Fixture::open("/text-field/raw-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let input = "[data-case=password-raw] input";
        let reveal = "[data-case=password-raw] [data-slot=trailing] button";
        let kind = format!("document.querySelector({input:?}).type");

        pointer::click(page, reveal).await.unwrap();
        wait::for_js_true(page, &format!("{kind} === 'text'"), "the reveal click")
            .await
            .unwrap();
        pointer::click(page, "[data-case=password-raw] [type=reset]")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{kind} === 'password'"), "the form reset")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("resetting a plain form")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
