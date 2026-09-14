//! `Divider`: a separator named by its label, and a caller's decorative role.

use e2e::browser::block_on;
use e2e::{Fixture, Suite, Viewport, ax, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("divider", "/divider").run();
}

/// A separator's name comes only from the author, so a label without
/// `aria-labelledby` leaves it unnamed and is flattened in Firefox.
#[test]
fn a_label_names_its_separator() {
    block_on(async {
        let fixture = Fixture::open("/divider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#labelled").await.unwrap();

        let labelled = ax::snapshot(page, "#labelled").await.unwrap();
        assert!(
            labelled.starts_with("separator \"Or continue with\""),
            "{labelled}"
        );
        let vertical = ax::snapshot(page, "#vertical").await.unwrap();
        assert!(vertical.starts_with("separator \"Or\""), "{vertical}");
        assert_eq!(ax::snapshot(page, "#plain").await.unwrap(), "separator\n");

        fixture.console.assert_clean("the divider fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `role: "none"` from the caller marks a rule decorative.
#[test]
fn a_caller_role_replaces_separator() {
    block_on(async {
        let fixture = Fixture::open("/divider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#labelled").await.unwrap();

        let role: String = page
            .evaluate("document.querySelector('#decorative').getAttribute('role')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(role, "none");
        fixture.close().await.unwrap();
    });
}
