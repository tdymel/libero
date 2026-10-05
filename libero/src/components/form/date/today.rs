use chrono::NaiveDate;
use dioxus::prelude::*;

use crate::platform::clock;

/// Today, asked again each time `open` turns true: a page open over midnight moves on.
pub(super) fn use_today_on_open(fixed: Option<NaiveDate>, open: bool) -> Option<NaiveDate> {
    use_refreshed_today(fixed, open).0
}

/// Today and a refresh that asks the clock again, for an inline picker's focus-in (2341).
pub(super) fn use_today_with_refresh(
    fixed: Option<NaiveDate>,
) -> (Option<NaiveDate>, TodayRefresh) {
    use_refreshed_today(fixed, false)
}

/// `fixed`, else today from the platform clock.
/// Asked after mount so hydration matches the server render: the first render has no today.
fn use_refreshed_today(fixed: Option<NaiveDate>, open: bool) -> (Option<NaiveDate>, TodayRefresh) {
    let today = use_signal(|| None);
    let refresh = TodayRefresh { today, fixed };
    // A fixed today skips the clock write, which would render the owner a second time.
    use_effect(use_reactive!(|fixed, open| {
        if open || today.peek().is_none() {
            TodayRefresh { today, fixed }.ask();
        }
    }));
    (fixed.or(today()), refresh)
}

/// Asks the platform clock again, writing only a changed day.
#[derive(Clone, Copy)]
pub(super) struct TodayRefresh {
    today: Signal<Option<NaiveDate>>,
    fixed: Option<NaiveDate>,
}

impl TodayRefresh {
    pub(super) fn ask(mut self) {
        if self.fixed.is_some() {
            return;
        }
        let next = clock().map(|clock| clock.today());
        if *self.today.peek() != next {
            self.today.set(next);
        }
    }
}
