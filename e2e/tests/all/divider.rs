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
        // The caller's own `role: "separator"` keeps the label name; their name wins.
        for (id, name) in [
            ("#same-role", "Advanced"),
            ("#named", "Billing"),
            ("#named-by", "Account"),
        ] {
            let tree = ax::snapshot(page, id).await.unwrap();
            assert!(
                tree.starts_with(&format!("separator \"{name}\"")),
                "{id}: {tree}"
            );
        }

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

        // `aria-orientation` is not allowed on `none` (axe `aria-allowed-attr`).
        let attributes: Vec<String> = page
            .evaluate(
                "['#decorative', '#decorative-vertical'].map(id => { \
                   const rule = document.querySelector(id); \
                   return `${id} ${rule.getAttribute('role')} ${rule.getAttribute('aria-orientation')}`; })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            attributes,
            ["#decorative none null", "#decorative-vertical none null"]
        );
        fixture.close().await.unwrap();
    });
}

/// `label_position` gives the leading or trailing half its `10%` basis.
#[test]
fn the_label_position_moves_the_label() {
    block_on(async {
        let fixture = Fixture::open("/divider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#end").await.unwrap();

        let sides: Vec<String> = page
            .evaluate(
                "['#start', '#labelled', '#end'].map(id => { \
                   const rule = document.querySelector(id); \
                   const half = pseudo => parseFloat(getComputedStyle(rule, pseudo).width); \
                   const before = half('::before'), after = half('::after'); \
                   const side = Math.abs(before - after) < 1 ? 'centre' : before < after ? 'start' : 'end'; \
                   return `${id} ${side}`; })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(sides, ["#start start", "#labelled centre", "#end end"]);
        fixture.close().await.unwrap();
    });
}

/// A label longer than the 320px column wraps inside the rule (1.4.10, todo 2471).
#[test]
fn a_long_label_wraps_inside_its_column() {
    block_on(async {
        let fixture = Fixture::open("/divider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#long").await.unwrap();

        let fit: String = page
            .evaluate(
                "(() => { const rule = document.querySelector('#long'); \
                   const label = rule.querySelector('[data-slot=label]'); \
                   const r = rule.getBoundingClientRect(), l = label.getBoundingClientRect(); \
                   const s = getComputedStyle(label); \
                   const line = parseFloat(s.lineHeight) || parseFloat(s.fontSize) * 1.2; \
                   const inside = l.left >= r.left - 0.5 && l.right <= r.right + 0.5 \
                     && rule.scrollWidth <= rule.clientWidth; \
                   return `inside ${inside}, wrapped ${l.height > line * 1.5}`; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(fit, "inside true, wrapped true");
        fixture.close().await.unwrap();
    });
}
