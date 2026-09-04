mod dialog;
mod paper;

pub use dialog::{Dialog, DialogProps};
/// The surface definition itself, for a component that renders one as part of
/// its own element instead of nesting a `Paper` - see `PaperProps::framework_sx`.
pub use paper::paper_sx;
pub use paper::{Paper, PaperProps};
