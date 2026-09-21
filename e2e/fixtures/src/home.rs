//! A copy of the docs home page's booking card: e2e never includes docs source
//! (todo 940), so a change to `docs/src/pages/home/booking.rs` is copied by hand.

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{
        Button, CodeBlock, DateField, Fields, Flex, Form, Notifications, Options, Paper, Rule,
        Select, Switch, Table, Tabs, TextField, Title, VisuallyHidden, column, not_empty, use_form,
        use_notifications,
    },
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/home-booking", || {
        rsx! {
            BookingCard {}
            Notifications {}
        }
    }),
    ("/home-stats", || rsx! { Stats {} }),
    ("/home-code", || rsx! { HomeCode {} }),
];

/// A copy of the "Libero in numbers" section of `docs/src/pages/home/stats.rs`, cards cut to one.
#[component]
fn Stats() -> Element {
    rsx! {
        section { "aria-labelledby": "stats-title",
            Title { component: "h2", id: "stats-title", sx: sx().margin("0"),
                VisuallyHidden { "Libero in numbers" }
            }
            Paper { bordered: true, "100+ Components and Hooks" }
        }
    }
}

/// The landing page's code blocks, copied like the card: `hero.rs`, `closing.rs`, `example.rs` (todo 1025).
#[component]
fn HomeCode() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            CodeBlock {
                id: "install",
                source: "cargo add libero",
                language: "shell",
                label: "Add libero to your project",
                header: false,
                line_numbers: false,
            }
            CodeBlock {
                id: "card-code",
                source: "#[component]\npub fn BookingCard() -> Element {{\n    rsx! {{ Paper {{}} }}\n}}",
                language: "rust",
                label: "The booking card, Rust code",
                header: false,
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Guests {
    #[option(label = "1 guest")]
    One,
    #[option(label = "2 guests")]
    Two,
    #[option(label = "4 guests")]
    Four,
    #[option(label = "6 guests")]
    Six,
}

impl Guests {
    fn count(self) -> u8 {
        match self {
            Guests::One => 1,
            Guests::Two => 2,
            Guests::Four => 4,
            Guests::Six => 6,
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Booking {
    name: String,
    day: Option<NaiveDate>,
    guests: Option<Guests>,
    terrace: bool,
}

#[derive(Clone, PartialEq)]
struct Row {
    name: String,
    day: NaiveDate,
    guests: u8,
    terrace: bool,
}

#[derive(Clone, Copy, PartialEq, Options)]
enum CardTab {
    Book,
    #[option(label = "Your bookings")]
    Bookings,
}

fn row(name: &str, (y, m, d): (i32, u32, u32), guests: u8, terrace: bool) -> Row {
    Row {
        name: name.into(),
        day: NaiveDate::from_ymd_opt(y, m, d).unwrap(),
        guests,
        terrace,
    }
}

#[component]
pub fn BookingCard() -> Element {
    let booking = use_store(Booking::default);
    let mut bookings = use_signal(|| {
        vec![
            row("Ada", (2026, 10, 2), 2, true),
            row("Grace", (2026, 10, 9), 6, false),
        ]
    });
    let mut tab = use_signal(|| CardTab::Book);
    let form = use_form();
    let notify = use_notifications();

    rsx! {
        Paper {
            shadow: "lg",
            radius: "lg",
            sx: sx().padding("lg").width("100%").max_width("440px"),
            Title { size: "md", component: "h3", id: "booking-title", "Book a table" }
            Tabs {
                value: tab(),
                onchange: move |next| tab.set(next),
                // Blitz sizes a tab to its min-content and breaks "Your bookings".
                sx: sx().selector("& [role=tab]", sx().white_space("nowrap")),
                panel: move |selected| match selected {
                    CardTab::Book => rsx! {
                        Form {
                            "aria-labelledby": "booking-title",
                            value: booking,
                            form,
                            sx: sx().padding_top("md"),
                            onsubmit: move |_| {
                                let value = booking();
                                let (Some(day), Some(guests)) = (value.day, value.guests) else {
                                    return;
                                };
                                notify.show(format!("Booked a table for {}.", value.name.trim()));
                                bookings.push(Row {
                                    name: value.name.trim().to_string(),
                                    day,
                                    guests: guests.count(),
                                    terrace: value.terrace,
                                });
                                form.reset();
                            },
                            TextField {
                                label: "Name",
                                name: Booking::FIELDS.name(),
                                validate: not_empty.error("Enter a name."),
                            }
                            DateField {
                                label: "Day",
                                name: Booking::FIELDS.day(),
                                validate: not_empty.error("Pick a day."),
                            }
                            Select {
                                label: "Guests",
                                placeholder: "How many?",
                                name: Booking::FIELDS.guests(),
                                validate: not_empty.error("Pick how many guests."),
                            }
                            Switch { label: "On the terrace", name: Booking::FIELDS.terrace() }
                            Button { r#type: "submit", "Book" }
                        }
                    },
                    CardTab::Bookings => rsx! {
                        Table {
                            caption: "Your bookings",
                            data: bookings(),
                            columns: vec![
                                column("Name").value(|r: &Row| r.name.clone()).sortable(),
                                column("Day").value(|r: &Row| r.day.format("%Y-%m-%d").to_string()).sortable(),
                                column("Guests").value(|r: &Row| r.guests).sortable(),
                                column("Seat").value(|r: &Row| if r.terrace { "Terrace" } else { "Inside" }.to_string()),
                            ],
                        }
                    },
                },
            }
        }
    }
}
