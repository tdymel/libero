use dioxus::prelude::*;
use libero::{
    components::{Anchor, Box, Code, Flex, List, ListItem},
    sx::sx,
    theme::Size,
};

use super::{SectionHead, booking::BookingCard, tint};
use crate::Route;

/// The components the booking card is made of, each linking to its page.
const PARTS: [(&str, &str, Route); 7] = [
    (
        "Form",
        "validates on submit and focuses the error summary",
        Route::FormPage {},
    ),
    ("TextField", "the name", Route::TextFieldPage {}),
    (
        "DateField",
        "the day, typed or picked",
        Route::ChronoFieldPage {},
    ),
    ("Select", "the guests, over an enum", Route::SelectPage {}),
    ("Switch", "the terrace", Route::SwitchPage {}),
    ("Tabs", "the form and the bookings", Route::TabsPage {}),
    ("Table", "the bookings, sortable", Route::TablePage {}),
];

/// The booking card in a frame of its own, beside what it is made of.
#[component]
pub fn TryIt() -> Element {
    rsx! {
        section { "aria-labelledby": "try-title",
            Flex {
                direction: "column",
                gap: "xl",
                sx: sx().breakpoint(Size::Md, sx().flex_direction("row").align_items("center")),
                Flex { direction: "column", gap: "lg", sx: sx().min_width("0").breakpoint(Size::Md, sx().flex("1 1 0")),
                    SectionHead { id: "try-title", eyebrow: "Live demo", title: "A whole form, ready to use",
                        "Book a table, then open your bookings. A confirmation comes from "
                        Code { source: "Notifications" }
                        ", and every step works from the keyboard."
                    }
                    List { size: "sm",
                        for (name, what, route) in PARTS {
                            ListItem { key: "{name}",
                                Anchor { to: route, Code { source: name } }
                                " {what}"
                            }
                        }
                    }
                }
                Box {
                    sx: sx()
                        .display("flex")
                        .justify_content("center")
                        .padding("lg")
                        .border_radius("xl")
                        .background(tint(10))
                        .border(format!("1px solid {}", tint(25)))
                        .breakpoint(Size::Sm, sx().padding("xl"))
                        .breakpoint(Size::Md, sx().flex("1 1 0")),
                    BookingCard {}
                }
            }
        }
    }
}
