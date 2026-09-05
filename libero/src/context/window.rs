use dioxus::prelude::*;

/// Stacks the open floating windows: the last one raised is on top.
///
/// A stack of ids rather than a counter, so the z-indices stay a dense run
/// `window, window + step, ..` however often windows are raised - a counter
/// would climb with every click. The top is still capped below `overlay`, so
/// a modal and its overlay cover every window even with more windows open
/// than the gap between the two layers holds; the ones past the cap then tie.
#[derive(Clone, Copy)]
pub(crate) struct WindowHost {
    stack: Signal<Vec<u64>>,
    base: i32,
    step: i32,
    ceiling: i32,
}

impl WindowHost {
    pub(crate) fn new(stack: Signal<Vec<u64>>, base: i32, step: i32, ceiling: i32) -> Self {
        Self {
            stack,
            base,
            step,
            ceiling,
        }
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
        z_index_at(self.base, self.step, self.ceiling, position)
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
