use dioxus::prelude::*;

/// Stacks the open floating windows: the last one raised is on top.
///
/// Ids, not a counter, so z-indices stay dense however often windows are raised.
/// Capped below `overlay` so a modal covers every window; windows past the cap tie.
#[derive(Clone, Copy)]
pub(crate) struct WindowHost {
    stack: Signal<Vec<u64>>,
    layers: ReadSignal<ZLayers>,
}

/// A stack's z-index run, read off the active theme: `base`, then `step` per
/// place, below `ceiling`.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct ZLayers {
    pub(crate) base: i32,
    pub(crate) step: i32,
    pub(crate) ceiling: i32,
}

impl ZLayers {
    pub(crate) fn at(self, position: usize) -> i32 {
        z_index_at(self.base, self.step, self.ceiling, position)
    }
}

impl WindowHost {
    pub(crate) fn new(stack: Signal<Vec<u64>>, layers: ReadSignal<ZLayers>) -> Self {
        Self { stack, layers }
    }

    /// Puts `id` on top, adding it if it is new. A no-op when it already is.
    pub(crate) fn raise(&self, id: u64) {
        if self.stack.peek().last() == Some(&id) {
            return;
        }
        let mut stack = self.stack;
        let mut stack = stack.write();
        stack.retain(|other| *other != id);
        stack.push(id);
    }

    /// Whether `id` is on top. Unsubscribed: a key callback asks it.
    pub(crate) fn is_top(&self, id: u64) -> bool {
        self.stack.peek().last() == Some(&id)
    }

    pub(crate) fn remove(&self, id: u64) {
        let mut stack = self.stack;
        stack.write().retain(|other| *other != id);
    }

    /// `id`'s z-index, subscribed: a raise re-renders every window. `base`
    /// for an id that is not stacked yet.
    pub(crate) fn z_index(&self, id: u64) -> i32 {
        let position = self
            .stack
            .read()
            .iter()
            .position(|other| *other == id)
            .unwrap_or(0);
        self.layers.read().at(position)
    }
}

/// The z-index of the window `position` places from the bottom: one step
/// each, never reaching `ceiling`.
pub(crate) fn z_index_at(base: i32, step: i32, ceiling: i32, position: usize) -> i32 {
    let position = i32::try_from(position).unwrap_or(i32::MAX);
    base.saturating_add(step.saturating_mul(position))
        .min(ceiling - 1)
}

#[cfg(test)]
mod tests {
    use super::z_index_at;

    #[test]
    fn each_window_above_steps_once() {
        assert_eq!(z_index_at(250, 1, 300, 0), 250);
        assert_eq!(z_index_at(250, 1, 300, 3), 253);
    }

    /// However many windows are open, none reaches the overlay layer.
    #[test]
    fn the_top_is_capped_below_the_ceiling() {
        assert_eq!(z_index_at(250, 1, 300, 49), 299);
        assert_eq!(z_index_at(250, 1, 300, 50), 299);
        assert_eq!(z_index_at(250, 1, 300, usize::MAX), 299);
    }
}
