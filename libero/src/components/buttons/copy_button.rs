use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::ActionIcon,
        common::{Glyph, Input, Variant, base_props},
    },
    context::IconSlot,
    hooks::{use_clipboard, use_id, use_localization, use_silent_focus_out},
    sx::ThemeAwareValue,
};

base_props! {
    pub struct CopyButtonProps {
        /// The text a press writes to the clipboard.
        #[props(into)]
        value: String,
        /// Unset, no chrome of its own.
        #[props(default, into)]
        variant: Input<Variant>,
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<ThemeAwareValue>,
        #[props(default, into)]
        radius: Input<ThemeAwareValue>,
        /// Replaces the localized name, e.g. "Copy link".
        #[props(default, into)]
        aria_label: Option<String>,
        /// What a press copies, read after the name, e.g. "Add libero to your project".
        #[props(default, into)]
        label: Option<String>,
        #[props(default)]
        disabled: Option<bool>,
    }
}

/// An icon button that copies `value` to the clipboard and announces the result.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::CopyButton;
/// # fn app() -> Element {
/// rsx! { CopyButton { value: "cargo add libero", variant: "outlined" } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/buttons/copy-button>
#[component]
pub fn CopyButton(props: CopyButtonProps) -> Element {
    let mut clipboard = use_clipboard();
    let labels = use_localization().copy_button;
    // Blitz's Tab fires no `blur`: the same reset, from the silent move.
    let silent = use_silent_focus_out(move || {
        let mut clipboard = clipboard;
        clipboard.reset();
    });
    let value = props.value.clone();
    let label_id = use_id();
    let described = props.label.as_ref().map(|_| label_id());

    rsx! {
        ActionIcon {
            aria_label: props.aria_label.clone().unwrap_or_else(|| labels.copy.to_string()),
            variant: props.variant.clone(),
            color: props.color.clone(),
            size: props.size.clone(),
            radius: props.radius.clone(),
            disabled: props.disabled,
            aria_describedby: described,
            class: props.class.clone(),
            sx: props.sx.clone(),
            states: props.states.clone(),
            attributes: props.attributes.clone(),
            // Reset first, so a second copy re-announces rather than repeating unchanged text.
            onclick: move |_| {
                clipboard.reset();
                clipboard.copy(value.clone());
            },
            onmouseleave: move |_| clipboard.reset(),
            // A keyboard or touch user never leaves with a mouse.
            onblur: move |_| clipboard.reset(),
            onmounted: move |event| {
                if let Some(silent) = silent {
                    silent.mount()(event);
                }
            },
            if clipboard.copied() {
                Glyph { slot: IconSlot::Check, icon: lucide::check::outlined }
            } else if clipboard.failed() {
                Glyph { slot: IconSlot::CopyFailed, icon: lucide::circle_alert::outlined }
            } else {
                Glyph { slot: IconSlot::Copy, icon: lucide::copy::outlined }
            }
        }
        // `hidden`: a description only, not read again in browse mode.
        if let Some(label) = props.label.as_deref() {
            span { id: label_id, hidden: true, "{label}" }
        }
        // Always mounted, so a screen reader is watching it when the text arrives.
        VisuallyHidden { role: "status",
            if clipboard.copied() {
                {labels.copied}
            } else if clipboard.failed() {
                {labels.copy_failed}
            }
        }
    }
}
