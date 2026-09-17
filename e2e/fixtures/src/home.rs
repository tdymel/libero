//! The docs home page's booking card, from the docs' own source, so the test
//! drives the card the page renders.

#[path = "../../../docs/src/pages/home/booking.rs"]
mod booking;

use dioxus::prelude::*;
use libero::components::Notifications;

use crate::Routes;

pub const ROUTES: Routes = &[("/home-booking", || {
    rsx! {
        booking::BookingCard {}
        Notifications {}
    }
})];
