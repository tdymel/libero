//! The E2E harness.
//!
//! Drives a real Chromium over CDP against the fixture app in `e2e/fixtures`.
//! See `.agents/brain/plans/13-e2e-pass/overview.md` for why this is raw CDP
//! rather than a Playwright binding, and `.agents/brain/codebase/e2e-harness.md`
//! for the model it follows.
//!
//! Entry point is `cargo run -p e2e`, not `cargo test -p e2e`: the runner owns
//! the fixture server's lifecycle and hands the tests its URL. Running the
//! tests directly fails with an explanatory message rather than a connection
//! error.

pub mod archetypes;
pub mod ax;
pub mod browser;
pub mod clock;
pub mod driver;
pub mod journal;
pub mod passes;
pub mod suite;
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
