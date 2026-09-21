//! The home page's live card: a booking form and the bookings it made, in two
//! tabs. The e2e fixture keeps its own copy (`e2e/fixtures/src/home.rs`, todo 940).

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{
        Box, Button, DateField, Fields, Form, Options, Paper, Rule, Select, Switch, Table, Tabs,
        TextField, column, not_empty, use_form, use_notifications,
    },
    sx::sx,
};

use super::icon_label;
use crate::icons::{CalendarCheckIcon, UtensilsIcon};

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
    #[option(label = "Book a table")]
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
            sx: sx().padding("lg").width("100%"),
            Tabs {
                value: tab(),
                onchange: move |next| tab.set(next),
                option_label: |tab: CardTab| {
                    let icon = match tab {
                        CardTab::Book => rsx! { UtensilsIcon {} },
                        CardTab::Bookings => rsx! { CalendarCheckIcon {} },
                    };
                    icon_label(tab.label(), icon)
                },
                // Blitz sizes a tab to its min-content and breaks "Your bookings".
                sx: sx().selector("& [role=tab]", sx().white_space("nowrap")),
                // The form stays mounted under the table and sets the card's height, so a
                // switch never resizes it and a long table scrolls instead of growing it.
                panel: move |selected| rsx! {
                    Box { sx: sx().position("relative"),
                        Box {
                            inert: (selected != CardTab::Book).then_some(true),
                            sx: if selected == CardTab::Book {
                                sx()
                            } else {
                                sx().with("visibility", "hidden")
                            },
                            Form {
                                "aria-label": "Book a table",
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
                        }
                        if selected == CardTab::Bookings {
                            Box {
                                sx: sx()
                                    .position("absolute")
                                    .inset("0")
                                    .overflow("auto")
                                    .padding_top("md"),
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
                            }
                        }
                    }
                },
            }
        }
    }
}
