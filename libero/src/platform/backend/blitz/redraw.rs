//! Blitz reports no render. Its mutator asks the shell for a redraw after each
//! change to the document, so libero wraps the document's `ShellProvider`: each
//! ask flushes [`Outlet`], which heals stale dirty bits before the restyle, and
//! arms the baked-box [`check`]. A render no press or key preceded (a timer, a
//! task after an async read) restyles and repaints too (todos 870, 872).
//!
//! [`Outlet`]: super::Outlet
//! [`check`]: super::baked

use std::{cell::Cell, path::PathBuf, rc::Rc, sync::Arc};

use blitz_dom::BaseDocument;
use blitz_traits::shell::{ClipboardError, FileDialogFilter, ShellProvider};
use cursor_icon::CursorIcon;
use dioxus::{core::Runtime, prelude::ScopeId};

use super::{DOCS, Doc, HELD, baked, flush_soon};

thread_local! {
    static QUIET: Cell<u32> = const { Cell::new(0) };
}

/// Runs `run` with the redraws it asks for unanswered: libero's own changes
/// (a flush, a baked-box rebuild), which would otherwise arm themselves again.
pub(super) fn quiet<T>(run: impl FnOnce() -> T) -> T {
    struct Loud;
    impl Drop for Loud {
        fn drop(&mut self) {
            QUIET.set(QUIET.get() - 1);
        }
    }
    QUIET.set(QUIET.get() + 1);
    let _loud = Loud;
    run()
}

/// Wraps `doc`'s shell provider, once per document.
pub(super) fn watch(state: &Doc, doc: &mut BaseDocument) {
    let Some(runtime) = Runtime::try_current() else {
        return;
    };
    if state.redraws_watched.replace(true) {
        return;
    }
    let inner = doc.shell_provider.clone();
    doc.set_shell_provider(Arc::new(Watching {
        inner,
        runtime: Rc::as_ptr(&runtime) as usize,
    }));
}

/// The document's own provider, and its runtime by address: an `Rc` is not
/// `Send`, and Blitz asks for a redraw from its network threads too.
struct Watching {
    inner: Arc<dyn ShellProvider>,
    runtime: usize,
}

/// The document changed. On another thread no runtime matches; a drag's
/// renders wait for the release, which checks.
fn changed(runtime: usize) {
    if QUIET.get() > 0 || HELD.get() {
        return;
    }
    let runtime = DOCS.with(|docs| {
        let docs = docs.try_borrow().ok()?;
        docs.iter()
            .find(|(owner, _)| owner.as_ptr() as usize == runtime)
            .and_then(|(owner, _)| owner.upgrade())
    });
    // Called after `render_immediate`, where no runtime is current.
    if let Some(runtime) = runtime {
        runtime.in_scope(ScopeId::ROOT, || {
            flush_soon();
            baked::check_soon();
        });
    }
}

// Every method forwarded, the defaulted ones too: one left out would drop the
// shell's own (cursor, IME, clipboard). Re-check this list on a Blitz bump.
impl ShellProvider for Watching {
    fn request_redraw(&self) {
        self.inner.request_redraw();
        changed(self.runtime);
    }
    fn set_cursor(&self, icon: Option<CursorIcon>) {
        self.inner.set_cursor(icon);
    }
    fn set_window_title(&self, title: String) {
        self.inner.set_window_title(title);
    }
    fn set_ime_enabled(&self, is_enabled: bool) {
        self.inner.set_ime_enabled(is_enabled);
    }
    fn set_ime_cursor_area(&self, x: f32, y: f32, width: f32, height: f32) {
        self.inner.set_ime_cursor_area(x, y, width, height);
    }
    fn get_clipboard_text(&self) -> Result<String, ClipboardError> {
        self.inner.get_clipboard_text()
    }
    fn set_clipboard_text(&self, text: String) -> Result<(), ClipboardError> {
        self.inner.set_clipboard_text(text)
    }
    fn open_file_dialog(&self, multiple: bool, filter: Option<FileDialogFilter>) -> Vec<PathBuf> {
        self.inner.open_file_dialog(multiple, filter)
    }
    fn request_window_close(&self) {
        self.inner.request_window_close();
    }
    fn set_window_minimized(&self, minimized: bool) {
        self.inner.set_window_minimized(minimized);
    }
    fn set_window_maximized(&self, maximized: bool) {
        self.inner.set_window_maximized(maximized);
    }
    fn is_window_maximized(&self) -> bool {
        self.inner.is_window_maximized()
    }
    fn set_window_decorations(&self, decorations: bool) {
        self.inner.set_window_decorations(decorations);
    }
    fn drag_window(&self) {
        self.inner.drag_window();
    }
}
