//! The Android WebView has no `Notification`: the framework's `NotificationManager`
//! over JNI on the main thread (todo 1348).

use std::cell::Cell;
use std::collections::HashMap;
use std::future::poll_fn;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, mpsc as std_mpsc};
use std::time::{Duration, Instant};

use dioxus::core::Task;
use dioxus::prelude::{debug, spawn, try_consume_context};
use dioxus_desktop::DesktopContext;
use dioxus_desktop::tao::event::Event;
use dioxus_desktop::wry::prelude::{
    JNIEnv, JObject, dispatch,
    jni::{errors::Result as JniResult, objects::JValue},
};
use futures_channel::{mpsc, oneshot};
use futures_core::Stream;

use super::{
    Answer, NotificationEvent, Shown, ShownNotification, SystemNotification, SystemNotificationApi,
    SystemNotificationError,
};
use crate::platform::PermissionState;

const POST_NOTIFICATIONS: &str = "android.permission.POST_NOTIFICATIONS";
const CHANNEL: &str = "libero";
/// A tap opens the activity with this scheme; tao hands the URL over as `Event::Opened`.
const SCHEME: &str = "libero-notification";
/// `FLAG_IMMUTABLE | FLAG_UPDATE_CURRENT`.
const PENDING_FLAGS: i32 = 0x0400_0000 | 0x0800_0000;
/// A dialog takes the focus within this; else the system answered without one.
const DIALOG_OPENS: Duration = Duration::from_secs(1);
const DIALOG_ANSWERED: Duration = Duration::from_secs(60);
const POLL: Duration = Duration::from_millis(100);

/// Set after the first request: an unanswered permission reads `Prompt` before it, `Denied` after.
static ASKED: AtomicBool = AtomicBool::new(false);
static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
/// Where each shown notification's tap goes, by the key in its URL.
static ROUTES: Mutex<Option<HashMap<u64, mpsc::UnboundedSender<NotificationEvent>>>> =
    Mutex::new(None);

thread_local! {
    static LISTENING: Cell<bool> = const { Cell::new(false) };
}

/// Runs `call` on the main thread and waits for it. Clears a thrown Java exception.
fn on_main<T: Send + 'static>(
    call: impl FnOnce(&mut JNIEnv, &JObject) -> JniResult<T> + Send + 'static,
) -> Option<T> {
    let (reply, answer) = std_mpsc::channel();
    dispatch(move |env, activity, _webview| {
        let result = call(env, activity);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_describe();
            let _ = env.exception_clear();
        }
        if let Err(error) = &result {
            debug!("libero notification JNI call failed: {error}");
        }
        let _ = reply.send(result.ok());
    });
    answer.recv_timeout(Duration::from_secs(5)).ok().flatten()
}

/// Off the dioxus thread, which `on_main` would block while the main thread waits on it.
fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> oneshot::Receiver<T> {
    let (reply, answer) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = reply.send(work());
    });
    answer
}

fn sdk(env: &mut JNIEnv) -> JniResult<i32> {
    env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")?
        .i()
}

fn manager<'a>(env: &mut JNIEnv<'a>, activity: &JObject) -> JniResult<JObject<'a>> {
    let name = env.new_string("notification")?;
    env.call_method(
        activity,
        "getSystemService",
        "(Ljava/lang/String;)Ljava/lang/Object;",
        &[JValue::Object(&name)],
    )?
    .l()
}

fn permitted(env: &mut JNIEnv, activity: &JObject) -> JniResult<bool> {
    let name = env.new_string(POST_NOTIFICATIONS)?;
    let state = env
        .call_method(
            activity,
            "checkSelfPermission",
            "(Ljava/lang/String;)I",
            &[JValue::Object(&name)],
        )?
        .i()?;
    Ok(state == 0)
}

/// The facts [`permission_of`] reads: SDK level, enabled, runtime permission held.
fn facts(env: &mut JNIEnv, activity: &JObject) -> JniResult<(i32, bool, bool)> {
    let sdk = sdk(env)?;
    let manager = manager(env, activity)?;
    let enabled = env
        .call_method(&manager, "areNotificationsEnabled", "()Z", &[])?
        .z()?;
    let permitted = sdk < 33 || permitted(env, activity)?;
    Ok((sdk, enabled, permitted))
}

/// Below API 33 there is no runtime prompt; off is the user's switch in Settings.
fn permission_of(sdk: i32, enabled: bool, permitted: bool, asked: bool) -> PermissionState {
    match (enabled, permitted) {
        (true, _) => PermissionState::Granted,
        (false, false) if sdk >= 33 && !asked => PermissionState::Prompt,
        _ => PermissionState::Denied,
    }
}

fn read_permission() -> PermissionState {
    on_main(facts)
        .map(|(sdk, enabled, permitted)| {
            permission_of(sdk, enabled, permitted, ASKED.load(Ordering::Relaxed))
        })
        .unwrap_or(PermissionState::Unsupported)
}

/// No Rust callback for the answer: waits for the dialog to take the
/// activity's focus and give it back, then reads the state.
fn ask() -> PermissionState {
    let asked = on_main(|env, activity| {
        let (sdk, enabled, permitted) = facts(env, activity)?;
        if enabled || permitted || sdk < 33 {
            return Ok(false);
        }
        let name = env.new_string(POST_NOTIFICATIONS)?;
        let names = env.new_object_array(1, "java/lang/String", &name)?;
        env.call_method(
            activity,
            "requestPermissions",
            "([Ljava/lang/String;I)V",
            &[JValue::Object(&names), JValue::Int(1348)],
        )?;
        Ok(true)
    });
    if asked == Some(true) {
        let focused = || {
            on_main(|env, activity| env.call_method(activity, "hasWindowFocus", "()Z", &[])?.z())
        };
        let started = Instant::now();
        let mut opened = false;
        while started.elapsed() < DIALOG_ANSWERED {
            let focus = focused().unwrap_or(true);
            if !opened && !focus {
                opened = true;
                debug!("libero notification permission dialog opened");
            }
            if (opened && focus) || (!opened && started.elapsed() > DIALOG_OPENS) {
                break;
            }
            std::thread::sleep(POLL);
        }
        debug!("libero notification permission answered, dialog seen: {opened}");
        ASKED.store(true, Ordering::Relaxed);
    }
    read_permission()
}

struct Content {
    key: u64,
    title: String,
    body: Option<String>,
    tag: Option<String>,
    silent: bool,
}

fn post(env: &mut JNIEnv, activity: &JObject, content: &Content) -> JniResult<()> {
    let sdk = sdk(env)?;
    let manager = manager(env, activity)?;
    let channel = env.new_string(CHANNEL)?;
    let builder = if sdk >= 26 {
        let label = app_label(env, activity)?;
        let created = env.new_object(
            "android/app/NotificationChannel",
            "(Ljava/lang/String;Ljava/lang/CharSequence;I)V",
            &[
                JValue::Object(&channel),
                JValue::Object(&label),
                JValue::Int(3),
            ],
        )?;
        env.call_method(
            &manager,
            "createNotificationChannel",
            "(Landroid/app/NotificationChannel;)V",
            &[JValue::Object(&created)],
        )?;
        env.new_object(
            "android/app/Notification$Builder",
            "(Landroid/content/Context;Ljava/lang/String;)V",
            &[JValue::Object(activity), JValue::Object(&channel)],
        )?
    } else {
        env.new_object(
            "android/app/Notification$Builder",
            "(Landroid/content/Context;)V",
            &[JValue::Object(activity)],
        )?
    };
    let info = env
        .call_method(
            activity,
            "getApplicationInfo",
            "()Landroid/content/pm/ApplicationInfo;",
            &[],
        )?
        .l()?;
    let icon = env.get_field(&info, "icon", "I")?.i()?;
    let builder_class = "Landroid/app/Notification$Builder;";
    env.call_method(
        &builder,
        "setSmallIcon",
        format!("(I){builder_class}"),
        &[JValue::Int(icon)],
    )?;
    let title = env.new_string(&content.title)?;
    env.call_method(
        &builder,
        "setContentTitle",
        format!("(Ljava/lang/CharSequence;){builder_class}"),
        &[JValue::Object(&title)],
    )?;
    if let Some(body) = &content.body {
        let body = env.new_string(body)?;
        env.call_method(
            &builder,
            "setContentText",
            format!("(Ljava/lang/CharSequence;){builder_class}"),
            &[JValue::Object(&body)],
        )?;
    }
    if content.silent && sdk >= 31 {
        env.call_method(
            &builder,
            "setSilent",
            format!("(Z){builder_class}"),
            &[JValue::Bool(1)],
        )?;
    }
    env.call_method(
        &builder,
        "setAutoCancel",
        format!("(Z){builder_class}"),
        &[JValue::Bool(1)],
    )?;
    let tap = tap_intent(env, activity, content.key)?;
    env.call_method(
        &builder,
        "setContentIntent",
        format!("(Landroid/app/PendingIntent;){builder_class}"),
        &[JValue::Object(&tap)],
    )?;
    let notification = env
        .call_method(&builder, "build", "()Landroid/app/Notification;", &[])?
        .l()?;
    let (tag, id) = identity(content.tag.as_deref(), content.key);
    let tag = match tag {
        Some(tag) => env.new_string(tag)?.into(),
        None => JObject::null(),
    };
    env.call_method(
        &manager,
        "notify",
        "(Ljava/lang/String;ILandroid/app/Notification;)V",
        &[
            JValue::Object(&tag),
            JValue::Int(id),
            JValue::Object(&notification),
        ],
    )?;
    Ok(())
}

/// Android's `(tag, id)` pair: a tagged one keeps id 0 so the next same-tag one replaces it.
fn identity(tag: Option<&str>, key: u64) -> (Option<&str>, i32) {
    match tag {
        Some(tag) => (Some(tag), 0),
        None => (None, key as i32),
    }
}

fn app_label<'a>(env: &mut JNIEnv<'a>, activity: &JObject) -> JniResult<JObject<'a>> {
    let info = env
        .call_method(
            activity,
            "getApplicationInfo",
            "()Landroid/content/pm/ApplicationInfo;",
            &[],
        )?
        .l()?;
    let packages = env
        .call_method(
            activity,
            "getPackageManager",
            "()Landroid/content/pm/PackageManager;",
            &[],
        )?
        .l()?;
    env.call_method(
        &info,
        "loadLabel",
        "(Landroid/content/pm/PackageManager;)Ljava/lang/CharSequence;",
        &[JValue::Object(&packages)],
    )?
    .l()
}

/// An explicit `VIEW` intent back to this activity: tao forwards only `VIEW`'s data.
fn tap_intent<'a>(env: &mut JNIEnv<'a>, activity: &JObject, key: u64) -> JniResult<JObject<'a>> {
    let url = env.new_string(format!("{SCHEME}:{key}"))?;
    let uri = env
        .call_static_method(
            "android/net/Uri",
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&url)],
        )?
        .l()?;
    let action = env.new_string("android.intent.action.VIEW")?;
    let intent = env.new_object(
        "android/content/Intent",
        "(Ljava/lang/String;Landroid/net/Uri;)V",
        &[JValue::Object(&action), JValue::Object(&uri)],
    )?;
    let class = env.get_object_class(activity)?;
    env.call_method(
        &intent,
        "setClass",
        "(Landroid/content/Context;Ljava/lang/Class;)Landroid/content/Intent;",
        &[JValue::Object(activity), JValue::Object(&class)],
    )?;
    env.call_static_method(
        "android/app/PendingIntent",
        "getActivity",
        "(Landroid/content/Context;ILandroid/content/Intent;I)Landroid/app/PendingIntent;",
        &[
            JValue::Object(activity),
            JValue::Int(key as i32),
            JValue::Object(&intent),
            JValue::Int(PENDING_FLAGS),
        ],
    )?
    .l()
}

fn cancel(tag: Option<String>, key: u64) {
    dispatch(move |env, activity, _webview| {
        let cancelled = (|| -> JniResult<()> {
            let manager = manager(env, activity)?;
            let (tag, id) = identity(tag.as_deref(), key);
            let tag = match tag {
                Some(tag) => env.new_string(tag)?.into(),
                None => JObject::null(),
            };
            env.call_method(
                &manager,
                "cancel",
                "(Ljava/lang/String;I)V",
                &[JValue::Object(&tag), JValue::Int(id)],
            )?;
            Ok(())
        })();
        if cancelled.is_err() && env.exception_check().unwrap_or(false) {
            let _ = env.exception_clear();
        }
    });
}

/// The key in a tapped notification's URL.
fn tapped(scheme: &str, path: &str) -> Option<u64> {
    (scheme == SCHEME).then(|| path.parse().ok()).flatten()
}

/// Once per window: tao's `Opened` carries the tapped notification's URL.
fn listen_for_taps() {
    if LISTENING.get() {
        return;
    }
    let Some(desktop) = try_consume_context::<DesktopContext>() else {
        return;
    };
    LISTENING.set(true);
    desktop.create_wry_event_handler(|event, _| {
        let Event::Opened { urls } = event else {
            return;
        };
        for url in urls {
            let Some(key) = tapped(url.scheme(), url.path()) else {
                continue;
            };
            debug!("libero notification {key} tapped");
            let mut routes = ROUTES.lock().unwrap_or_else(|poison| poison.into_inner());
            if let Some(events) = routes.get_or_insert_default().remove(&key) {
                let _ = events.unbounded_send(NotificationEvent::Click);
                // Auto-cancelled by the tap.
                let _ = events.unbounded_send(NotificationEvent::Close);
            }
        }
    });
}

pub(super) struct AndroidNotification;

pub(super) static SYSTEM_NOTIFICATION: AndroidNotification = AndroidNotification;

impl SystemNotificationApi for AndroidNotification {
    fn probe(&self) -> Answer<Option<PermissionState>> {
        let answer = off_thread(read_permission);
        Box::pin(async move {
            let state = answer.await.unwrap_or(PermissionState::Unsupported);
            (state != PermissionState::Unsupported).then_some(state)
        })
    }

    fn request(&self) -> Answer<PermissionState> {
        let answer = off_thread(ask);
        Box::pin(async move { answer.await.unwrap_or(PermissionState::Unsupported) })
    }

    /// `icon` is ignored: the app's launcher icon shows. A tap reopens the app.
    fn show(
        &self,
        notification: &SystemNotification,
        events: Box<dyn Fn(NotificationEvent)>,
    ) -> Shown {
        listen_for_taps();
        let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
        let tag = notification.tag.clone();
        let content = Content {
            key,
            title: notification.title.clone(),
            body: notification.body.clone(),
            tag: tag.clone(),
            silent: notification.silent,
        };
        let answer = off_thread(move || {
            if read_permission() != PermissionState::Granted {
                return Err(SystemNotificationError::Denied);
            }
            on_main(move |env, activity| post(env, activity, &content))
                .ok_or(SystemNotificationError::Failed)
        });
        Box::pin(async move {
            answer
                .await
                .unwrap_or(Err(SystemNotificationError::Failed))?;
            let (sender, mut receiver) = mpsc::unbounded();
            ROUTES
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .get_or_insert_default()
                .insert(key, sender);
            let task = spawn(async move {
                while let Some(event) = poll_fn(|cx| Pin::new(&mut receiver).poll_next(cx)).await {
                    events(event);
                }
            });
            Ok(Box::new(AndroidShown { key, tag, task }) as Box<dyn ShownNotification>)
        })
    }
}

struct AndroidShown {
    key: u64,
    tag: Option<String>,
    task: Task,
}

impl ShownNotification for AndroidShown {
    fn close(&self) {
        cancel(self.tag.clone(), self.key);
    }
}

impl Drop for AndroidShown {
    fn drop(&mut self) {
        self.task.cancel();
        if let Ok(mut routes) = ROUTES.lock() {
            routes.get_or_insert_default().remove(&self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_permission_reads_prompt_only_before_the_first_request() {
        assert_eq!(
            permission_of(34, true, true, false),
            PermissionState::Granted
        );
        assert_eq!(
            permission_of(34, false, false, false),
            PermissionState::Prompt
        );
        assert_eq!(
            permission_of(34, false, false, true),
            PermissionState::Denied
        );
        // Allowed, then switched off in Settings.
        assert_eq!(
            permission_of(34, false, true, false),
            PermissionState::Denied
        );
        assert_eq!(
            permission_of(30, false, true, false),
            PermissionState::Denied
        );
    }

    #[test]
    fn a_tag_keeps_one_identity_and_taps_parse_the_key() {
        assert_eq!(identity(Some("build"), 9), (Some("build"), 0));
        assert_eq!(identity(None, 9), (None, 9));
        assert_eq!(tapped(SCHEME, "12"), Some(12));
        assert_eq!(tapped("https", "12"), None);
        assert_eq!(tapped(SCHEME, "x"), None);
    }
}
