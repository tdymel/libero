//! A real component with one defect planted through a prop. Most plants are
//! injected by the test instead (`tests/all/planted.rs`); these are the ones
//! only a prop can make.

use dioxus::prelude::*;

use crate::{Routes, segmented_control::SegmentedControlPage};

pub const ROUTES: Routes = &[(
    "/planted/segmented-control-readonly",
    || rsx! { SegmentedControlPage { readonly: true } },
)];
