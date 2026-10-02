use chrono::NaiveDate;
use dioxus::prelude::*;

use crate::platform::clock;

/// `fixed`, else today from the platform clock.
/// Asked after mount so hydration matches the server render: the first render has no today.
pub(super) fn use_today(fixed: Option<NaiveDate>) -> Option<NaiveDate> {
    let mut today = use_signal(|| None);
    // A fixed today skips the clock write, which would render the owner a second time.
    use_effect(use_reactive!(|fixed| {
        if fixed.is_none() && today.peek().is_none() {
            today.set(clock().map(|clock| clock.today()));
        }
    }));
    fixed.or(today())
}
