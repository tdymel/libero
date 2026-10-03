/// Named on/off states, rendered as the element's `data-state`, which `sx().when(..)` matches.
/// Setting a state twice keeps the last.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct States(Vec<(&'static str, bool)>);

impl States {
    /// No state yet, the same as [`states()`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `state` on or off, moving it to the end.
    pub fn with(mut self, state: &'static str, active: bool) -> Self {
        self.0.retain(|(existing, _)| existing != &state);
        self.0.push((state, active));
        self
    }

    /// [`with`](Self::with) `state` on.
    pub fn active(self, state: &'static str) -> Self {
        self.with(state, true)
    }

    /// [`with`](Self::with) `state` off, which overrides an earlier `active`.
    pub fn inactive(self, state: &'static str) -> Self {
        self.with(state, false)
    }

    /// Every state set, on or off, in order.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, bool)> + '_ {
        self.0.iter().copied()
    }

    /// The element's `data-state` value, or `None` when nothing is active.
    pub fn data_state(&self) -> Option<String> {
        let active = || self.0.iter().filter(|(_, active)| *active).map(|(s, _)| *s);

        let length: usize = active().map(|state| state.len() + 1).sum();
        if length == 0 {
            return None;
        }

        let mut value = String::with_capacity(length - 1);
        for state in active() {
            if !value.is_empty() {
                value.push(' ');
            }
            value.push_str(state);
        }
        Some(value)
    }
}

impl From<Vec<(&'static str, bool)>> for States {
    fn from(value: Vec<(&'static str, bool)>) -> Self {
        Self(value)
    }
}

/// Starts an empty [`States`].
///
/// ```
/// # use libero::components::states;
/// let states = states().active("open").inactive("disabled");
/// assert_eq!(states.data_state().as_deref(), Some("open"));
/// ```
pub fn states() -> States {
    States::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_state_lists_only_the_active_ones() {
        let states = states().active("open").inactive("disabled").active("hover");

        assert_eq!(states.data_state().as_deref(), Some("open hover"));
    }

    #[test]
    fn no_active_state_renders_no_attribute() {
        assert_eq!(states().inactive("open").data_state(), None);
        assert_eq!(states().data_state(), None);
    }

    #[test]
    fn setting_a_state_twice_keeps_only_the_last_value() {
        let states = states().active("open").inactive("open");

        assert_eq!(states.data_state(), None);
        assert_eq!(states.iter().count(), 1);
    }

    #[test]
    fn re_setting_a_state_moves_it_to_the_end() {
        let states = states().active("a").active("b").active("a");

        assert_eq!(states.data_state().as_deref(), Some("b a"));
    }

    #[test]
    fn with_takes_the_flag_from_its_argument() {
        let states = states().with("open", true).with("disabled", false);

        assert_eq!(states.data_state().as_deref(), Some("open"));
    }
}
