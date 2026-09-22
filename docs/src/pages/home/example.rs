use dioxus::prelude::*;
use libero::{
    components::{Box, CodeBlock, Flex, Options, Tabs},
    sx::sx,
    theme::Size,
};

use super::{SectionTitle, booking::BookingCard, icon_label};
use crate::icons::FileIcon;

// The card beside it is this code, less the details: it needs the three files together.
// snippet: after FORM_CODE, TABLE_CODE
const CARD_CODE: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum CardTab {
    #[option(label = "Book a table")]
    Book,
    #[option(label = "Your bookings")]
    Bookings,
}

#[component]
pub fn BookingCard() -> Element {
    let mut tab = use_signal(|| CardTab::Book);
    let mut bookings = use_signal(Vec::new);

    rsx! {
        Paper { shadow: "lg", radius: "lg", sx: sx().padding("lg"),
            Tabs {
                value: tab(),
                onchange: move |next| tab.set(next),
                panel: move |tab| match tab {
                    CardTab::Book => rsx! {
                        BookingForm { onbook: move |booking| bookings.push(booking) }
                    },
                    CardTab::Bookings => rsx! {
                        BookingsTable { bookings: bookings() }
                    },
                },
            }
        }
    }
}"#;

const FORM_CODE: &str = r#"use libero::chrono::NaiveDate;

#[derive(Clone, Copy, PartialEq, Options)]
enum Guests { One, Two, Four, Six }

#[derive(Clone, PartialEq, Default, Fields)]
struct Booking {
    name: String,
    day: Option<NaiveDate>,
    guests: Option<Guests>,
    terrace: bool,
}

#[component]
fn BookingForm(onbook: EventHandler<Booking>) -> Element {
    let booking = use_store(Booking::default);
    let notify = use_notifications();

    rsx! {
        Form {
            value: booking,
            onsubmit: move |_| {
                onbook.call(booking());
                notify.show("Table booked.");
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
                name: Booking::FIELDS.guests(),
                validate: not_empty.error("Pick how many guests."),
            }
            Switch { label: "On the terrace", name: Booking::FIELDS.terrace() }
            Button { r#type: "submit", "Book" }
        }
    }
}"#;

// snippet: after FORM_CODE
const TABLE_CODE: &str = r#"#[component]
fn BookingsTable(bookings: Vec<Booking>) -> Element {
    rsx! {
        Table {
            caption: "Your bookings",
            data: bookings,
            columns: vec![
                column("Name").value(|b: &Booking| b.name.clone()).sortable(),
                column("Day")
                    .value(|b: &Booking| b.day.map(|d| d.to_string()).unwrap_or_default())
                    .sortable(),
                column("Terrace")
                    .value(|b: &Booking| if b.terrace { "Yes" } else { "No" }.to_string()),
            ],
        }
    }
}"#;

// copy: card-label
const CARD_LABEL: &str = "The booking card, Rust code";
// copy: end

#[derive(Clone, Copy, PartialEq, Options)]
enum File {
    #[option(label = "booking_card.rs")]
    Card,
    #[option(label = "booking_form.rs")]
    Form,
    #[option(label = "bookings_table.rs")]
    Table,
}

impl File {
    fn code(self) -> &'static str {
        match self {
            File::Card => CARD_CODE,
            File::Form => FORM_CODE,
            File::Table => TABLE_CODE,
        }
    }

    // Names the block and its copy button in the visitor's words, not the file name.
    fn code_label(self) -> &'static str {
        match self {
            File::Card => CARD_LABEL,
            File::Form => "The booking form, Rust code",
            File::Table => "The bookings table, Rust code",
        }
    }
}

/// The booking code by file on the left, the card it makes on the right.
#[component]
pub fn Example() -> Element {
    rsx! {
        section { "aria-labelledby": "example-title",
            Flex { direction: "column", gap: "lg",
                SectionTitle { id: "example-title", "Libero in action" }
                Flex {
                    direction: "column",
                    gap: "xl",
                    sx: sx().breakpoint(Size::Md, sx().flex_direction("row")),
                    // The card sets the row's height, so the code sits out of the flow and scrolls.
                    Box {
                        sx: sx()
                            .position("relative")
                            .height("28rem")
                            .min_width("0")
                            .breakpoint(Size::Md, sx().height("auto").flex("1 1 0")),
                        Box { sx: sx().position("absolute").inset("0"), Files {} }
                    }
                    // Full width when stacked, a fixed column beside the code.
                    Box { sx: sx().breakpoint(Size::Md, sx().flex("0 1 440px")), BookingCard {} }
                }
            }
        }
    }
}

#[component]
fn Files() -> Element {
    let mut open = use_signal(|| File::Card);

    rsx! {
        Tabs {
            value: open(),
            onchange: move |next| open.set(next),
            "aria-label": "Files",
            option_label: |file: File| icon_label(file.label(), rsx! { FileIcon {} }),
            sx: sx()
                .display("flex")
                .flex_direction("column")
                .height("100%")
                .selector("& [role=tabpanel]", sx().flex("1").min_height("0")),
            panel: |file: File| rsx! {
                CodeBlock {
                    source: file.code(),
                    language: "rust",
                    label: file.code_label(),
                    header: false,
                    sx: sx()
                        .display("flex")
                        .flex_direction("column")
                        .height("100%")
                        .selector("& > div:last-child", sx().flex("1").min_height("0")),
                }
            },
        }
    }
}
