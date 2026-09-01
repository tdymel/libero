mod glyphs;
#[allow(clippy::module_inception)]
mod pagination;
mod range;

pub use pagination::{Pagination, PaginationLabel, PaginationProps};
pub use range::{PaginationItem, pagination_range};
