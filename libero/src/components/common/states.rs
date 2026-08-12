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

    pub fn active(mut self, state: &'static str) -> Self {
        self.0.retain(|(existing, _)| existing != &state);
        self.0.push((state, true));
        self
    }

    pub fn inactive(mut self, state: &'static str) -> Self {
        self.0.retain(|(existing, _)| existing != &state);
        self.0.push((state, false));
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, bool)> + '_ {
        self.0.iter().copied()
    }

    pub fn active_data_state(&self) -> Option<String> {
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
