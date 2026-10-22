//! Picker fields whose tap opens only the picker: in a phone app (todo 1012) and under a
//! finger in a browser (3905). The picker then holds a text input of its own for typing.

use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::common::{COARSE_POINTER, focus_ring_sx},
    hooks::{use_css, use_media_query},
    platform::soft_keyboard_app,
    sx::{StaticSx, sx},
    theme::{Size, SizeCss},
};

/// A field's text box, one row of the sheet. 16px text: a phone browser zooms in on less.
static PICKER_INPUT_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .with("box-sizing", "border-box")
        .width("100%")
        .min_height("44px")
        .padding("0 12px")
        .margin("0")
        .border("1px solid var(--lsx-paper-border-color, currentColor)")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("max(16px, 1em)")
        .focus_visible(focus_ring_sx())
});

/// Whether a field's input keeps the soft keyboard down, and the picker's own text input.
#[derive(Clone, PartialEq)]
pub(crate) struct Keyboardless {
    keyboardless: bool,
    class: Option<String>,
    press: Press,
}

pub(crate) fn use_keyboardless() -> Keyboardless {
    let coarse = use_media_query(COARSE_POINTER);
    Keyboardless {
        keyboardless: soft_keyboard_app() || coarse(),
        class: use_css(Some(&PICKER_INPUT_SX), CssLayer::Framework),
        press: Press(use_hook(|| CopyValue::new(false))),
    }
}

/// A press on the picker's text box, from its `pointerdown` until the field's blur it causes.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Press(CopyValue<bool>);

impl Press {
    /// Whether the blur came from a press on the text box; answered once.
    pub(crate) fn taken(self) -> bool {
        let mut press = self.0;
        let pressed = *press.peek();
        if pressed {
            press.set(false);
        }
        pressed
    }
}

/// The picker's text input: its text, the names it takes, and its listeners.
pub(crate) struct PickerInput<I, K, B> {
    /// The field's control id; the field label, when there is one, is `{id}-label`.
    pub(crate) field_id: String,
    pub(crate) labelled: bool,
    /// The name without a field label.
    pub(crate) fallback_label: &'static str,
    pub(crate) value: String,
    pub(crate) invalid: bool,
    pub(crate) oninput: I,
    pub(crate) onkeydown: K,
    pub(crate) onblur: B,
}

impl Keyboardless {
    pub(crate) fn press(&self) -> Press {
        self.press
    }

    /// The field input's `inputmode`.
    pub(crate) fn inputmode(&self) -> Option<&'static str> {
        self.keyboardless.then_some("none")
    }

    /// The picker's own text input, while the field's keyboard is down. Its tap focuses it
    /// natively, so the keyboard rises in the tap's gesture; a WebView raises none for an
    /// `inputmode` lifted on a focused input, nor for a `focus()` sent from Rust (3902, 3905).
    pub(crate) fn input<I, K, B>(&self, input: PickerInput<I, K, B>) -> Option<Element>
    where
        I: FnMut(FormEvent) + 'static,
        K: FnMut(KeyboardEvent) + 'static,
        B: FnMut(FocusEvent) + 'static,
    {
        if !self.keyboardless {
            return None;
        }
        let PickerInput {
            field_id,
            labelled,
            fallback_label,
            value,
            invalid,
            oninput,
            onkeydown,
            onblur,
        } = input;
        let mut press = self.press.0;
        Some(rsx! {
            input {
                class: self.class.clone(),
                "data-slot": "text-input",
                r#type: "text",
                id: "{field_id}-text",
                value,
                autocomplete: "off",
                spellcheck: "false",
                "aria-labelledby": labelled.then(|| format!("{field_id}-label")),
                "aria-label": (!labelled).then_some(fallback_label),
                "aria-invalid": invalid.then_some("true"),
                // Past the picker's cancelled mousedown, so the tap focuses it natively.
                onmousedown: move |event| event.stop_propagation(),
                // Heard before the field's blur, which then keeps the picker open.
                onpointerdown: move |_| press.set(true),
                onpointercancel: move |_| press.set(false),
                oninput,
                onkeydown,
                onblur,
            }
        })
    }
}
