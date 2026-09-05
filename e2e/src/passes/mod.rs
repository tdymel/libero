//! The passes.
//!
//! Each is a small async fn over a `Fixture`, so a component's test composes
//! the ones that apply to it. They are deliberately *not* wrapped in a builder
//! DSL: with two components in the suite an abstraction over them would be
//! invented before anything had asked for it, and a plain call reads better
//! than a fluent one that does the same thing.
//!
//! The composition that does pay off lives one level up, in `archetypes`: a
//! whole interaction pattern parameterised once, so every component sharing it
//! inherits the same assertions.

pub mod console;
pub mod contrast;
pub mod dismissal;
pub mod focus;
pub mod keyboard;
pub mod live_region;
pub mod motion;
pub mod pointer;
pub mod target_size;
