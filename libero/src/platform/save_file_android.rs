//! The Android WebView saves no Blob download: bytes go to the system share
//! sheet over JNI on the main thread, small text inline (todo 2016), the rest as a
//! file in Downloads (2078).

use dioxus_desktop::wry::prelude::{
    JNIEnv, JObject, dispatch,
    jni::{self, errors::Result as JniResult, objects::JValue},
};
use futures_channel::oneshot;

use super::{SaveFileApi, SaveOutcome, Saving};

const ACTION_SEND: &str = "android.intent.action.SEND";
const EXTRA_TEXT: &str = "android.intent.extra.TEXT";
const EXTRA_STREAM: &str = "android.intent.extra.STREAM";
const EXTRA_SUBJECT: &str = "android.intent.extra.SUBJECT";
const FLAG_GRANT_READ_URI_PERMISSION: i32 = 1;
/// Android 10, which added `MediaStore.Downloads`.
const MIN_DOWNLOADS_SDK: i32 = 29;
/// Under the Binder's 1 MB transaction cap, which an intent's extras travel through.
const MAX_TEXT: usize = 500_000;

pub(super) struct AndroidSaveFile;

pub(super) static SAVE_FILE: AndroidSaveFile = AndroidSaveFile;

/// What goes to the share sheet: small text inline, anything else as a file.
enum Payload {
    Text(String),
    File { mime: String, bytes: Vec<u8> },
}

impl Payload {
    fn new(mime: &str, bytes: Vec<u8>) -> Self {
        match String::from_utf8(bytes) {
            Ok(text) if text.len() <= MAX_TEXT => Self::Text(text),
            Ok(text) => Self::file(mime, text.into_bytes()),
            Err(error) => Self::file(mime, error.into_bytes()),
        }
    }

    fn file(mime: &str, bytes: Vec<u8>) -> Self {
        let mime = match mime.is_empty() {
            true => "application/octet-stream".to_string(),
            false => mime.to_string(),
        };
        Self::File { mime, bytes }
    }
}

impl SaveFileApi for AndroidSaveFile {
    fn save(&self, name: &str, mime: &str, bytes: Vec<u8>) -> Saving {
        let payload = Payload::new(mime, bytes);
        let name = name.to_string();
        let (reply, answer) = oneshot::channel();
        dispatch(move |env, activity, _webview| {
            let shared = match &payload {
                Payload::Text(text) => {
                    share(env, activity, &name, text).map_err(|error| error.to_string())
                }
                Payload::File { mime, bytes } => share_file(env, activity, &name, mime, bytes),
            };
            if env.exception_check().unwrap_or(false) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }
            let _ = reply.send(shared);
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

/// `ACTION_SEND` as `text/plain`, so messengers and notes take it too, in a chooser titled `name`.
fn share(env: &mut JNIEnv, activity: &JObject, name: &str, text: &str) -> JniResult<()> {
    let intent = send_intent(env, "text/plain")?;
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
    choose(env, activity, &intent, name)
}

/// Writes `bytes` to Downloads through MediaStore (no permission, no FileProvider
/// in the app's manifest), then shares that `content://` URI as `EXTRA_STREAM`.
fn share_file(
    env: &mut JNIEnv,
    activity: &JObject,
    name: &str,
    mime: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let sdk = env
        .get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
        .and_then(|sdk| sdk.i())
        .map_err(|error| error.to_string())?;
    if sdk < MIN_DOWNLOADS_SDK {
        return Err("files are shared from Android 10 on".to_string());
    }
    let uri = download(env, activity, name, mime, bytes).map_err(|error| error.to_string())?;
    let intent = send_intent(env, mime).map_err(|error| error.to_string())?;
    stream(env, &intent, &uri, name)
        .and_then(|()| choose(env, activity, &intent, name))
        .map_err(|error| error.to_string())
}

fn stream(env: &mut JNIEnv, intent: &JObject, uri: &JObject, name: &str) -> JniResult<()> {
    let key = env.new_string(EXTRA_STREAM)?;
    env.call_method(
        intent,
        "putExtra",
        "(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;",
        &[JValue::Object(&key), JValue::Object(uri)],
    )?;
    let key = env.new_string(EXTRA_SUBJECT)?;
    let subject = env.new_string(name)?;
    env.call_method(
        intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&key), JValue::Object(&subject)],
    )?;
    env.call_method(
        intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(FLAG_GRANT_READ_URI_PERMISSION)],
    )?;
    Ok(())
}

/// A new `MediaStore.Downloads` entry holding `bytes`; a failed write removes it.
fn download<'a>(
    env: &mut JNIEnv<'a>,
    activity: &JObject,
    name: &str,
    mime: &str,
    bytes: &[u8],
) -> JniResult<JObject<'a>> {
    let values = env.new_object("android/content/ContentValues", "()V", &[])?;
    for (key, value) in [("_display_name", name), ("mime_type", mime)] {
        let key = env.new_string(key)?;
        let value = env.new_string(value)?;
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[JValue::Object(&key), JValue::Object(&value)],
        )?;
    }
    let collection = env
        .get_static_field(
            "android/provider/MediaStore$Downloads",
            "EXTERNAL_CONTENT_URI",
            "Landroid/net/Uri;",
        )?
        .l()?;
    let resolver = env
        .call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )?
        .l()?;
    let uri = env
        .call_method(
            &resolver,
            "insert",
            "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
            &[JValue::Object(&collection), JValue::Object(&values)],
        )?
        .l()?;
    if uri.is_null() {
        return Err(jni::errors::Error::NullPtr(
            "MediaStore refused the download",
        ));
    }
    if let Err(error) = write(env, &resolver, &uri, bytes) {
        let _ = env.exception_clear();
        let _ = env.call_method(
            &resolver,
            "delete",
            "(Landroid/net/Uri;Ljava/lang/String;[Ljava/lang/String;)I",
            &[
                JValue::Object(&uri),
                JValue::Object(&JObject::null()),
                JValue::Object(&JObject::null()),
            ],
        );
        return Err(error);
    }
    Ok(uri)
}

fn write(env: &mut JNIEnv, resolver: &JObject, uri: &JObject, bytes: &[u8]) -> JniResult<()> {
    let out = env
        .call_method(
            resolver,
            "openOutputStream",
            "(Landroid/net/Uri;)Ljava/io/OutputStream;",
            &[JValue::Object(uri)],
        )?
        .l()?;
    let array = env.byte_array_from_slice(bytes)?;
    env.call_method(&out, "write", "([B)V", &[JValue::Object(&array)])?;
    env.call_method(&out, "close", "()V", &[])?;
    Ok(())
}

fn send_intent<'a>(env: &mut JNIEnv<'a>, mime: &str) -> JniResult<JObject<'a>> {
    let action = env.new_string(ACTION_SEND)?;
    let intent = env.new_object(
        "android/content/Intent",
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action)],
    )?;
    let mime = env.new_string(mime)?;
    env.call_method(
        &intent,
        "setType",
        "(Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&mime)],
    )?;
    Ok(intent)
}

/// The system chooser over `intent`, titled `name`.
fn choose(env: &mut JNIEnv, activity: &JObject, intent: &JObject, name: &str) -> JniResult<()> {
    let title = env.new_string(name)?;
    let chooser = env
        .call_static_method(
            "android/content/Intent",
            "createChooser",
            "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
            &[JValue::Object(intent), JValue::Object(&title)],
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
