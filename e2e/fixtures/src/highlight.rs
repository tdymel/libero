//! `Highlight`.

use dioxus::prelude::*;
use libero::components::{Highlight, Text};

use crate::Routes;

/// Todo 3983: three cases of one term, the longest of two overlapping terms, and a tint.
pub const ROUTES: Routes = &[(
    "/highlight",
    || rsx! {
        Text {
            Highlight {
                id: "highlight",
                text: "Rust for rust, RUST in dioxus apps",
                terms: vec!["rust".to_string(), "dio".to_string(), "dioxus".to_string()],
                color: "info",
            }
        }
    },
)];
