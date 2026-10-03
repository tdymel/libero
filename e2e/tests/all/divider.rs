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
        assert_eq!(
            ax::snapshot(page, "#plain").await.unwrap(),
            "separator [orientation=horizontal]\n"
        );

        fixture.console.assert_clean("the divider fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A label's line halves drawn as background fills turn `Canvas` under forced colours.
#[test]
fn the_label_lines_show_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/divider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        // Each half: `drawn` if its fill or its edge differs from the page's `Canvas`.
        let halves: Vec<String> = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const ink = c => c !== canvas && !/^rgba\\(.*, 0\\)$/.test(c); \
                 return [['#labelled', 'Top'], ['#vertical', 'Left']].flatMap(([id, side]) => \
                   ['::before', '::after'].map(pseudo => { \
                     const s = getComputedStyle(document.querySelector(id), pseudo); \
                     const fill = ink(s.backgroundColor) \
                       && s.width !== '0px' && s.height !== '0px'; \
                     const edge = ink(s['border' + side + 'Color']) \
                       && s['border' + side + 'Style'] !== 'none' \
                       && s['border' + side + 'Width'] !== '0px'; \
                     return `${id}${pseudo} ${fill || edge ? 'drawn' : 'gone'}`; })); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            halves.iter().all(|half| half.ends_with("drawn")),
            "{halves:?}"
        );
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
