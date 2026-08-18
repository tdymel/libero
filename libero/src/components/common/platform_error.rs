/// Why a platform-backed call (the DOM, the clipboard, ...) couldn't be
/// served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformError {
    /// This platform has no such capability - e.g. any DOM call off the web.
    Unsupported,
    /// The platform supports it; the thing asked for wasn't there.
    NotFound,
}
