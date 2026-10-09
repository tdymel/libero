//! `Copy`: a press copies, the status says so, and leaving resets it.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, ax, wait};

const BARE: &str = "#bare button";
const NAMED: &str = "#named button";

#[test]
fn it_meets_the_baseline() {
    Suite::new("copy", "/copy")
        .focusable(BARE)
        .focusable(NAMED)
        .targets(BARE)
        .targets(NAMED)
        .run();
}

fn status_is(id: &str, text: &str) -> String {
    format!("document.querySelector('#{id} [role=status]').textContent === '{text}'")
}

/// Enter copies and the always-mounted status says "Copied"; focus stays and
/// the name does not change. Tabbing away empties the status again.
#[test]
fn a_press_is_announced_and_leaving_resets_it() {
    block_on(async {
        let fixture = Fixture::open("/copy", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, NAMED, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("named", "Copied"),
            "the copy to be announced",
        )
        .await
        .unwrap();
        let name: String = page
            .evaluate("document.activeElement.getAttribute('aria-label')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(name, "Copy the install command");

        // A second copy resets and announces again: the status empties, then fills.
        page.evaluate(
            "window.__mutations = 0; new MutationObserver(r => window.__mutations += r.length) \
             .observe(document.querySelector('#named [role=status]'), \
             { childList: true, characterData: true, subtree: true })",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && window.__mutations >= 2",
                status_is("named", "Copied")
            ),
            "the second copy to be announced again",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(page, &status_is("named", ""), "leaving to reset the status")
            .await
            .unwrap();

        // Unnamed, it takes the localization's name.
        pointer::click(page, BARE).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("bare", "Copied"),
            "the click to be announced",
        )
        .await
        .unwrap();
        let bare_name: String = page
            .evaluate(format!(
                "document.querySelector('{BARE}').getAttribute('aria-label')"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(bare_name, "Copy");

        // The pointer leaving resets it while focus stays.
        pointer::hover(page, NAMED).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("bare", ""),
            "the pointer leaving to reset the status",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("copying").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A disabled `Copy` is a disabled button that focus cannot reach.
#[test]
fn a_disabled_copy_cannot_be_focused() {
    block_on(async {
        let fixture = Fixture::open("/copy", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        let unfocusable: bool = page
            .evaluate(
                "(() => { const b = document.querySelector('#off button'); \
                 b.focus(); return b.disabled && document.activeElement !== b; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(unfocusable, "the disabled Copy took focus");

        fixture.console.assert_clean("disabled copy").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1025: `label` describes the button, its name stays short, and the words are not read twice.
#[test]
fn a_label_describes_the_button() {
    block_on(async {
        let fixture = Fixture::open("/copy", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        let tree = ax::snapshot(page, "#described").await.unwrap();
        assert!(tree.contains(r#"button "Copy""#), "{tree}");
        assert!(
            tree.contains("  description \"Add libero to your project\""),
            "{tree}"
        );
        // The description line is the only place the words may show.
        assert!(
            !tree
                .lines()
                .any(|line| line.contains("Add libero") && !line.contains("description ")),
            "read as text too:\n{tree}"
        );
        let description = ax::description(page, "#described button").await.unwrap();
        assert_eq!(description, "Add libero to your project");
        assert_eq!(ax::description(page, BARE).await.unwrap(), "");

        fixture.close().await.unwrap();
    });
}
