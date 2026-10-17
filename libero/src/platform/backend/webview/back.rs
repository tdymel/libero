use dioxus::core::spawn_forever;
use serde_json::json;

use super::{eval, eval_with, runs_scripts};

/// [`back_entries`] in the page. Chromium honours one pushed entry per user activation and
/// skips on Back the one before a second (2240): one push per tap or key, the rest owed.
/// A tap, Enter or Space with nothing owed is reported, for Rust to bank an entry (3322).
const BACK_ENTRIES: &str =
    "const back = window.__lsxBack = { held: 0, want: 0, ignore: 0, spent: false };
    back.settle = (push) => {
        if (back.want < back.held) {
            back.ignore++;
            history.go(back.want - back.held);
            back.held = back.want;
        } else if (back.want > back.held && push && !back.spent
            && navigator.userActivation?.isActive !== false) {
            history.pushState({ lsxBack: true }, '');
            back.held++;
            back.spent = true;
        }
    };
    addEventListener('popstate', () => {
        if (back.ignore > 0) return void back.ignore--;
        back.held = Math.max(0, back.held - 1);
        dioxus.send(true);
        back.settle(false);
    });
    const activated = (tap) => {
        back.spent = false;
        if (back.want > back.held) back.settle(true);
        else if (tap && back.want > 0) dioxus.send(false);
    };
    addEventListener('pointerup', () => activated(true), true);
    addEventListener('keydown', (event) => {
        if (event.key !== 'Escape') activated(!event.repeat && (event.key === 'Enter' || event.key === ' '));
    }, true);";

/// Back pops one of [`back_entries`]' history entries: `onback(true)` runs, unless the
/// page took it back itself; a tap while some are held, `onback(false)`.
pub(crate) fn watch_back(onback: impl Fn(bool) + 'static) -> bool {
    if !runs_scripts() {
        return false;
    }
    let script = eval_with(
        json!(null),
        &format!("{BACK_ENTRIES}\nawait new Promise(() => {{}});"),
    );
    spawn_forever(async move {
        let mut script = script;
        while let Ok(back) = script.recv::<bool>().await {
            onback(back);
        }
    });
    true
}

/// Asks the page to hold `count` entries: pushed inside a tap's activation (now
/// or at the next one), taken back with their `popstate` ignored.
pub(crate) fn back_entries(count: usize) {
    eval(&format!(
        "if (window.__lsxBack) {{ window.__lsxBack.want = {count}; window.__lsxBack.settle(true); }}"
    ));
}
