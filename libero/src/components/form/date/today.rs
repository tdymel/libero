use chrono::NaiveDate;
use dioxus::prelude::*;

use crate::platform::clock;

/// Today, from the platform clock - or `fixed`, when the caller names one.
///
/// Asked after mount: a server render has no clock, and a client answering
/// while it hydrates would not match it. So the first render never knows
/// today, and neither does a platform without a clock.
pub(super) fn use_today(fixed: Option<NaiveDate>) -> Option<NaiveDate> {
    let mut today = use_signal(|| None);
    use_effect(move || today.set(clock().map(|clock| clock.today())));
    fixed.or(today())
}
