use chrono::NaiveDate;
use dioxus::prelude::*;

use crate::platform::clock;

/// `fixed`, else today from the platform clock.
/// Asked after mount so hydration matches the server render: the first render has no today.
pub(super) fn use_today(fixed: Option<NaiveDate>) -> Option<NaiveDate> {
    let mut today = use_signal(|| None);
    use_effect(move || today.set(clock().map(|clock| clock.today())));
    fixed.or(today())
}
