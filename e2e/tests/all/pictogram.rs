//! `Pictogram`: hidden unless named, and a name must not be empty.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, ax};

/// An empty `aria_label` stays decorative: an unnamed `role="img"` is announced as "image".
#[test]
fn a_name_makes_an_image_and_an_empty_one_stays_hidden() {
    block_on(async {
        let fixture = Fixture::open("/pictogram", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let named = ax::snapshot(page, "#named").await.unwrap();
        assert!(named.starts_with("image \"Home\"\n"), "{named}");
        let labelled = ax::snapshot(page, "#labelled").await.unwrap();
        assert!(labelled.starts_with("image \"Starred\"\n"), "{labelled}");
        for id in ["decorative", "empty"] {
            let hidden = ax::snapshot(page, &format!("#{id}"))
                .await
                .unwrap_or_default();
            assert_eq!(hidden.trim(), "", "#{id} is in the tree: {hidden}");
        }

        fixture
            .console
            .assert_clean("the pictogram fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
