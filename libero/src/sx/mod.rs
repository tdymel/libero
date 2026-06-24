mod context;
mod declaration;

mod optimizer;
mod static_value;
mod sx;

pub(crate) use context::class_name_from_id;
pub use context::{SxContext, use_sx};
pub use declaration::StaticDeclaration;
pub use optimizer::optimize_styles;
pub use sx::{NestedRule, Sx, SxDyn, sx};
