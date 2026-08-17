use crate::components::common::DomApiError;

/// A single resolved, currently-valid element - obtained only via
/// [`DomApi::query_selector`](crate::components::common::DomApi::query_selector),
/// never held past the call that produced it.
pub trait ElementApi {
    fn focus(&self) -> Result<(), DomApiError>;
    fn blur(&self) -> Result<(), DomApiError>;
}
