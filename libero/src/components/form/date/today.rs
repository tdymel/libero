use chrono::NaiveDate;
use dioxus::prelude::*;

use crate::platform::clock;

/// `fixed`, else today from the platform clock.
/// Asked after mount so hydration matches the server render: the first render has no today.
pub(super) fn use_today(fixed: Option<NaiveDate>) -> Option<NaiveDate> {
    use_today_on_open(fixed, false)
}

/// [`use_today`], asked again each time `open` turns true: a page open over midnight moves on.
pub(super) fn use_today_on_open(fixed: Option<NaiveDate>, open: bool) -> Option<NaiveDate> {
    let mut today = use_signal(|| None);
    // A fixed today skips the clock write, which would render the owner a second time.
    use_effect(use_reactive!(|fixed, open| {
        if fixed.is_some() || (!open && today.peek().is_some()) {
            return;
        }
        let next = clock().map(|clock| clock.today());
        if *today.peek() != next {
            today.set(next);
        }
    }));
    fixed.or(today())
}
