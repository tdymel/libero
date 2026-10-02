//! The Android WebView saves no Blob download: text goes to the system share
//! sheet over JNI on the main thread (todo 2016).

use dioxus_desktop::wry::prelude::{
    JNIEnv, JObject, dispatch,
    jni::{errors::Result as JniResult, objects::JValue},
};
use futures_channel::oneshot;

use super::{SaveFileApi, SaveOutcome, Saving};

const ACTION_SEND: &str = "android.intent.action.SEND";
const EXTRA_TEXT: &str = "android.intent.extra.TEXT";
const EXTRA_SUBJECT: &str = "android.intent.extra.SUBJECT";
/// Under the Binder's 1 MB transaction cap, which an intent's extras travel through.
const MAX_TEXT: usize = 500_000;

pub(super) struct AndroidSaveFile;

pub(super) static SAVE_FILE: AndroidSaveFile = AndroidSaveFile;

impl SaveFileApi for AndroidSaveFile {
    fn save(&self, name: &str, _mime: &str, bytes: Vec<u8>) -> Saving {
        let Ok(text) = String::from_utf8(bytes) else {
            return failed("only text can be shared on Android");
        };
        if text.len() > MAX_TEXT {
            return failed("too large to share on Android");
        }
        let name = name.to_string();
        let (reply, answer) = oneshot::channel();
        dispatch(move |env, activity, _webview| {
            let shared = share(env, activity, &name, &text);
            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }
            let _ = reply.send(shared.map_err(|error| error.to_string()));
        });
        Box::pin(async move {
            match answer.await {
                Ok(Ok(())) => SaveOutcome::Shared,
                Ok(Err(error)) => SaveOutcome::Failed(error),
                Err(_) => SaveOutcome::Failed("the share sheet did not open".to_string()),
            }
        })
    }
}

fn failed(reason: &str) -> Saving {
    Box::pin(std::future::ready(SaveOutcome::Failed(reason.to_string())))
}

/// `ACTION_SEND` as `text/plain`, so messengers and notes take it too, in a chooser titled `name`.
fn share(env: &mut JNIEnv, activity: &JObject, name: &str, text: &str) -> JniResult<()> {
    let action = env.new_string(ACTION_SEND)?;
    let intent = env.new_object(
        "android/content/Intent",
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action)],
    )?;
    let mime = env.new_string("text/plain")?;
    env.call_method(
        &intent,
        "setType",
        "(Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&mime)],
    )?;
    for (key, value) in [(EXTRA_TEXT, text), (EXTRA_SUBJECT, name)] {
        let key = env.new_string(key)?;
        let value = env.new_string(value)?;
        env.call_method(
            &intent,
            "putExtra",
            "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&key), JValue::Object(&value)],
        )?;
    }
    let title = env.new_string(name)?;
    let chooser = env
        .call_static_method(
            "android/content/Intent",
            "createChooser",
            "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
            &[JValue::Object(&intent), JValue::Object(&title)],
        )?
        .l()?;
    env.call_method(
        activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&chooser)],
    )?;
    Ok(())
}
