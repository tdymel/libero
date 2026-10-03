use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        common::{HtmlTag, Input, base_props, inset_focus_ring_sx},
        layout::use_box,
        overlay::Dialog,
        typography::Kbd,
    },
    hooks::{chord_keys, current_localization, use_css, use_element, use_resize_fallback},
    platform::{ElementApi, mod_is_meta},
    sx::{StaticSx, sx},
    utils::warn,
};

const SHORTCUT_LIST: &str = "lsx-shortcut-list";

// Chords in one column, what they do in the next; stacked below 20rem (Blitz drops `@container`).
static SHORTCUT_LIST_SX: StaticSx = StaticSx::new(|| {
    sx().display("grid")
        .grid_template_columns("max-content 1fr")
        .column_gap("lg")
        .row_gap("xs")
        .align_items("baseline")
        .margin("0")
        .container(SHORTCUT_LIST)
        .selector("& > dd", sx().margin("0"))
        .container_query(
            SHORTCUT_LIST,
            "(max-width: 20rem)",
            sx().selector("& > dt, & > dd", sx().grid_column("1 / -1"))
                .selector("& > dt:not(:first-child)", sx().margin_top("sm")),
        )
});

// A long keymap scrolls inside, so the dialog fits a short window or panel.
static SHORTCUT_SCROLL_SX: StaticSx = StaticSx::new(|| {
    sx().max_height("min(24rem, 60vh)")
        .overflow_y("auto")
        .focus_visible(inset_focus_ring_sx("-2px"))
});

/// A chord's keys from [`chord_keys`], one [`Kbd`] each, joined by " + ".
pub(crate) fn chord_kbd(keys: Vec<String>) -> Element {
    let last = keys.len().saturating_sub(1);
    rsx! {
        for (index, key) in keys.into_iter().enumerate() {
            Kbd { "{key}" }
            if index < last { " + " }
        }
    }
}

/// One row of a [`ShortcutHelp`]: a chord as [`Hotkey`](crate::hooks::Hotkey) takes it, and
/// what it does.
#[derive(Clone, Debug, PartialEq)]
pub struct Shortcut {
    chord: String,
    description: String,
}

impl Shortcut {
    /// `chord` is parsed as `use_hotkeys` parses it, so `"mod+b"` shows Ctrl or Cmd.
    pub fn new(chord: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            chord: chord.into(),
            description: description.into(),
        }
    }
}

base_props! {
    pub struct ShortcutHelpProps {
        /// The heading. Unset, the localization's `shortcut_help.title`.
        #[props(default, into)]
        title: Option<String>,
        shortcuts: Vec<Shortcut>,
    }
}

/// A dialog listing keyboard shortcuts, each chord in [`Kbd`]s named for the platform.
/// Open it with `use_modal`, often from a `shift+?` hotkey.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Shortcut, ShortcutHelp};
/// # use libero::hooks::{Hotkey, ModalScope, use_hotkeys, use_modal};
/// # fn app() -> Element {
/// let help = use_modal(|_: ModalScope<()>| rsx! {
///     ShortcutHelp {
///         shortcuts: vec![Shortcut::new("mod+b", "Bold"), Shortcut::new("alt+f10", "Go to the toolbar")],
///     }
/// });
/// use_hotkeys([Hotkey::new("shift+?", move || { help.open(); })]);
/// rsx! {}
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/shortcut-help>
#[component]
pub fn ShortcutHelp(props: ShortcutHelpProps) -> Element {
    let words = &current_localization().shortcut_help;
    let list_class = use_css(Some(&SHORTCUT_LIST_SX), CssLayer::Framework);
    let apple = mod_is_meta();
    let rows = props.shortcuts.iter().filter_map(|shortcut| {
        let Some(keys) = chord_keys(&shortcut.chord, apple, words) else {
            warn(&format!(
                "ShortcutHelp: `{}` is not a chord `use_hotkeys` binds, so it is left out.",
                shortcut.chord
            ));
            return None;
        };
        Some(rsx! {
            dt { {chord_kbd(keys)} }
            dd { "{shortcut.description}" }
        })
    });
    let title = props.title.unwrap_or_else(|| words.title.to_string());

    // A named tab stop only while it scrolls, so the keyboard can scroll it (todo 1615).
    let scroll_element = use_element();
    let mut overflows = use_signal(|| false);
    let measure = move || {
        if !scroll_element.is_mounted() {
            return;
        }
        let (content, size) = (scroll_element.scroll_size(), scroll_element.dimensions());
        spawn(async move {
            if let (Ok(content), Ok(size)) = (content.await, size.await) {
                let next = content.height > size.height + 1.0;
                if next != *overflows.peek() {
                    overflows.set(next);
                }
            }
        });
    };
    use_resize_fallback(scroll_element, move |_| measure());
    let scrolls = overflows();
    // `role=region` on the `dl` itself would drop its list semantics.
    let scroll_box = use_box()
        .framework_sx(&SHORTCUT_SCROLL_SX)
        .prepare()
        .element(&scroll_element)
        .event("onresize", move |_: Event<ResizeData>| measure())
        .attr("tabindex", scrolls.then_some("0"))
        .attr("role", scrolls.then_some("region"))
        .attr("aria-label", scrolls.then(|| title.clone()));

    rsx! {
        Dialog {
            title,
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
            {scroll_box.render(HtmlTag::Div, Vec::new(), rsx! {
                dl { class: list_class, {rows} }
            })}
        }
    }
}
