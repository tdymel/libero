//! Keyboard operation.
//!
//! **WCAG 2.1.1 Keyboard**: all functionality operable through a keyboard
//! interface. The archetypes say what each pattern's keys mean; this module is
//! only the dispatch.
//!
//! Real key events through CDP's `Input` domain rather than synthetic DOM
//! events, so what the component receives is what a user's keypress produces,
//! including the modifier state and the default actions the browser applies.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventParamsBuilder, DispatchKeyEventType,
};

/// A key, with the virtual key code Chromium needs to apply default actions
/// (Tab moving focus, Enter activating) rather than merely reporting the press.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub key: &'static str,
    pub code: &'static str,
    pub vk: i64,
    /// The text the key produces, where it produces any.
    ///
    /// This is not cosmetic. Chromium applies a key's **default action** only
    /// for a `keyDown` carrying its text; a bare `rawKeyDown` reports the press
    /// and does nothing else. So Enter dispatched as `rawKeyDown` does not
    /// activate a button - the event fires, every listener runs, and the dialog
    /// never opens. That cost a whole `Modal` unit looking like a broken
    /// component.
    pub text: Option<&'static str>,
}

pub const TAB: Key = Key {
    key: "Tab",
    code: "Tab",
    vk: 9,
    text: None,
};
pub const SPACE: Key = Key {
    key: " ",
    code: "Space",
    vk: 32,
    text: Some(" "),
};
pub const ENTER: Key = Key {
    key: "Enter",
    code: "Enter",
    vk: 13,
    text: Some("\r"),
};
pub const ESCAPE: Key = Key {
    key: "Escape",
    code: "Escape",
    vk: 27,
    text: None,
};
pub const END: Key = Key {
    key: "End",
    code: "End",
    vk: 35,
    text: None,
};
pub const HOME: Key = Key {
    key: "Home",
    code: "Home",
    vk: 36,
    text: None,
};
pub const ARROW_LEFT: Key = Key {
    key: "ArrowLeft",
    code: "ArrowLeft",
    vk: 37,
    text: None,
};
pub const ARROW_UP: Key = Key {
    key: "ArrowUp",
    code: "ArrowUp",
    vk: 38,
    text: None,
};
pub const ARROW_RIGHT: Key = Key {
    key: "ArrowRight",
    code: "ArrowRight",
    vk: 39,
    text: None,
};
pub const ARROW_DOWN: Key = Key {
    key: "ArrowDown",
    code: "ArrowDown",
    vk: 40,
    text: None,
};

const SHIFT: i64 = 8;

/// Press and release a key.
pub async fn press(page: &Page, key: Key) -> Result<()> {
    dispatch(page, key, 0).await
}

/// Press a key with Shift held, for Shift+Tab.
pub async fn press_shift(page: &Page, key: Key) -> Result<()> {
    dispatch(page, key, SHIFT).await
}

async fn dispatch(page: &Page, key: Key, modifiers: i64) -> Result<()> {
    let base = |kind: DispatchKeyEventType| -> DispatchKeyEventParamsBuilder {
        let mut b = DispatchKeyEventParams::builder()
            .r#type(kind)
            .key(key.key)
            .code(key.code)
            .windows_virtual_key_code(key.vk)
            .native_virtual_key_code(key.vk);
        if modifiers != 0 {
            b = b.modifiers(modifiers);
        }
        if let Some(text) = key.text {
            b = b.text(text);
        }
        b
    };

    // `keyDown` for a key that carries text, `rawKeyDown` otherwise. See
    // `Key::text`.
    let down = if key.text.is_some() {
        DispatchKeyEventType::KeyDown
    } else {
        DispatchKeyEventType::RawKeyDown
    };
    page.execute(base(down).build().map_err(anyhow::Error::msg)?)
        .await?;
    page.execute(
        base(DispatchKeyEventType::KeyUp)
            .build()
            .map_err(anyhow::Error::msg)?,
    )
    .await?;
    Ok(())
}

/// Type text one character at a time, as a user would.
pub async fn type_text(page: &Page, text: &str) -> Result<()> {
    for ch in text.chars() {
        page.execute(
            DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::Char)
                .text(ch.to_string())
                .build()
                .map_err(anyhow::Error::msg)?,
        )
        .await?;
    }
    Ok(())
}

/// Tab forward until `selector` holds focus, or fail.
///
/// This is how the suite proves a control is *reachable*, which is a different
/// claim from it being focusable: a control with `tabindex="-1"` focuses
/// perfectly well by script and can never be reached by a keyboard user.
pub async fn tab_to(page: &Page, selector: &str, max: usize) -> Result<usize> {
    // Wait for it to exist before walking the keyboard at it. Without this, a
    // control that has not mounted yet reports as "not reachable within N tab
    // stops", which sends the reader looking for a tabindex bug that is not
    // there.
    crate::wait::for_selector(page, selector).await?;
    let sel = serde_json::to_string(selector)?;
    for pressed in 1..=max {
        press(page, TAB).await?;
        let arrived: bool = page
            .evaluate(format!(
                "document.activeElement === document.querySelector({sel})"
            ))
            .await?
            .into_value()?;
        if arrived {
            return Ok(pressed);
        }
    }
    let actual = super::focus::active_element(page).await?;
    bail!("{selector} was not reachable within {max} tab stops; focus ended on {actual:?}");
}
