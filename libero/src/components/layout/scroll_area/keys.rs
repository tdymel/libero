//! The browser's scrolling keys for a focused area, where the renderer has none
//! (Blitz, todo 1267).

use dioxus::prelude::*;

use crate::{
    hooks::ElementHandle,
    platform::{ElementApi, arrow_target, key_taken, typing_target},
};

/// An arrow's step in px, as Chromium's.
const LINE: f64 = 40.0;
/// A page keeps this share of the view in sight, as Chromium's.
const PAGE: f64 = 0.875;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Move {
    By(f64, f64),
    Top,
    Bottom,
}

/// What `key` does to the area; Space pages only with focus on the area itself.
fn key_move(key: &Key, shift: bool, on_area: bool, view_height: f64) -> Option<Move> {
    let page = (view_height * PAGE).max(LINE);
    Some(match key {
        Key::Named(NamedKey::ArrowUp) => Move::By(0.0, -LINE),
        Key::Named(NamedKey::ArrowDown) => Move::By(0.0, LINE),
        Key::Named(NamedKey::ArrowLeft) => Move::By(-LINE, 0.0),
        Key::Named(NamedKey::ArrowRight) => Move::By(LINE, 0.0),
        Key::Named(NamedKey::PageUp) => Move::By(0.0, -page),
        Key::Named(NamedKey::PageDown) => Move::By(0.0, page),
        Key::Named(NamedKey::Home) => Move::Top,
        Key::Named(NamedKey::End) => Move::Bottom,
        Key::Character(space) if space == " " && on_area => {
            Move::By(0.0, if shift { -page } else { page })
        }
        _ => return None,
    })
}

/// Scrolls `root` on a key nothing nearer took. Taken here, so an outer area
/// does not scroll too.
pub(crate) fn scroll_on_key(root: ElementHandle, event: Event<KeyboardData>) {
    let modifiers = event.modifiers();
    if modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META)
        || key_taken(&event)
        || typing_target(&event)
    {
        return;
    }
    let key = event.key();
    let arrow = matches!(
        key,
        Key::Named(NamedKey::ArrowUp)
            | Key::Named(NamedKey::ArrowDown)
            | Key::Named(NamedKey::ArrowLeft)
            | Key::Named(NamedKey::ArrowRight)
    );
    if arrow && arrow_target(&event) {
        return;
    }
    let (shift, on_area) = (modifiers.contains(Modifiers::SHIFT), root.is_focused());
    // The view's height is read below; this only asks whether the key scrolls.
    if key_move(&key, shift, on_area, 0.0).is_none() {
        return;
    }
    event.prevent_default();
    let (size, view, offset) = (root.scroll_size(), root.dimensions(), root.scroll_offset());
    spawn(async move {
        let (Ok(size), Ok(view), Ok((x, y))) = (size.await, view.await, offset.await) else {
            return;
        };
        let (x, y) = match key_move(&key, shift, on_area, view.height) {
            Some(Move::By(dx, dy)) => (x + dx, y + dy),
            Some(Move::Top) => (x, 0.0),
            Some(Move::Bottom) => (x, (size.height - view.height).max(0.0)),
            None => return,
        };
        let _ = root.scroll_to(x, y);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_step_a_line_and_pages_most_of_the_view() {
        assert_eq!(
            key_move(&Key::Named(NamedKey::ArrowDown), false, false, 400.0),
            Some(Move::By(0.0, 40.0))
        );
        assert_eq!(
            key_move(&Key::Named(NamedKey::ArrowLeft), false, false, 400.0),
            Some(Move::By(-40.0, 0.0))
        );
        assert_eq!(
            key_move(&Key::Named(NamedKey::PageDown), false, false, 400.0),
            Some(Move::By(0.0, 350.0))
        );
        assert_eq!(
            key_move(&Key::Named(NamedKey::PageUp), false, false, 20.0),
            Some(Move::By(0.0, -40.0))
        );
        assert_eq!(
            key_move(&Key::Named(NamedKey::End), false, false, 400.0),
            Some(Move::Bottom)
        );
    }

    /// Space presses a focused button inside, so it pages only on the area.
    #[test]
    fn space_pages_only_with_focus_on_the_area() {
        let space = Key::Character(" ".into());
        assert_eq!(key_move(&space, false, false, 400.0), None);
        assert_eq!(
            key_move(&space, true, true, 400.0),
            Some(Move::By(0.0, -350.0))
        );
        assert_eq!(
            key_move(&Key::Character("a".into()), false, true, 400.0),
            None
        );
    }
}
