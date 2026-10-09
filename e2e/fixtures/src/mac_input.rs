//! The desktop driver's input on macOS (2782): pointer, wheel and key ops off `E2E_BRIDGE`, built
//! as NSEvents and handed to the WKWebView in this process. No TCC grant, cursor or screen needed.

use objc2::rc::Retained;
use objc2::{MainThreadMarker, Message};
use objc2_app_kit::{
    NSApplication, NSEvent, NSEventModifierFlags, NSEventType, NSScreen, NSView, NSWindow,
};
use objc2_core_graphics::{CGEvent, CGScrollEventUnit};
use objc2_foundation::{NSPoint, NSProcessInfo, NSString};
use serde_json::{Value, json};

/// Runs one op on the main thread; `view` is the WKWebView. Its JSON answer, `null` for input.
pub fn run(view: &NSView, op: &Value) -> Result<Value, String> {
    let Some(mtm) = MainThreadMarker::new() else {
        return Err("the bridge ran off the main thread".into());
    };
    let window = view.window().ok_or("the WebView has no window")?;
    let number = |name: &str| op[name].as_f64().ok_or(format!("{op}: no number `{name}`"));
    match op["op"].as_str().unwrap_or_default() {
        "prepare" => Ok(prepare(mtm, view, &window)),
        "move" => {
            let at = point(view, number("x")?, number("y")?);
            let kind = if op["held"].as_bool() == Some(true) {
                NSEventType::LeftMouseDragged
            } else {
                NSEventType::MouseMoved
            };
            mouse(view, &window, kind, at, 0)?;
            Ok(Value::Null)
        }
        "down" | "up" => {
            let at = point(view, number("x")?, number("y")?);
            let kind = if op["op"] == "down" {
                NSEventType::LeftMouseDown
            } else {
                NSEventType::LeftMouseUp
            };
            mouse(
                view,
                &window,
                kind,
                at,
                op["count"].as_i64().unwrap_or(1) as isize,
            )?;
            Ok(Value::Null)
        }
        "wheel" => {
            wheel(view, &window, number("x")?, number("y")?, number("dy")?)?;
            Ok(Value::Null)
        }
        "key" => {
            let key = op["key"].as_str().ok_or("a key op without `key`")?;
            let mods: Vec<&str> = op["mods"]
                .as_array()
                .map(|mods| mods.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            key_event(&window, key, &mods)?;
            Ok(Value::Null)
        }
        other => Err(format!("unknown input op {other:?}")),
    }
}

/// Key window, WebView first responder, moves delivered; what the runner's session gave the window.
fn prepare(mtm: MainThreadMarker, view: &NSView, window: &NSWindow) -> Value {
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    window.makeKeyAndOrderFront(None);
    window.setAcceptsMouseMovedEvents(true);
    let responder = window.makeFirstResponder(Some(view));
    let frame = window.frame();
    let screen = NSScreen::mainScreen(mtm).map(|screen| {
        let size = screen.frame().size;
        (size.width, size.height, screen.backingScaleFactor())
    });
    json!({
        "key": window.isKeyWindow(),
        "active": app.isActive(),
        "first_responder": responder,
        "occlusion_visible": window.occlusionState().0 & 2 != 0,
        "window": [frame.origin.x, frame.origin.y, frame.size.width, frame.size.height],
        "scale": window.backingScaleFactor(),
        "screen": screen,
        "flipped": view.isFlipped(),
    })
}

/// A viewport CSS px point in window coordinates; CSS px are points at page zoom 1.
fn point(view: &NSView, x: f64, y: f64) -> NSPoint {
    let y = if view.isFlipped() {
        y
    } else {
        view.bounds().size.height - y
    };
    view.convertPoint_toView(NSPoint::new(x, y), None)
}

fn now() -> f64 {
    NSProcessInfo::processInfo().systemUptime()
}

/// The deepest view under `at`, as AppKit would hit-test a real event.
fn target(view: &NSView, at: NSPoint) -> Retained<NSView> {
    // SAFETY: read on the main thread; nothing reparents the WebView meanwhile.
    let local =
        unsafe { view.superview() }.map_or(at, |parent| parent.convertPoint_fromView(at, None));
    view.hitTest(local).unwrap_or_else(|| view.retain())
}

/// Buttons go through `sendEvent:` like a real click; moves go straight to the view under the
/// point, as WebKitTestRunner does, since a window routes them by tracking area, not by event.
fn mouse(
    view: &NSView,
    window: &NSWindow,
    kind: NSEventType,
    at: NSPoint,
    count: isize,
) -> Result<(), String> {
    let pressure = if kind == NSEventType::LeftMouseDown {
        1.0
    } else {
        0.0
    };
    let event = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
        kind,
        at,
        NSEventModifierFlags::empty(),
        now(),
        window.windowNumber(),
        None,
        0,
        count,
        pressure,
    )
    .ok_or("NSEvent refused a mouse event")?;
    match kind {
        NSEventType::MouseMoved => target(view, at).mouseMoved(&event),
        _ => window.sendEvent(&event),
    }
    Ok(())
}

/// A pixel-unit CGEvent wheel at the point, as a trackpad would send it; `dy` > 0 scrolls down.
fn wheel(view: &NSView, window: &NSWindow, x: f64, y: f64, dy: f64) -> Result<(), String> {
    let at = point(view, x, y);
    let on_screen = window.convertPointToScreen(at);
    // CGEvent wants global coordinates from the main screen's top-left.
    let mtm = MainThreadMarker::from(view);
    let top = NSScreen::screens(mtm)
        .firstObject()
        .map_or(0.0, |screen| screen.frame().size.height);
    let event = CGEvent::new_scroll_wheel_event2(
        None,
        CGScrollEventUnit::Pixel,
        1,
        -dy.round() as i32,
        0,
        0,
    )
    .ok_or("CGEvent refused a wheel event")?;
    CGEvent::set_location(
        Some(&event),
        objc2_core_foundation::CGPoint::new(on_screen.x, top - on_screen.y),
    );
    let event = NSEvent::eventWithCGEvent(&event).ok_or("NSEvent refused the CGEvent wheel")?;
    target(view, at).scrollWheel(&event);
    Ok(())
}

/// One key down and up to the first responder; `mods` among shift, ctrl, alt, meta.
fn key_event(window: &NSWindow, key: &str, mods: &[&str]) -> Result<(), String> {
    let (code, shifted) = keystroke(key).ok_or(format!("no macOS key for {key:?}"))?;
    let mut flags = NSEventModifierFlags::empty();
    for name in mods {
        flags |= match *name {
            "shift" => NSEventModifierFlags::Shift,
            "ctrl" => NSEventModifierFlags::Control,
            "alt" => NSEventModifierFlags::Option,
            "meta" => NSEventModifierFlags::Command,
            other => return Err(format!("unknown modifier {other:?}")),
        };
    }
    if shifted {
        flags |= NSEventModifierFlags::Shift;
    }
    let mut chars = key_chars(key);
    let letter = key
        .chars()
        .next()
        .filter(|c| key.len() == 1 && c.is_ascii_alphabetic());
    if let (Some(letter), true) = (letter, mods.contains(&"ctrl")) {
        // AppKit's characters for a Control chord are the C0 control code.
        chars = char::from(letter.to_ascii_lowercase() as u8 & 0x1f).to_string();
    } else if key == "Tab" && flags.contains(NSEventModifierFlags::Shift) {
        chars = "\u{19}".into();
    }
    let ignoring = if key.chars().count() == 1 {
        key.to_string()
    } else {
        chars.clone()
    };
    for kind in [NSEventType::KeyDown, NSEventType::KeyUp] {
        let event = NSEvent::keyEventWithType_location_modifierFlags_timestamp_windowNumber_context_characters_charactersIgnoringModifiers_isARepeat_keyCode(
            kind,
            NSPoint::new(0.0, 0.0),
            flags,
            now(),
            window.windowNumber(),
            None,
            &NSString::from_str(&chars),
            &NSString::from_str(&ignoring),
            false,
            code,
        )
        .ok_or("NSEvent refused a key event")?;
        window.sendEvent(&event);
    }
    Ok(())
}

/// What AppKit puts in `characters` for a DOM `key`: the text, or a function-key code point.
fn key_chars(key: &str) -> String {
    let special = match key {
        "Enter" => '\r',
        "Tab" => '\t',
        "Escape" => '\u{1b}',
        "Backspace" => '\u{7f}',
        "Delete" => '\u{f728}',
        "ArrowUp" => '\u{f700}',
        "ArrowDown" => '\u{f701}',
        "ArrowLeft" => '\u{f702}',
        "ArrowRight" => '\u{f703}',
        "Home" => '\u{f729}',
        "End" => '\u{f72b}',
        "PageUp" => '\u{f72c}',
        "PageDown" => '\u{f72d}',
        _ => return key.to_string(),
    };
    special.to_string()
}

/// The US-ANSI virtual key code (`kVK_*`) of a DOM `key`, and whether it takes Shift.
fn keystroke(key: &str) -> Option<(u16, bool)> {
    const NAMED: &[(&str, u16)] = &[
        ("Enter", 36),
        ("Tab", 48),
        (" ", 49),
        ("Backspace", 51),
        ("Escape", 53),
        ("Delete", 117),
        ("Home", 115),
        ("End", 119),
        ("PageUp", 116),
        ("PageDown", 121),
        ("ArrowLeft", 123),
        ("ArrowRight", 124),
        ("ArrowDown", 125),
        ("ArrowUp", 126),
    ];
    // Unshifted, shifted, key code.
    const CHARS: &[(&str, &str, u16)] = &[
        ("a", "A", 0),
        ("s", "S", 1),
        ("d", "D", 2),
        ("f", "F", 3),
        ("h", "H", 4),
        ("g", "G", 5),
        ("z", "Z", 6),
        ("x", "X", 7),
        ("c", "C", 8),
        ("v", "V", 9),
        ("b", "B", 11),
        ("q", "Q", 12),
        ("w", "W", 13),
        ("e", "E", 14),
        ("r", "R", 15),
        ("y", "Y", 16),
        ("t", "T", 17),
        ("1", "!", 18),
        ("2", "@", 19),
        ("3", "#", 20),
        ("4", "$", 21),
        ("6", "^", 22),
        ("5", "%", 23),
        ("=", "+", 24),
        ("9", "(", 25),
        ("7", "&", 26),
        ("-", "_", 27),
        ("8", "*", 28),
        ("0", ")", 29),
        ("]", "}", 30),
        ("o", "O", 31),
        ("u", "U", 32),
        ("[", "{", 33),
        ("i", "I", 34),
        ("p", "P", 35),
        ("l", "L", 37),
        ("j", "J", 38),
        ("'", "\"", 39),
        ("k", "K", 40),
        (";", ":", 41),
        ("\\", "|", 42),
        (",", "<", 43),
        ("/", "?", 44),
        ("n", "N", 45),
        ("m", "M", 46),
        (".", ">", 47),
        ("`", "~", 50),
    ];
    if let Some(&(_, code)) = NAMED.iter().find(|(name, _)| *name == key) {
        return Some((code, false));
    }
    CHARS
        .iter()
        .find(|(plain, shifted, _)| *plain == key || *shifted == key)
        .map(|&(_, shifted, code)| (code, shifted == key))
        // Text outside the US layout still types through `characters`; its `code` is wrong.
        .or_else(|| (key.chars().count() == 1).then_some((0, false)))
}
