use super::backend;

/// Whether the user asked for reduced motion, `false` where unknown. For motion
/// started from Rust, e.g. a smooth scroll, which no `@media` arm can reach.
///
/// `LiberoProvider`'s answer in the sheets wins: Blitz's, or a forced one (954).
pub(crate) fn prefers_reduced_motion() -> bool {
    super::current_a11y_answers()
        .reduced_motion
        .unwrap_or_else(backend::prefers_reduced_motion)
}
