//! A column's pointer drag scrolls the board near its side. The drag itself is the board's
//! column sortable, which follows the scroll as it does a wheel's (2540).

use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, edge_scroll_step, use_interval},
    platform::ElementApi,
};

/// The edge scroll's tick, as a card drag's.
const AUTO_SCROLL_MS: u64 = 40;

/// The board read once a column drag moves: its left edge and width, the least and most
/// `scrollLeft`, `scrollLeft` since, and `scrollTop`, which a column drag leaves.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Board {
    left: f64,
    width: f64,
    range: (f64, f64),
    scroll: f64,
    top: f64,
}

impl Board {
    /// `scrollLeft` after one tick with the pointer at `x`, from `now`.
    fn next(&self, x: f64, now: f64) -> f64 {
        let step = edge_scroll_step(x, self.left, self.width);
        (now + step).clamp(self.range.0, self.range.1)
    }
}

/// What the board and its column handles share for a column's pointer drag.
#[derive(Clone, Copy)]
pub(super) struct ColumnDrag {
    /// A column handle is pressed by a pointer, until it is let go.
    pressed: CopyValue<bool>,
    /// The column list sorts: by this pointer drag or by keys.
    sorting: Memo<bool>,
    pointer: CopyValue<Option<f64>>,
    board: CopyValue<Option<Board>>,
    reading: CopyValue<bool>,
    element: ElementHandle,
    auto_scroll: crate::hooks::IntervalHandle,
}

impl ColumnDrag {
    /// A column handle's press, before the sortable's.
    pub(super) fn press(mut self) {
        self.pressed.set(true);
    }

    /// A column is lifted, by pointer or keys: a card press waits.
    pub(super) fn busy(self) -> bool {
        *self.sorting.peek()
    }

    /// The board's pointer move: near a side while a column drags, it scrolls that way.
    pub(super) fn track(mut self, x: f64) {
        if !*self.pressed.peek() || !*self.sorting.peek() {
            return;
        }
        self.pointer.set(Some(x));
        if self.board.peek().is_none() && !*self.reading.peek() {
            self.read();
        }
        if !self.auto_scroll.active() {
            self.auto_scroll.start();
        }
    }

    /// The pointer let go or was revoked.
    pub(super) fn release(mut self) {
        self.pressed.set(false);
        self.pointer.set(None);
        self.board.set(None);
        self.auto_scroll.stop();
    }

    fn read(mut self) {
        self.reading.set(true);
        let element = self.element;
        let rtl = element.is_rtl();
        let reads = (
            element.client_offset(),
            element.dimensions(),
            element.client_size(),
            element.scroll_size(),
            element.scroll_offset(),
        );
        let mut board = self.board;
        let mut reading = self.reading;
        let pressed = self.pressed;
        spawn(async move {
            let (at, size, client, content, scroll) = reads;
            let (at, size, content, scroll) = (at.await, size.await, content.await, scroll.await);
            reading.set(false);
            let (Ok((left, _)), Ok(size), Ok(content), Ok((scroll, top))) =
                (at, size, content, scroll)
            else {
                return;
            };
            // The client box leaves out the border and a classic scrollbar.
            let extent = client.await.map_or(size.width, |client| client.width);
            let most = (content.width - extent).max(0.0);
            // Right to left, `scrollLeft` runs from 0 down to minus the overflow.
            let range = if rtl { (-most, 0.0) } else { (0.0, most) };
            // A release before the read resolved leaves no board to restore.
            if *pressed.peek() {
                board.set(Some(Board {
                    left,
                    width: size.width,
                    range,
                    scroll,
                    top,
                }));
            }
        });
    }
}

pub(super) fn use_column_drag(element: ElementHandle, sorting: Memo<bool>) -> ColumnDrag {
    let pressed = use_hook(|| CopyValue::new(false));
    let pointer = use_hook(|| CopyValue::new(None::<f64>));
    let mut board = use_hook(|| CopyValue::new(None::<Board>));
    let reading = use_hook(|| CopyValue::new(false));
    let auto_scroll = use_interval(
        move || {
            let (Some(x), Some(read)) = (*pointer.peek(), *board.peek()) else {
                return;
            };
            let next = read.next(x, read.scroll);
            if next == read.scroll {
                return;
            }
            let _ = element.scroll_to(next, read.top);
            board.set(Some(Board {
                scroll: next,
                ..read
            }));
        },
        AUTO_SCROLL_MS,
    );
    let drag = ColumnDrag {
        pressed,
        sorting,
        pointer,
        board,
        reading,
        element,
        auto_scroll,
    };
    // A drop or a cancel by key ends the scroll too.
    use_effect(move || {
        if !sorting() && *drag.pressed.peek() {
            drag.release();
        }
    });
    drag
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOARD: Board = Board {
        left: 100.0,
        width: 600.0,
        range: (0.0, 400.0),
        scroll: 50.0,
        top: 0.0,
    };

    #[test]
    fn the_middle_of_the_board_scrolls_nothing() {
        assert_eq!(BOARD.next(400.0, 50.0), 50.0);
    }

    #[test]
    fn a_side_scrolls_that_way_and_stops_at_the_end() {
        assert!(BOARD.next(695.0, 50.0) > 50.0);
        assert!(BOARD.next(105.0, 50.0) < 50.0);
        assert_eq!(BOARD.next(105.0, 0.0), 0.0);
        assert_eq!(BOARD.next(695.0, 400.0), 400.0);
    }

    #[test]
    fn right_to_left_the_range_runs_below_zero() {
        let rtl = Board {
            range: (-400.0, 0.0),
            ..BOARD
        };
        assert!(rtl.next(105.0, 0.0) < 0.0);
        assert_eq!(rtl.next(695.0, 0.0), 0.0);
    }
}
