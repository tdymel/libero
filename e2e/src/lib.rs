//! The E2E harness: Chromium over CDP against `e2e/fixtures` (`codebase/e2e-harness`).
//! Run with `cargo run -p e2e`, not `cargo test`: the runner owns the fixture server.

#[cfg(feature = "android")]
pub mod android;
pub mod archetypes;
pub mod ax;
pub mod browser;
pub mod clock;
pub mod driver;
pub mod frames;
pub mod journal;
#[cfg(feature = "native")]
pub mod native;
pub mod passes;
pub mod suite;
pub mod sweep;
pub mod vendor;
pub mod wait;

pub use browser::{Fixture, Scheme, Viewport};
// For `scenario!`'s native arm.
pub use futures;
pub use suite::Suite;

/// Where the fixture server is listening. Set by the runner.
pub fn base_url() -> String {
    std::env::var("E2E_BASE_URL").unwrap_or_else(|_| {
        panic!(
            "E2E_BASE_URL is unset. The fixture server is started by the runner, \
             so run `cargo run -p e2e` rather than `cargo test -p e2e`."
        )
    })
}
