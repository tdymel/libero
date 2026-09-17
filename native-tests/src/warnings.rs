//! Collects `dioxus_signals` warnings, which a native-only path can raise where
//! no browser test sees it (todo 719).

use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use tracing::{
    Event, Level, Metadata, Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id, Record},
    subscriber::{self, DefaultGuard},
};

/// Every `dioxus_signals` WARN on this thread while it lives, first line only.
pub(crate) struct SignalWarnings {
    seen: Arc<Mutex<Vec<String>>>,
    _guard: DefaultGuard,
}

impl SignalWarnings {
    pub(crate) fn watch() -> Self {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let guard = subscriber::set_default(Collector(seen.clone()));
        Self {
            seen,
            _guard: guard,
        }
    }

    pub(crate) fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.seen.lock().unwrap())
    }
}

struct Collector(Arc<Mutex<Vec<String>>>);

impl Subscriber for Collector {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        *metadata.level() <= Level::WARN && metadata.target().starts_with("dioxus_signals")
    }

    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }

    fn record(&self, _: &Id, _: &Record<'_>) {}

    fn record_follows_from(&self, _: &Id, _: &Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut message = Message(String::new());
        event.record(&mut message);
        let line = message.0.lines().next().unwrap_or_default().to_owned();
        self.0.lock().unwrap().push(line);
    }

    fn enter(&self, _: &Id) {}

    fn exit(&self, _: &Id) {}
}

struct Message(String);

impl Visit for Message {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.0 = format!("{value:?}");
        }
    }
}
