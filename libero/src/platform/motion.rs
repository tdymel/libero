use super::backend;

/// Whether the user asked for reduced motion, `false` where this build cannot
/// tell - which is what every motion started from Rust did before it asked.
///
/// For motion CSS cannot switch off: a `@media (prefers-reduced-motion)` arm
/// covers anything a stylesheet animates, but a smooth scroll requested from
/// Rust names its behaviour explicitly, and dioxus's `ScrollBehavior` has no
/// `auto` that would defer to the stylesheet.
///
/// **Only the wasm32 arm answers.**
pub(crate) fn prefers_reduced_motion() -> bool {
    backend::prefers_reduced_motion()
}
