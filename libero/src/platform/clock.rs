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
        /// JS `Date` answers in the browser's time zone: no tz database needed.
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

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use chrono::{Local, NaiveDate};

    use super::ClockApi;

    pub(super) struct NativeClock;

    impl ClockApi for NativeClock {
        fn today(&self) -> NaiveDate {
            Local::now().date_naive()
        }
    }

    pub(super) static CLOCK: NativeClock = NativeClock;
}

/// The platform's clock: JS `Date` on the web, the system clock off it. Call it
/// after mount, never while rendering: server and hydrating client may disagree.
///
/// ```
/// let today = libero::platform::clock().map(|clock| clock.today());
/// # let _ = today;
/// ```
pub fn clock() -> Option<&'static dyn ClockApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::CLOCK);
    #[cfg(not(target_arch = "wasm32"))]
    return Some(&native::CLOCK);
}

/// Milliseconds since the Unix epoch, for cache ages; 0 if the clock is before it.
pub(crate) fn unix_millis() -> u64 {
    #[cfg(target_arch = "wasm32")]
    return js_sys::Date::now() as u64;
    #[cfg(not(target_arch = "wasm32"))]
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use chrono::{TimeDelta, Utc};

    use super::clock;

    #[test]
    fn the_native_clock_answers_a_day_within_one_of_utc() {
        let today = clock().expect("a native clock").today();
        let utc = Utc::now().date_naive();
        assert!((today - utc).abs() <= TimeDelta::days(1), "{today} {utc}");
    }
}
