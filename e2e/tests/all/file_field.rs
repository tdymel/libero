//! `FileField`: a real drop through CDP's drag plumbing, and where focus goes
//! once the file it was on is removed (todo 406).

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchDragEventParams, DispatchDragEventType, DragData, DragDataItem,
};
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

/// The dropzone surface: the only `role="button"` the fixture draws.
const SURFACE: &str = "[data-fixture-ready] [role=button]";
const REMOVE_ALPHA: &str = "[aria-label=\"Remove alpha.txt\"]";
const REMOVE_BETA: &str = "[aria-label=\"Remove beta.txt\"]";
/// The `Input` variant's control, which holds the chips.
const CONTROL: &str = "#attachments";

/// Todo 483: the label focuses the surface it names by id.
#[test]
fn a_click_on_the_label_focuses_the_surface() {
    crate::select::label_click_focuses("/file-field", SURFACE);
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("file_field", "/file-field")
        .focusable(SURFACE)
        .targets(SURFACE)
        .run();
}

/// Real files on disk, so the drop carries a `FileList` the page can read.
fn files_on_disk() -> Vec<String> {
    let dir = std::env::temp_dir().join(format!("e2e-file-field-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    ["alpha.txt", "beta.txt"]
        .iter()
        .map(|name| {
            let path = dir.join(name);
            std::fs::write(&path, format!("{name} contents")).unwrap();
            path.to_string_lossy().into_owned()
        })
        .collect()
}

/// Enter, over, drop: what the browser sends for an OS file dragged onto the
/// surface. Trusted events, unlike a `DataTransfer` built in the page.
async fn drop_files(page: &Page, selector: &str, files: Vec<String>) {
    let at = pointer::centre_of(page, selector).await.unwrap();
    for kind in [
        DispatchDragEventType::DragEnter,
        DispatchDragEventType::DragOver,
        DispatchDragEventType::Drop,
    ] {
        // CDP requires `items`, and chromiumoxide drops an empty list.
        let data = DragData::builder()
            .item(DragDataItem::new("text/plain", "files"))
            .files(files.clone())
            .drag_operations_mask(1)
            .build()
            .unwrap();
        let params = DispatchDragEventParams::builder()
            .r#type(kind.clone())
            .x(at.x)
            .y(at.y)
            .data(data)
            .build()
            .unwrap();
        page.execute(params)
            .await
            .unwrap_or_else(|e| panic!("dispatching {kind:?}: {e}"));
    }
}

async fn open_with_two_files() -> Fixture {
    let fixture = Fixture::open("/file-field", Viewport::Desktop)
        .await
        .unwrap();
    wait::for_visible(&fixture.page, SURFACE).await.unwrap();
    drop_files(&fixture.page, SURFACE, files_on_disk()).await;
    wait::for_selector(&fixture.page, REMOVE_BETA)
        .await
        .unwrap();
    fixture
}

/// Todo 509: Ctrl/Alt/Meta+ArrowLeft/Right leave the chip cursor alone; the
/// plain arrow still moves it.
#[test]
fn modifier_chords_leave_the_chip_cursor_alone() {
    block_on(async {
        let fixture = Fixture::open("/file-field/input", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, CONTROL).await.unwrap();
        drop_files(page, CONTROL, files_on_disk()).await;
        wait::for_selector(page, REMOVE_BETA).await.unwrap();
        page.evaluate(format!("document.querySelector({CONTROL:?}).focus()"))
            .await
            .unwrap();

        let probe =
            format!("document.querySelector({CONTROL:?}).getAttribute('aria-activedescendant')");
        keyboard::assert_chords_ignored(
            page,
            &[keyboard::ARROW_LEFT, keyboard::ARROW_RIGHT],
            &probe,
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(page, &format!("!!{probe}"), "ArrowLeft to reach a chip")
            .await
            .unwrap();

        fixture.console.assert_clean("file chip chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The cards alone could come from `value`; the hidden input's `FileList` is
/// what a form posts, so it must hold the same two files.
#[test]
fn a_dropped_file_becomes_a_card_and_posts() {
    block_on(async {
        let fixture = open_with_two_files().await;
        let page = &fixture.page;

        wait::for_selector(page, REMOVE_ALPHA).await.unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelector('input[type=file]').files].map(f => f.name).join() \
             === 'alpha.txt,beta.txt'",
            "the hidden input to hold both dropped files",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("dropping two files").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Removing a card destroys the button focus was on. Focus moves to the new
/// last card, and to the surface once none is left. The last card goes first:
/// cards are keyed by index, so removing the first re-labels the node focus
/// is on and would pass with no repair at all.
#[test]
fn removing_a_file_moves_focus_to_what_took_its_place() {
    block_on(async {
        let fixture = open_with_two_files().await;
        let page = &fixture.page;

        keyboard::tab_to(page, REMOVE_BETA, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!document.querySelector({REMOVE_BETA:?})"),
            "the beta card to go",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, REMOVE_ALPHA, "removing beta")
            .await
            .unwrap();

        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!document.querySelector({REMOVE_ALPHA:?})"),
            "the alpha card to go",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, SURFACE, "removing the last file")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("removing both files by keyboard")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 520: with no placeholder and no files the control was 0px tall, so a
/// drop at its centre landed on the frame and went nowhere.
#[test]
fn an_empty_control_fills_its_frame_and_takes_a_drop() {
    block_on(async {
        let fixture = Fixture::open("/file-field/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#bare").await.unwrap();

        let (control, frame): (f64, f64) = page
            .evaluate(
                "(() => { const c = document.querySelector('#bare'); \
                 const f = c.closest('[data-frame]'); const s = getComputedStyle(f); \
                 const inner = f.clientHeight - parseFloat(s.paddingTop) - parseFloat(s.paddingBottom); \
                 return [c.getBoundingClientRect().height, inner]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            control > 0.0 && (control - frame).abs() < 0.5,
            "the control is {control}px tall in a {frame}px frame"
        );

        // One file: a single-file field warns when it drops the second.
        drop_files(page, "#bare", files_on_disk()[..1].to_vec()).await;
        wait::for_js_true(
            page,
            "document.querySelector('#bare').textContent.includes('alpha.txt')",
            "the file dropped at the control's centre to land",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("dropping on a bare control")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
