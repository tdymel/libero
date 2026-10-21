// Components that report state back to the user.

mod alert;
#[cfg(feature = "circular-progress")]
mod circular_progress;
mod loader;
#[cfg(feature = "notifications")]
mod notifications;
#[cfg(feature = "progress-bar")]
mod progress_bar;
#[cfg(any(feature = "circular-progress", feature = "progress-bar"))]
mod progress_value;
#[cfg(feature = "skeleton")]
mod skeleton;

pub use alert::{Alert, AlertPart, AlertProps};
#[cfg(feature = "circular-progress")]
pub use circular_progress::{CircularProgress, CircularProgressPart, CircularProgressProps};
pub use loader::{Loader, LoaderProps, LoaderVariant};
#[cfg(feature = "notifications")]
pub use notifications::{
    NotificationData, NotificationHandle, NotificationId, NotificationLive, NotificationOptions,
    NotificationScope, Notifications, NotificationsProps, SnackbarAction, SnackbarData, snackbar,
    use_notifications, use_notifications_with,
};
#[cfg(feature = "progress-bar")]
pub use progress_bar::{ProgressBar, ProgressBarPart, ProgressBarProps, ProgressBarSegment};
#[cfg(feature = "skeleton")]
pub use skeleton::{Skeleton, SkeletonProps};

#[cfg(test)]
#[cfg(any(feature = "circular-progress", feature = "progress-bar"))]
pub(crate) use progress_value::sheet_hex;
