/// Why a platform-backed call (DOM, clipboard, ...) couldn't be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformError {
    /// No such capability here - e.g. any DOM call off the web.
    Unsupported,
    /// Supported, but the thing asked for wasn't there.
    NotFound,
    /// Supported and attempted, but the platform refused - e.g. a clipboard
    /// write without permission or off a secure context.
    Denied,
}
