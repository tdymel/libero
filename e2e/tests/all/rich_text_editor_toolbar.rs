//! `RichTextEditor` toolbar, menus, announcements, labels and read-only states on the web.

use super::rich_text_editor::{
    EDITOR, KEY_B, TOOLBAR_ROWS, TRAILING, announced, editor_status, out, out_eq, out_is, press,
    select, select_first_leaf, until,
};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, CTRL, ENTER, Key};
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, js, wait};

#[test]
fn a_caller_toolbar_runs_commands_and_reads_state_through_the_handle() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/handle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let undo_disabled = "document.getElementById('ext-undo').disabled";
        assert!(js::<bool>(page, undo_disabled).await);

        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;
        select_first_leaf(page).await;
        pointer::click(page, "#ext-bold").await.unwrap();
        out_eq(page, "**ab**\n").await;
        wait::for_js_true(
            page,
            &format!(
                "document.getElementById('ext-bold').getAttribute('aria-pressed') === 'true' && !{undo_disabled}"
            ),
            "the handle's state: bold, undo enabled",
        )
        .await
        .unwrap();

        pointer::click(page, "#ext-undo").await.unwrap();
        out_eq(page, "ab\n").await;
        fixture.close().await.unwrap();
    });
}

const KEY_K: Key = Key {
    key: "k",
    code: "KeyK",
    vk: 75,
    text: None,
};

const SLASH: Key = Key {
    key: "/",
    code: "Slash",
    vk: 191,
    text: None,
};

#[test]
fn link_dialog_shortcut_help_block_menu_and_announcements() {
    const DIALOG: &str = "document.querySelector('[role=dialog]')";
    const HEADING: &str = "[...document.querySelectorAll('[role=menuitemradio]')].find(i => i.textContent.includes('Heading 2'))";
    const TRIGGER: &str = "document.querySelector('[role=toolbar] button[aria-haspopup]')";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;

        // Mod+B from the keyboard is announced, on and off.
        editor_status(page).await;
        select_first_leaf(page).await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        announced(page, "Bold on").await;
        keyboard::press_with(page, KEY_B, CTRL).await.unwrap();
        out_eq(page, "ab\n").await;
        announced(page, "Bold off").await;

        // Mod+K: an unsafe scheme is a field error, a safe one links the selection.
        select_first_leaf(page).await;
        keyboard::press_with(page, KEY_K, CTRL).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement === document.querySelector('[role=dialog] input')",
            "focus in the link field",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "javascript:alert(1)")
            .await
            .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=dialog] input').getAttribute('aria-invalid') === 'true'",
            "the unsafe link to be a field error",
        )
        .await
        .unwrap();
        page.evaluate(
            "(() => { const i = document.querySelector('[role=dialog] input'); i.select(); })()",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "https://example.com")
            .await
            .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        out_eq(page, "[ab](https://example.com)\n").await;
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "focus back in the text",
        )
        .await
        .unwrap();

        // Mod+/ lists the keymap.
        keyboard::press_with(page, SLASH, CTRL).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "!!{DIALOG}?.textContent.includes('Keyboard shortcuts') && {DIALOG}.textContent.includes('Heading 2')"
            ),
            "the keymap listed",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "the keymap closed, focus back in the text",
        )
        .await
        .unwrap();

        // The block-type menu turns the paragraph into a heading.
        press(page, TRIGGER).await;
        wait::for_js_true(page, &format!("!!{HEADING}"), "the block menu")
            .await
            .unwrap();
        press(page, HEADING).await;
        out_eq(page, "## [ab](https://example.com)\n").await;
        wait::for_js_true(
            page,
            &format!("{TRIGGER}.textContent === 'Heading 2'"),
            "the menu trigger to name the heading",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

const MORE: &str = "document.querySelector('[role=toolbar] [aria-label=\"More formatting\"]')";

#[test]
fn a_narrow_toolbar_keeps_one_row_and_moves_the_rest_into_more() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        assert!(until(page, MORE).await, "no More trigger");
        // [no horizontal overflow, distinct item rows]
        let (overflow, rows): (bool, u32) = js(
            page,
            "(() => { const t = document.querySelector('[role=toolbar]'); \
               const tops = new Set([...t.querySelectorAll('button')].map(b => Math.round(b.getBoundingClientRect().top))); \
               return [t.scrollWidth > t.clientWidth, tops.size]; })()",
        )
        .await;
        assert!(!overflow, "the toolbar runs out of its column");
        assert_eq!(rows, 1, "the toolbar wraps");

        // A hidden toggle runs from the menu, on the caret's block.
        select(
            page,
            &format!(
                "const e = {EDITOR}; e.focus(); const l = e.querySelector('[data-key]'); getSelection().selectAllChildren(l); getSelection().collapseToEnd()"
            ),
        )
        .await;
        press(page, MORE).await;
        let quote = "[...document.querySelectorAll('[role=menuitemcheckbox]')].find(i => i.textContent.includes('Quote'))";
        assert!(until(page, quote).await, "no Quote in the More menu");
        press(page, quote).await;
        let quoted = until(
            page,
            "document.getElementById('out').textContent.startsWith('> above')",
        )
        .await;
        assert!(quoted, "{:?}", out(page).await);
        fixture.close().await.unwrap();
    });
}

/// Larger icons than the default theme's: the bar measures them and hides more (1259).
#[test]
fn a_toolbar_with_larger_icons_still_keeps_one_row() {
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        assert!(until(page, MORE).await, "no More trigger");
        let (_, _, before): (bool, u32, u32) = js(page, TOOLBAR_ROWS).await;
        page.evaluate(
            "(() => { const s = document.createElement('style'); \
               s.textContent = '[role=toolbar] button:not([aria-haspopup]) { min-width: 56px; }'; \
               document.head.append(s); })()",
        )
        .await
        .unwrap();
        let fewer = format!("{TOOLBAR_ROWS}[2] < {before}");
        assert!(until(page, &fewer).await, "nothing moved into More");
        let (overflow, rows, _): (bool, u32, u32) = js(page, TOOLBAR_ROWS).await;
        assert!(!overflow, "the toolbar runs out of its column");
        assert_eq!(rows, 1, "the toolbar wraps");
        fixture.close().await.unwrap();
    });
}

const FAR: &str = "/rich-text-editor/far";

/// Waits for the editor to hold focus and sit inside the viewport.
async fn editor_in_view(page: &Page, what: &str) {
    wait::for_js_true(
        page,
        &format!(
            "(() => {{ const e = {EDITOR}, r = e.getBoundingClientRect(); \
               return document.activeElement === e && r.top >= 0 && r.top < innerHeight; }})()"
        ),
        what,
    )
    .await
    .unwrap();
}

#[test]
fn the_handle_scrolls_a_far_editor_into_view_when_it_focuses_it() {
    block_on(async {
        let fixture = Fixture::open(FAR, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "[role=textbox]").await.unwrap();

        for (button, what) in [
            ("far-focus", "focus() to bring the editor into view"),
            ("far-insert", "edit() to bring the editor into view"),
            ("far-clear", "clear() to bring the editor into view"),
        ] {
            page.evaluate("document.activeElement.blur(); scrollTo(0, 0)")
                .await
                .unwrap();
            let top: f64 = js(page, "scrollY").await;
            assert_eq!(top, 0.0);
            js::<bool>(
                page,
                format!("document.getElementById('{button}').click(), true"),
            )
            .await;
            editor_in_view(page, what).await;
        }
        fixture.close().await.unwrap();
    });
}

#[test]
fn toolbar_buttons_show_their_name_and_chord_in_a_tooltip() {
    const BOLD: &str = "[role=toolbar] button[aria-label=\"Bold\"]";
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, BOLD).await.unwrap();
        let shortcut: String = js(
            page,
            &format!("document.querySelector('{BOLD}').getAttribute('aria-keyshortcuts')"),
        )
        .await;
        assert_eq!(shortcut, "Control+B");
        pointer::hover(page, BOLD).await.unwrap();
        wait::for_js_true(
            page,
            "!!document.querySelector('[role=tooltip]')",
            "a tooltip",
        )
        .await
        .unwrap();
        let text: String = js(
            page,
            "document.querySelector('[role=tooltip]').textContent.replace(/\\s+/g, ' ').trim()",
        )
        .await;
        assert_eq!(text, "Bold Ctrl + B");
        fixture.close().await.unwrap();
    });
}

#[test]
fn the_text_is_named_by_its_label_and_a_label_click_focuses_it() {
    block_on(async {
        let fixture = Fixture::open(TRAILING, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &format!("!!{EDITOR}"), "the editor")
            .await
            .unwrap();
        let tree = e2e::ax::snapshot(page, "[role=textbox]").await.unwrap();
        let first = tree.lines().next().unwrap_or_default();
        assert!(first.contains("textbox \"Snippet\""), "{first}");

        pointer::click(page, "label").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "the label click to focus the text",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_readonly_editor_is_a_tab_stop_and_a_disabled_one_says_so() {
    const READONLY: &str = "document.querySelector('#readonly [role=textbox]')";
    const DISABLED: &str = "document.querySelector('#disabled [role=textbox]')";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &format!("!!{DISABLED}"), "both editors")
            .await
            .unwrap();
        // [read-only tabindex, disabled tabindex, disabled aria-disabled]
        let (readonly, disabled, state): (String, Option<String>, String) = js(
            page,
            &format!(
                "[{READONLY}.getAttribute('tabindex'), {DISABLED}.getAttribute('tabindex'), {DISABLED}.getAttribute('aria-disabled')]"
            ),
        )
        .await;
        assert_eq!(readonly, "0");
        assert_eq!(disabled, None);
        assert_eq!(state, "true");

        // The label reaches the read-only text, as Tab does.
        pointer::click(page, "#readonly label").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {READONLY}"),
            "the read-only text to take focus",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3001: a caller's toggle says its new state, as the builtins do.
#[test]
fn a_custom_toggle_is_announced() {
    const LOUD: &str = "[role=toolbar] button[aria-label=\"Loud\"]";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/custom-toggle", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        editor_status(page).await;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        pointer::click(page, LOUD).await.unwrap();
        announced(page, "Loud on").await;
        pointer::click(page, LOUD).await.unwrap();
        announced(page, "Loud off").await;
        fixture.close().await.unwrap();
    });
}

#[test]
fn toolbar_and_menu_actions_are_announced() {
    const BULLETS: &str = "[role=toolbar] button[aria-label=\"Bulleted list\"]";
    const HEADING: &str = "[...document.querySelectorAll('[role=menuitemradio]')].find(i => i.textContent.includes('Heading 3'))";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        editor_status(page).await;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        wait::for_js_true(page, &out_is("ab\n"), "the typed text")
            .await
            .unwrap();

        pointer::click(page, BULLETS).await.unwrap();
        announced(page, "Bulleted list on").await;
        pointer::click(page, BULLETS).await.unwrap();
        announced(page, "Bulleted list off").await;

        pointer::click(page, "[role=toolbar] button[aria-haspopup]")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("!!{HEADING}"), "the block menu")
            .await
            .unwrap();
        press(page, HEADING).await;
        announced(page, "Heading 3").await;
        fixture.close().await.unwrap();
    });
}

/// Todo 1795: the toolbar is one tab stop the arrows walk, Tab leaves it for the text, and
/// the block-type menu runs from the keyboard.
#[test]
fn the_toolbar_and_its_block_menu_work_from_the_keyboard() {
    const ITEMS: &str =
        "[...document.querySelectorAll('[role=toolbar] button')].filter(b => b.offsetParent)";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("{EDITOR}.focus()")).await.unwrap();
        keyboard::type_text(page, "ab").await.unwrap();
        out_eq(page, "ab\n").await;
        let at = |index: &str| format!("{ITEMS}.indexOf(document.activeElement) === {index}");

        page.evaluate("document.activeElement.blur()")
            .await
            .unwrap();
        keyboard::tab_to(page, "[role=toolbar] button", 10)
            .await
            .unwrap();
        for (key, index, what) in [
            (keyboard::ARROW_RIGHT, "1", "ArrowRight to the second tool"),
            (
                keyboard::END,
                &format!("{ITEMS}.length - 1"),
                "End to the last tool",
            ),
            (keyboard::HOME, "0", "Home to the first tool"),
        ] {
            keyboard::press(page, key).await.unwrap();
            wait::for_js_true(page, &at(index), what).await.unwrap();
        }
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "Tab from the toolbar to the text",
        )
        .await
        .unwrap();

        // Back on the remembered tool, then right to the block-type trigger.
        keyboard::press_with(page, keyboard::TAB, keyboard::SHIFT)
            .await
            .unwrap();
        wait::for_js_true(page, &at("0"), "Shift+Tab back to the first tool")
            .await
            .unwrap();
        let trigger: usize = js(
            page,
            format!("{ITEMS}.findIndex(b => b.hasAttribute('aria-haspopup'))"),
        )
        .await;
        for step in 1..=trigger {
            keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
            wait::for_js_true(page, &at(&step.to_string()), "ArrowRight along the toolbar")
                .await
                .unwrap();
        }
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.getAttribute('role') === 'menuitemradio'",
            "Enter to open the block menu with focus in it",
        )
        .await
        .unwrap();
        let on_heading =
            "document.activeElement?.textContent.includes('Heading 2') ?? false".to_string();
        for _ in 0..8 {
            if js::<bool>(page, &on_heading).await {
                break;
            }
            let before: String = js(page, "document.activeElement.textContent").await;
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_js_true(
                page,
                &format!("document.activeElement.textContent !== {before:?}"),
                "ArrowDown to the next block type",
            )
            .await
            .unwrap();
        }
        assert!(js::<bool>(page, &on_heading).await, "no Heading 2 in reach");
        keyboard::press(page, ENTER).await.unwrap();
        out_eq(page, "## ab\n").await;
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "focus back in the text after the pick",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1795: on a phone the overflow tools sit in More, whose menu the arrows walk and
/// Escape closes back onto its trigger.
#[test]
fn the_more_menu_works_from_the_keyboard_on_a_phone() {
    const ACTIVE_ROLE: &str = "document.activeElement?.getAttribute('role')";
    block_on(async {
        let fixture = Fixture::open("/rich-text-editor/code", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        assert!(until(page, MORE).await, "no More trigger");
        keyboard::tab_to(page, "[role=toolbar] button", 10)
            .await
            .unwrap();
        keyboard::press(page, keyboard::END).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {MORE}"),
            "End to the More trigger",
        )
        .await
        .unwrap();
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{ACTIVE_ROLE} === 'menuitemcheckbox'"),
            "Enter to open More with focus in it",
        )
        .await
        .unwrap();
        let first: String = js(page, "document.activeElement.textContent").await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{ACTIVE_ROLE} === 'menuitemcheckbox' && document.activeElement.textContent !== {first:?}"
            ),
            "ArrowDown to the next hidden tool",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {MORE}"),
            "Escape back onto More",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === {EDITOR}"),
            "Tab from the toolbar to the text",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
