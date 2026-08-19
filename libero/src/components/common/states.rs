#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct States(Vec<(&'static str, bool)>);

impl States {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, state: &'static str, active: bool) -> Self {
        self.0.retain(|(existing, _)| existing != &state);
        self.0.push((state, active));
        self
    }

    pub fn active(self, state: &'static str) -> Self {
        self.with(state, true)
    }

    pub fn inactive(self, state: &'static str) -> Self {
        self.with(state, false)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, bool)> + '_ {
        self.0.iter().copied()
    }

    /// The space-joined names of the active states - the value for the
    /// element's `data-state` attribute, or `None` when none are active.
    pub fn data_state(&self) -> Option<String> {
        let value = self
            .0
            .iter()
            .filter_map(|(state, active)| active.then_some(*state))
            .collect::<Vec<_>>()
            .join(" ");

        (!value.is_empty()).then_some(value)
    }
}

impl From<Vec<(&'static str, bool)>> for States {
    fn from(value: Vec<(&'static str, bool)>) -> Self {
        Self(value)
    }
}

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
