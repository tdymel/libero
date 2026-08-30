use chrono::NaiveDate;

/// The wall clock, for "today".
pub trait ClockApi {
    /// The local calendar day.
    fn today(&self) -> NaiveDate;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use chrono::NaiveDate;

    use super::ClockApi;

    pub(super) struct WebClock;

    impl ClockApi for WebClock {
        /// JS `Date` already answers in the browser's time zone, so no time
        /// zone database is needed.
        fn today(&self) -> NaiveDate {
            let now = js_sys::Date::new_0();
            NaiveDate::from_ymd_opt(
                now.get_full_year() as i32,
                now.get_month() + 1,
                now.get_date(),
            )
            .expect("JS Date answers a real day")
        }
    }

    pub(super) static CLOCK: WebClock = WebClock;
}

/// `None` off the web for now. A native build has the time in UTC, but not
/// the local offset without a time zone database, and a day in UTC is the
/// wrong day for half the world in the evening.
///
/// Call it after mount, never while rendering: a server render and the
/// hydrating client would disagree, if not on the answer then on the day.
pub fn clock() -> Option<&'static dyn ClockApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::CLOCK);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}
