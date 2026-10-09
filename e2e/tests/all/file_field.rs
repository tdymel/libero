//! `FileField`: a real CDP drop, a chooser pick (1061), focus after its file is removed (406), and the group,
//! Browse button and chip list (529, 530).

use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchDragEventParams, DispatchDragEventType, DragData, DragDataItem,
};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_text};
use e2e::passes::keyboard::Key;
use e2e::passes::{contrast, focus, keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, ax, wait};

/// The dropzone's Browse button: the only one the fixture draws.
const BROWSE: &str = "[data-fixture-ready] [data-slot=browse]";
/// The dropzone surface, the group around it.
const SURFACE: &str = "[data-fixture-ready] [role=group]";
const REMOVE_ALPHA: &str = "[aria-label=\"Remove alpha.txt\"]";
const REMOVE_BETA: &str = "[aria-label=\"Remove beta.txt\"]";
/// The `Input` variant's Browse button, which carries the field's id.
const CONTROL: &str = "#attachments";
/// The `Input` variant's frames.
const FRAME: &str = "[data-fixture-ready] [data-frame]";

/// What the field's live region says (todo 1822).
const STATUS: &str = "document.querySelector('[data-fixture-ready] [role=status]').textContent";

const DELETE: Key = Key {
    key: "Delete",
    code: "Delete",
    vk: 46,
    text: None,
};

/// Todo 483: the label focuses the Browse button it names by id.
#[test]
fn a_click_on_the_label_focuses_the_browse_button() {
    crate::select::label_click_focuses("/file-field", BROWSE);
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("file_field", "/file-field")
        .focusable(BROWSE)
        .targets(BROWSE)
        .run();
}

/// Todo 529: the `Input` variant's group and Browse button, empty.
#[test]
fn the_input_variant_meets_the_baseline() {
    Suite::new("file_field_input", "/file-field/input")
        .focusable(CONTROL)
        // The frame is the target: a press on its padding is a press on Browse.
        .targets(FRAME)
        .run();
}

/// Todo 529: axe found `aria-required` on a `role=button` here; the required
/// field with an error is now clean, and its tree is the baseline.
#[test]
fn the_required_and_bare_fields_meet_the_baseline() {
    Suite::new("file_field_states", "/file-field/states")
        .focusable("#contract")
        .targets(FRAME)
        .run();
}

/// Todo 1061: Browse opens the chooser, and the picked file's bytes reach Rust.
/// Android's WebView opened none: dioxus hands its inputs' clicks to the host.
async fn picks_and_reads<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.choose_files("#receipt", &[("note.txt", "picked on the device")])
        .await?;
    eventually_text(d, "#read", "picked on the device", "a pick").await?;
    eventually(d, "the picked file's name in the field", async |d| {
        Ok(d.text(FRAME).await?.contains("note.txt"))
    })
    .await
}

e2e::scenario!(
    browse_picks_a_file_and_rust_reads_it,
    "/file-field/pick",
    picks_and_reads,
    native: skip("rfd opens the desktop portal's dialog, which no driver answers"),
    desktop: skip("1126: the GTK file chooser is not driven")
);

/// Todo 2217: Android starts a pick on its event thread and ends it on the render thread;
/// the one-dialog latch must clear across both, so a second Browse opens again.
async fn picks_twice<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    picks_and_reads(d, route).await?;
    d.choose_files("#receipt", &[("again.txt", "picked a second time")])
        .await?;
    eventually_text(d, "#read", "picked a second time", "a second pick").await
}

e2e::scenario!(
    browse_picks_a_second_time,
    "/file-field/pick",
    picks_twice,
    native: skip("rfd opens the desktop portal's dialog, which no driver answers"),
    desktop: skip("1126: the GTK file chooser is not driven")
);

/// WCAG 1.4.11: the dashed border is all that shows the dropzone's extent
/// (todo 490).
#[test]
fn the_dropzone_border_parts_at_3_to_1() {
    crate::boundary::assert_boundaries(
        "/file-field",
        &format!(
            "const surface = document.querySelector({SURFACE:?});
             return [['dropzone border on the page', RATIO(CSS(surface, 'borderTopColor'), PAGE(surface))]];"
        ),
    );
}

/// Real files on disk, so the drop carries a `FileList` the page can read. A directory per
/// test, removed with it: parallel tests rewrote one shared pair (todo 1830).
struct OnDisk(std::path::PathBuf, &'static [&'static str]);

impl OnDisk {
    fn new() -> Self {
        Self::with(&["alpha.txt", "beta.txt"])
    }

    fn with(names: &'static [&'static str]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("e2e-file-field-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in names {
            std::fs::write(dir.join(name), format!("{name} contents")).unwrap();
        }
        Self(dir, names)
    }

    fn paths(&self) -> Vec<String> {
        self.1
            .iter()
            .map(|name| self.0.join(name).to_string_lossy().into_owned())
            .collect()
    }
}

impl Drop for OnDisk {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Enter, over, drop: what the browser sends for an OS file dragged onto the
/// surface. Trusted events, unlike a `DataTransfer` built in the page.
pub(crate) async fn drop_files(page: &Page, selector: &str, files: Vec<String>) {
    let at = pointer::centre_of(page, selector).await.unwrap();
    drop_files_at(page, at.x, at.y, files).await;
}

async fn drop_files_at(page: &Page, x: f64, y: f64, files: Vec<String>) {
    drag_files_at(page, x, y, files, true).await;
}

/// The drag of [`drop_files_at`], released over the target only if `release`.
async fn drag_files_at(page: &Page, x: f64, y: f64, files: Vec<String>, release: bool) {
    let mut kinds = vec![
        DispatchDragEventType::DragEnter,
        DispatchDragEventType::DragOver,
    ];
    if release {
        kinds.push(DispatchDragEventType::Drop);
    }
    for kind in kinds {
        // CDP requires `items`, and chromiumoxide drops an empty list.
        let data = DragData::builder()
            .item(DragDataItem::new("text/plain", "files"))
            .files(files.clone())
            .drag_operations_mask(1)
            .build()
            .unwrap();
        let params = DispatchDragEventParams::builder()
            .r#type(kind.clone())
            .x(x)
            .y(y)
            .data(data)
            .build()
            .unwrap();
        page.execute(params)
            .await
            .unwrap_or_else(|e| panic!("dispatching {kind:?}: {e}"));
    }
}

async fn open_with_two_files() -> (Fixture, OnDisk) {
    let fixture = Fixture::open("/file-field", Viewport::Desktop)
        .await
        .unwrap();
    wait::for_visible(&fixture.page, SURFACE).await.unwrap();
    let disk = OnDisk::new();
    drop_files(&fixture.page, SURFACE, disk.paths()).await;
    wait::for_selector(&fixture.page, REMOVE_BETA)
        .await
        .unwrap();
    (fixture, disk)
}

/// Counts the picker's openings, and keeps the dialog shut: the component
/// opens it by clicking the hidden input.
async fn count_openings(page: &Page) {
    page.evaluate(
        "window.__opened = 0; document.addEventListener('click', e => { \
         if (e.target.matches('input[type=file]')) { window.__opened++; e.preventDefault(); } \
         }, true)",
    )
    .await
    .unwrap();
}

async fn openings(page: &Page) -> u32 {
    page.evaluate("window.__opened")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn active_id(page: &Page) -> String {
    page.evaluate("document.activeElement && document.activeElement.id")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Todo 509: Ctrl/Alt/Meta+ArrowLeft/Right leave the focus where it is; the
/// plain arrow moves it from the Browse button to the last chip.
#[test]
fn modifier_chords_leave_the_chips_alone() {
    block_on(async {
        let fixture = Fixture::open("/file-field/input", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, CONTROL).await.unwrap();
        let disk = OnDisk::new();
        drop_files(page, CONTROL, disk.paths()).await;
        wait::for_selector(page, REMOVE_BETA).await.unwrap();
        page.evaluate(format!("document.querySelector({CONTROL:?}).focus()"))
            .await
            .unwrap();

        let probe = "document.activeElement.id";
        keyboard::assert_chords_ignored(
            page,
            &[keyboard::ARROW_LEFT, keyboard::ARROW_RIGHT],
            probe,
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        focus::wait_for_focus(page, "#attachments-file-1", "ArrowLeft from Browse")
            .await
            .unwrap();

        fixture.console.assert_clean("file chip chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 529, 530: with a chip focused the required field is axe clean, a named group of
/// the file list and Browse, which hears "Required" and the error.
#[test]
fn the_chips_take_the_focus_and_the_required_field_stays_clean() {
    block_on(async {
        let fixture = Fixture::open("/file-field/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#contract").await.unwrap();
        let disk = OnDisk::new();
        drop_files(page, "#contract", disk.paths()).await;
        wait::for_selector(page, "#contract-file-1").await.unwrap();

        keyboard::tab_to(page, "#contract-file-0", 10)
            .await
            .unwrap();
        contrast::assert_clean(page, "[data-fixture-ready]")
            .await
            .unwrap();
        let group = "[role=group][aria-labelledby=contract-label]";
        let tree = ax::snapshot(page, group).await.unwrap();
        for line in [
            "group \"Contract\"",
            "list",
            "listitem",
            "StaticText \"alpha.txt\"",
            "button \"Contract Browse files\" [invalid]",
        ] {
            assert!(tree.contains(line), "no {line:?} in\n{tree}");
        }
        assert_eq!(
            ax::description(page, "#contract").await.unwrap(),
            "Required We cannot read that file."
        );

        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        focus::wait_for_focus(page, "#contract-file-1", "ArrowRight")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        focus::wait_for_focus(page, "#contract", "ArrowRight past the last chip")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        focus::wait_for_focus(page, "#contract-file-1", "ArrowLeft from Browse")
            .await
            .unwrap();

        // The last chip goes: focus to the one left, then to Browse.
        keyboard::press(page, DELETE).await.unwrap();
        focus::wait_for_focus(page, "#contract-file-0", "removing beta")
            .await
            .unwrap();
        keyboard::press(page, DELETE).await.unwrap();
        focus::wait_for_focus(page, "#contract", "removing the last file")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#contract-file-0')",
            "both chips to go",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("walking the chips").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 2581, 2582: a read-only single-file dropzone holding a file draws no x, so its card
/// list is the tab stop; with the error on it, axe clean before and after.
#[test]
fn a_read_only_single_file_dropzone_stays_reachable() {
    block_on(async {
        let fixture = Fixture::open("/file-field/single", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        const LIST: &str = "[data-fixture-ready] ul[role=list]";
        wait::for_visible(page, SURFACE).await.unwrap();
        let disk = OnDisk::with(&["alpha.txt"]);
        drop_files(page, SURFACE, disk.paths()).await;
        wait::for_selector(page, LIST).await.unwrap();
        contrast::assert_clean(page, "[data-fixture-ready]")
            .await
            .unwrap();

        pointer::click(page, "#to-readonly").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector({LIST:?}).getAttribute('tabindex') === '0' \
                 && !document.querySelector('[aria-label=\"Remove alpha.txt\"]')"
            ),
            "the card list to become the tab stop",
        )
        .await
        .unwrap();
        keyboard::tab_to(page, LIST, 3).await.unwrap();
        contrast::assert_clean(page, "[data-fixture-ready]")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("a read-only dropzone")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Enter and Space on Browse open the picker; read-only keeps the tab order and refuses
/// every edit; disabled leaves the tab order (529).
#[test]
fn read_only_and_disabled_refuse_the_picker_and_the_remove() {
    block_on(async {
        let fixture = Fixture::open("/file-field/modes", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#modes").await.unwrap();
        count_openings(page).await;

        keyboard::tab_to(page, "#modes", 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, "window.__opened === 2", "Enter and Space to open")
            .await
            .unwrap();

        let disk = OnDisk::new();
        drop_files(page, "#modes", disk.paths()).await;
        wait::for_selector(page, "#modes-file-1").await.unwrap();
        pointer::click(page, "#to-readonly").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#modes').getAttribute('aria-disabled') === 'true'",
            "the field to turn read-only",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, "#modes-file-0", 10).await.unwrap();
        keyboard::press(page, DELETE).await.unwrap();
        keyboard::tab_to(page, "#modes", 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        pointer::click(page, "#modes").await.unwrap();
        // Nothing to wait for: give a refused edit a frame to show.
        crate::settle::painted(page).await.unwrap();
        assert_eq!(openings(page).await, 2, "read-only opened the picker");
        assert!(
            page.evaluate("!!document.querySelector('#modes-file-1')")
                .await
                .unwrap()
                .into_value::<bool>()
                .unwrap(),
            "read-only removed a file"
        );

        pointer::click(page, "#to-disabled").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#modes').disabled \
             && !document.querySelector('#modes-file-0').hasAttribute('tabindex')",
            "the field to turn disabled",
        )
        .await
        .unwrap();
        pointer::click(page, "[role=group][aria-labelledby=modes-label]")
            .await
            .unwrap();
        crate::settle::painted(page).await.unwrap();
        assert_eq!(openings(page).await, 2, "disabled opened the picker");
        assert_ne!(active_id(page).await, "modes");

        fixture
            .console
            .assert_clean("read-only and disabled")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The cards alone could come from `value`; the hidden input's `FileList` is
/// what a form posts, so it must hold the same two files. The drop is announced.
#[test]
fn a_dropped_file_becomes_a_card_and_posts() {
    block_on(async {
        let (fixture, _disk) = open_with_two_files().await;
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
        wait::for_js_true(
            page,
            &format!("{STATUS}.includes('alpha.txt') && {STATUS}.includes('beta.txt')"),
            "the live region to name both added files",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("dropping two files").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Removing the focused card moves focus to the new last card, then Browse. The last goes
/// first: cards are keyed by index, so removing the first would pass unrepaired. The removal
/// is announced.
#[test]
fn removing_a_file_moves_focus_to_what_took_its_place() {
    block_on(async {
        let (fixture, _disk) = open_with_two_files().await;
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
        wait::for_js_true(
            page,
            &format!("{STATUS}.includes('beta.txt') && !{STATUS}.includes('alpha.txt')"),
            "the live region to name the removed file alone",
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
        focus::wait_for_focus(page, BROWSE, "removing the last file")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("removing both files by keyboard")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 520: with no placeholder and no files the control was 0px tall, so a centred drop
/// went nowhere. The Browse button is that control now.
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

        // One file: a single-file field refuses the second.
        let disk = OnDisk::new();
        drop_files(page, "#bare", disk.paths()[..1].to_vec()).await;
        wait::for_js_true(
            page,
            "document.querySelector('#bare').closest('[role=group]').textContent.includes('alpha.txt')",
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

/// A file dropped on the frame's padding is the field's, not the browser's to
/// open (todo 531).
#[test]
fn a_drop_on_the_frame_padding_is_taken() {
    block_on(async {
        let fixture = Fixture::open("/file-field/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#bare").await.unwrap();

        let (x, y, on_frame): (f64, f64, bool) = page
            .evaluate(
                "(() => { const f = document.querySelector('#bare').closest('[data-frame]'); \
                 const r = f.getBoundingClientRect(); const x = r.left + 3, y = r.top + r.height / 2; \
                 return [x, y, document.elementFromPoint(x, y) === f]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(on_frame, "({x}, {y}) is not on the frame's own padding");

        let disk = OnDisk::new();
        drop_files_at(page, x, y, disk.paths()[..1].to_vec()).await;
        wait::for_js_true(
            page,
            "document.querySelector('#bare').closest('[role=group]').textContent.includes('alpha.txt')",
            "the file dropped on the padding to land",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("dropping on the frame padding")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2553: a file dragged over the `Input` variant's frame, padding included, tints it
/// and draws a primary border, as the dropzone does; the drop clears it.
#[test]
fn a_drag_over_the_input_frame_draws_the_dragging_look() {
    block_on(async {
        let fixture = Fixture::open("/file-field/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#bare").await.unwrap();

        let frame = "document.querySelector('#bare').closest('[data-frame]')";
        let look = format!(
            "(() => {{ const f = {frame}; const s = getComputedStyle(f); \
             return [f.dataset.state.split(' ').includes('dragging'), s.borderTopColor, s.backgroundImage]; }})()"
        );
        let at_rest: (bool, String, String) = page
            .evaluate(look.clone())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!at_rest.0, "dragging before any drag");

        let (x, y): (f64, f64) = page
            .evaluate(format!(
                "(() => {{ const r = {frame}.getBoundingClientRect(); return [r.left + 3, r.top + r.height / 2]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let disk = OnDisk::new();
        drag_files_at(page, x, y, disk.paths()[..1].to_vec(), false).await;
        wait::for_js_true(
            page,
            &format!("{frame}.dataset.state.split(' ').includes('dragging')"),
            "the frame to show the drag",
        )
        .await
        .unwrap();
        let dragged: (bool, String, String) = page
            .evaluate(look.clone())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(dragged.1, at_rest.1, "the border keeps its resting colour");
        assert_ne!(dragged.2, "none", "the frame draws no tint");

        drop_files_at(page, x, y, disk.paths()[..1].to_vec()).await;
        wait::for_js_true(
            page,
            &format!("!{frame}.dataset.state.split(' ').includes('dragging')"),
            "the drop to end the dragging look",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("dragging over the frame")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2551: a native reset empties the input, but a field holding its own value
/// still shows the file, so the input must post it again; in a raw `<form>` and
/// in a `Form`.
#[test]
fn a_native_reset_leaves_the_input_holding_the_shown_file() {
    block_on(async {
        let fixture = Fixture::open("/file-field/reset", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#held").await.unwrap();
        let disk = OnDisk::new();
        for id in ["raw", "held"] {
            let held = format!(
                "[...document.getElementById('{id}').parentElement \
                 .querySelector('input[type=file]').files].map(f => f.name).join() === 'alpha.txt'"
            );
            drop_files(page, &format!("#{id}"), disk.paths()[..1].to_vec()).await;
            wait::for_js_true(page, &held, "the input to hold the dropped file")
                .await
                .unwrap();
            pointer::click(page, &format!("#{id}-reset")).await.unwrap();
            wait::for_js_true(
                page,
                &format!("{held} && !!document.getElementById('{id}-file-0')"),
                "the input to hold the file again after the reset",
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("native resets").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1819: a drop the field refuses is said, not dropped silently: a PDF on an
/// `image/*` field, then a second file on a single-file one.
#[test]
fn a_refused_drop_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/file-field/reject", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#photo").await.unwrap();

        let pdf = OnDisk::with(&["note.pdf"]);
        drop_files(page, "#photo", pdf.paths()).await;
        wait::for_js_true(
            page,
            &format!(
                "{STATUS}.includes('note.pdf') && {STATUS}.includes('not an accepted file type')"
            ),
            "the live region to name the refused PDF",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('rejected').textContent === 'note.pdf:Type'",
            "onreject to get the PDF",
        )
        .await
        .unwrap();
        assert!(
            !page
                .evaluate("document.querySelector('#photo').closest('[role=group]').textContent.includes('note.pdf')")
                .await
                .unwrap()
                .into_value::<bool>()
                .unwrap(),
            "the refused PDF became a chip"
        );

        let two = OnDisk::with(&["one.png", "two.png"]);
        drop_files(page, "#photo", two.paths()).await;
        wait::for_js_true(
            page,
            &format!("{STATUS}.includes('one.png') && {STATUS}.includes('two.png') && {STATUS}.includes('takes one file')"),
            "the live region to name the kept and the refused file",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('rejected').textContent === 'two.png:TooMany'",
            "onreject to get the second file",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("refused drops").unwrap();
        fixture.close().await.unwrap();
    });
}
