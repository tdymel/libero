//! Keyboard dispatch (WCAG 2.1.1) through CDP's `Input` domain, so the browser applies
//! modifiers and default actions as for a real keypress. Key meanings live in `archetypes`.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventParamsBuilder, DispatchKeyEventType, InsertTextParams,
};

/// A key, with the virtual key code Chromium needs to apply default actions
/// (Tab moving focus, Enter activating) rather than merely reporting the press.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub key: &'static str,
    pub code: &'static str,
    pub vk: i64,
    /// The text the key produces. Needed: Chromium runs a default action (Enter activating)
    /// only for a `keyDown` carrying text, not a bare `rawKeyDown`.
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
pub const PAGE_UP: Key = Key {
    key: "PageUp",
    code: "PageUp",
    vk: 33,
    text: None,
};
pub const PAGE_DOWN: Key = Key {
    key: "PageDown",
    code: "PageDown",
    vk: 34,
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
pub const F10: Key = Key {
    key: "F10",
    code: "F10",
    vk: 121,
    text: None,
};
pub const BACKSPACE: Key = Key {
    key: "Backspace",
    code: "Backspace",
    vk: 8,
    text: None,
};

/// CDP modifier bits, for [`press_with`].
pub const ALT: i64 = 1;
pub const CTRL: i64 = 2;
pub const META: i64 = 4;
pub const SHIFT: i64 = 8;

/// Press and release a key.
pub async fn press(page: &Page, key: Key) -> Result<()> {
    dispatch(page, key, 0).await
}

/// Press a key with Shift held, for Shift+Tab.
pub async fn press_shift(page: &Page, key: Key) -> Result<()> {
    dispatch(page, key, SHIFT).await
}

/// Press a key with the given modifier bits held, e.g. `CTRL` for Ctrl+K.
/// Give a chord's key no `text`: a held Ctrl or Meta types nothing.
pub async fn press_with(page: &Page, key: Key, modifiers: i64) -> Result<()> {
    dispatch(page, key, modifiers).await
}

/// Ctrl+PageUp/PageDown/Tab switch the browser's tab unseen; a later Alt+ArrowLeft
/// then sends another test's page Back to `about:blank` (todo 597).
fn switches_tabs(key: Key, modifiers: i64) -> bool {
    modifiers & CTRL != 0 && matches!(key.key, "PageUp" | "PageDown" | "Tab")
}

async fn dispatch(page: &Page, key: Key, modifiers: i64) -> Result<()> {
    if switches_tabs(key, modifiers) {
        bail!(
            "Ctrl+{} switches the browser's tab and never reaches the page; \
             sending it breaks other tests (todo 597)",
            key.key
        );
    }
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

    // `keyDown` for a key that carries text, `rawKeyDown` otherwise (see `Key::text`).
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

/// Type text one character at a time, as a user would: a `keydown` (which inserts the
/// text) and a `keyup` per character, so keydown handlers such as typeahead see it (todo 1405).
pub async fn type_text(page: &Page, text: &str) -> Result<()> {
    for ch in text.chars() {
        let key = ch.to_string();
        let code = match ch {
            'a'..='z' | 'A'..='Z' => Some((
                format!("Key{}", ch.to_ascii_uppercase()),
                ch.to_ascii_uppercase() as i64,
            )),
            '0'..='9' => Some((format!("Digit{ch}"), ch as i64)),
            _ => None,
        };
        let event = |kind: DispatchKeyEventType| {
            let down = kind == DispatchKeyEventType::KeyDown;
            let mut b = DispatchKeyEventParams::builder().r#type(kind).key(&key);
            if let Some((code, vk)) = &code {
                b = b
                    .code(code)
                    .windows_virtual_key_code(*vk)
                    .native_virtual_key_code(*vk);
            }
            if down {
                b = b.text(&key);
            }
            b.build().map_err(anyhow::Error::msg)
        };
        page.execute(event(DispatchKeyEventType::KeyDown)?).await?;
        page.execute(event(DispatchKeyEventType::KeyUp)?).await?;
    }
    Ok(())
}

/// Text committed with no key event, as a soft keyboard's IME sends it.
pub async fn insert_text(page: &Page, text: &str) -> Result<()> {
    page.execute(InsertTextParams::new(text)).await?;
    Ok(())
}

/// Ctrl, Alt and Meta with each of `keys` leave `probe` (a JS expression) unchanged and are
/// not cancelled: the chord is the browser's, as for a native control.
pub async fn assert_chords_ignored(page: &Page, keys: &[Key], probe: &str) -> Result<()> {
    assert_chords_ignored_with(
        page,
        &[("Alt", ALT), ("Ctrl", CTRL), ("Meta", META)],
        keys,
        probe,
    )
    .await
}

/// [`assert_chords_ignored`] for the given `(name, bits)` modifiers only, for a
/// combobox where Alt+ArrowDown and Alt+ArrowUp open and close the list (APG).
pub async fn assert_chords_ignored_with(
    page: &Page,
    modifiers: &[(&str, i64)],
    keys: &[Key],
    probe: &str,
) -> Result<()> {
    let read = format!("JSON.stringify({probe})");
    let before: String = page.evaluate(read.as_str()).await?.into_value()?;
    page.evaluate(
        "window.__chordCancelled = []; if (!window.__chordListener) { \
         window.__chordListener = true; window.addEventListener('keydown', e => { \
         if ((e.ctrlKey || e.altKey || e.metaKey) && e.defaultPrevented) \
         window.__chordCancelled.push(e.key); }); } 1",
    )
    .await?;
    for &(name, modifier) in modifiers {
        for key in keys {
            // Honoured, these two navigate away (Back, home page).
            if modifier & ALT != 0 && matches!(key.key, "ArrowLeft" | "Home") {
                continue;
            }
            // The page never sees these; see [`switches_tabs`].
            if switches_tabs(*key, modifier) {
                continue;
            }
            press_with(page, *key, modifier).await?;
            page.evaluate("new Promise(r => setTimeout(() => r(1), 60))")
                .await?;
            let after: String = page.evaluate(read.as_str()).await?.into_value()?;
            if after != before {
                bail!(
                    "{name}+{} changed {probe} from {before} to {after}",
                    key.key
                );
            }
            let cancelled: Vec<String> = page
                .evaluate("window.__chordCancelled")
                .await?
                .into_value()?;
            if !cancelled.is_empty() {
                bail!(
                    "{name}+{} was cancelled; the chord belongs to the browser",
                    key.key
                );
            }
        }
    }
    Ok(())
}

/// Tab forward until `selector` holds focus, or fail: proves reachable, not merely focusable.
pub async fn tab_to(page: &Page, selector: &str, max: usize) -> Result<usize> {
    // Else an unmounted control reads as a tabindex bug.
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
