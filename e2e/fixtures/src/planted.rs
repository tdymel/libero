//! Real components with one defect planted through a prop, the ones the test
//! (`tests/all/planted.rs`) cannot inject.

use dioxus::prelude::*;

use crate::{Routes, segmented_control::SegmentedControlPage};

pub const ROUTES: Routes = &[(
    "/planted/segmented-control-readonly",
    || rsx! { SegmentedControlPage { readonly: true } },
)];
