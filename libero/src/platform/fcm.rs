//! Firebase Cloud Messaging on Android (todo 3404), behind the `fcm` feature: the
//! Gradle module `android/fcm` holds the service; its events reach Rust over JNI.

use std::collections::BTreeMap;

/// The Firebase app's values, as its `google-services.json` lists them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FcmOptions {
    /// `mobilesdk_app_id`, e.g. `1:123:android:abc`.
    pub application_id: String,
    /// `current_key`.
    pub api_key: String,
    /// `project_id`.
    pub project_id: String,
    /// `project_number`.
    pub sender_id: String,
}

/// A message the app received while running.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FcmMessage {
    /// The message's data payload.
    pub data: BTreeMap<String, String>,
    /// The notification payload's title, if it had one.
    pub title: Option<String>,
    /// The notification payload's body, if it had one.
    pub body: Option<String>,
}

/// Why no token came.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FcmError {
    /// Not an Android WebView app.
    Unsupported,
    /// Firebase refused, with its reason.
    Failed(String),
}

/// What Kotlin reported so far, kept for hooks mounted later.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FcmState {
    pub(crate) token: Option<String>,
    pub(crate) message: Option<FcmMessage>,
    pub(crate) error: Option<String>,
    /// Counts token and error answers, so a register call knows it was answered.
    pub(crate) answers: u64,
}

#[cfg(all(target_os = "android", not(feature = "native")))]
mod android {
    use std::sync::Mutex;

    use dioxus_desktop::wry::prelude::{
        JNIEnv,
        jni::objects::{JClass, JObjectArray, JString},
    };
    use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};

    use super::super::android_share::{app_object, on_main_thread};
    use super::{FcmMessage, FcmOptions, FcmState};

    #[manganis::ffi("android/fcm")]
    unsafe extern "Kotlin" {
        pub type LiberoFcm;
        pub fn register(
            this: &LiberoFcm,
            application_id: String,
            api_key: String,
            project_id: String,
            sender_id: String,
        );
        pub fn flush(this: &LiberoFcm);
        pub fn probe(this: &LiberoFcm, token: String, data: String);
    }

    struct Hub {
        state: FcmState,
        listeners: Vec<UnboundedSender<()>>,
    }

    static HUB: Mutex<Hub> = Mutex::new(Hub {
        state: FcmState {
            token: None,
            message: None,
            error: None,
            answers: 0,
        },
        listeners: Vec::new(),
    });

    fn publish(change: impl FnOnce(&mut FcmState)) {
        let mut hub = HUB.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        change(&mut hub.state);
        hub.listeners
            .retain(|listener| listener.unbounded_send(()).is_ok());
    }

    pub(crate) fn state() -> FcmState {
        let hub = HUB.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        hub.state.clone()
    }

    /// Wakes on each change of [`state`].
    pub(crate) struct Changes(UnboundedReceiver<()>);

    impl Changes {
        /// `false` once no change can come.
        pub(crate) async fn next(&mut self) -> bool {
            use futures_core::Stream;
            std::future::poll_fn(|cx| std::pin::Pin::new(&mut self.0).poll_next(cx))
                .await
                .is_some()
        }
    }

    pub(crate) fn watch() -> Changes {
        let (sender, receiver) = unbounded();
        let mut hub = HUB.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        hub.listeners.push(sender);
        Changes(receiver)
    }

    /// Runs `call` on the bridge on the main thread. Not `LiberoFcm::new`: its `FindClass`
    /// sees only the boot classes on the main and the dioxus thread (it aborted the app).
    fn on_bridge(
        call: impl FnOnce(&LiberoFcm) -> Result<(), String> + Send + 'static,
    ) -> impl Future<Output = Result<(), String>> {
        on_main_thread(move |env, activity| {
            let bridge = app_object(env, activity, "dev.libero.fcm.LiberoFcm")
                .map(LiberoFcm::from_global_ref)
                .map_err(|error| error.to_string())?;
            call(&bridge)
        })
    }

    pub(crate) fn register_app(options: FcmOptions) -> impl Future<Output = Result<(), String>> {
        on_bridge(move |bridge| {
            register(
                bridge,
                options.application_id,
                options.api_key,
                options.project_id,
                options.sender_id,
            )
            .map_err(|error| error.to_string())
        })
    }

    pub(crate) fn flush_events() {
        drop(on_bridge(|bridge| {
            flush(bridge).map_err(|error| error.to_string())
        }));
    }

    #[cfg(debug_assertions)]
    pub(crate) fn run_probe(token: String, data: String) {
        drop(on_bridge(move |bridge| {
            probe(bridge, token, data).map_err(|error| error.to_string())
        }));
    }

    fn text(env: &mut JNIEnv, value: &JString) -> Option<String> {
        if value.is_null() {
            return None;
        }
        env.get_string(value).ok().map(Into::into)
    }

    #[unsafe(no_mangle)]
    extern "system" fn Java_dev_libero_fcm_LiberoFcmEvents_onToken(
        mut env: JNIEnv,
        _class: JClass,
        token: JString,
    ) {
        let token = text(&mut env, &token);
        publish(|state| {
            state.token = token;
            state.error = None;
            state.answers += 1;
        });
    }

    #[unsafe(no_mangle)]
    extern "system" fn Java_dev_libero_fcm_LiberoFcmEvents_onError(
        mut env: JNIEnv,
        _class: JClass,
        error: JString,
    ) {
        let error = text(&mut env, &error);
        publish(|state| {
            state.error = error;
            state.answers += 1;
        });
    }

    #[unsafe(no_mangle)]
    extern "system" fn Java_dev_libero_fcm_LiberoFcmEvents_onMessage(
        mut env: JNIEnv,
        _class: JClass,
        pairs: JObjectArray,
        title: JString,
        body: JString,
    ) {
        let length = env.get_array_length(&pairs).unwrap_or(0);
        let mut fields = Vec::with_capacity(length as usize);
        for index in 0..length {
            let field = env
                .get_object_array_element(&pairs, index)
                .map(JString::from)
                .ok()
                .and_then(|field| text(&mut env, &field));
            fields.push(field.unwrap_or_default());
        }
        let message = FcmMessage {
            data: fields
                .as_chunks::<2>()
                .0
                .iter()
                .map(|[key, value]| (key.clone(), value.clone()))
                .collect(),
            title: text(&mut env, &title),
            body: text(&mut env, &body),
        };
        publish(|state| state.message = Some(message));
    }
}

#[cfg(all(target_os = "android", not(feature = "native")))]
pub(crate) use android::{flush_events, register_app, state, watch};

/// For e2e only, in debug builds: a token and a message with `data` under `probe`,
/// down the service's own path.
#[cfg(debug_assertions)]
pub fn fcm_probe(token: &str, data: &str) {
    #[cfg(all(target_os = "android", not(feature = "native")))]
    android::run_probe(token.into(), data.into());
    #[cfg(not(all(target_os = "android", not(feature = "native"))))]
    let _ = (token, data);
}
