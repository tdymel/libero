//! Android's system share sheet over JNI on the main thread, for `share` (3351)
//! and `save_file` (2016): text inline, `share`'s files through libero's own content
//! provider (3408), `save_file`'s through Downloads (2078).

use dioxus_desktop::wry::prelude::{
    JNIEnv, JObject, dispatch,
    jni::{
        self,
        errors::Result as JniResult,
        objects::{GlobalRef, JClass, JString, JValue},
    },
};
use futures_channel::oneshot;
use std::fs::File;
use std::io::Write;
use std::os::fd::{FromRawFd, RawFd};
use std::sync::Arc;

use super::share::{ShareApi, ShareData, ShareFile, ShareOutcome, Sharing, text_fields};

const ACTION_SEND: &str = "android.intent.action.SEND";
const ACTION_SEND_MULTIPLE: &str = "android.intent.action.SEND_MULTIPLE";
const EXTRA_TEXT: &str = "android.intent.extra.TEXT";
const EXTRA_STREAM: &str = "android.intent.extra.STREAM";
const EXTRA_SUBJECT: &str = "android.intent.extra.SUBJECT";
const FLAG_GRANT_READ_URI_PERMISSION: i32 = 1;
/// Android 10, which added `MediaStore.Downloads`.
const MIN_DOWNLOADS_SDK: i32 = 29;
const OCTET_STREAM: &str = "application/octet-stream";

// The module's provider serves staged files by name: the chooser cannot read a
// MediaStore row's name (3392). Its manifest merges into every Android app.
#[manganis::ffi("android/share")]
unsafe extern "Kotlin" {
    pub type LiberoShare;
    pub fn stage_path(this: &LiberoShare, name: String) -> String;
    pub fn content_uri(this: &LiberoShare, path: String) -> String;
}

pub(super) struct AndroidShare;

pub(super) static SHARE: AndroidShare = AndroidShare;

impl ShareApi for AndroidShare {
    fn share(&self, data: ShareData) -> Sharing {
        let data = Arc::new(data);
        let named = on_main_thread({
            let data = Arc::clone(&data);
            move |env, activity| stage(env, activity, &data.files)
        });
        Box::pin(async move {
            match open(data, named).await {
                Ok(outcome) => outcome,
                Err(error) => ShareOutcome::Failed(error),
            }
        })
    }
}

/// A file's staged path and the `content://` URI the provider serves it at.
type Staged = (String, String);

/// Writes the files off the main thread between its two jobs: naming the paths and
/// opening the chooser.
async fn open(
    data: Arc<ShareData>,
    named: impl Future<Output = Result<Vec<Staged>, String>>,
) -> Result<ShareOutcome, String> {
    let named = named.await?;
    let uris = write_files(Arc::clone(&data), named).await?;
    on_main_thread(move |env, activity| send(env, activity, &data, &uris)).await
}

/// Writes `data`'s files to their staged paths on a worker thread; answers their URIs.
async fn write_files(data: Arc<ShareData>, named: Vec<Staged>) -> Result<Vec<String>, String> {
    if named.is_empty() {
        return Ok(Vec::new());
    }
    let (reply, answer) = oneshot::channel();
    std::thread::spawn(move || {
        let written = data
            .files
            .iter()
            .zip(named)
            .map(|(file, (path, uri))| {
                std::fs::write(path, &file.bytes)
                    .map(|()| uri)
                    .map_err(|error| error.to_string())
            })
            .collect();
        let _ = reply.send(written);
    });
    answer
        .await
        .unwrap_or_else(|_| Err("the share files were not written".to_string()))
}

/// Runs `job` on the main thread with the activity and answers with its result;
/// a pending Java exception is logged and cleared.
pub(super) fn on_main_thread<T: Send + 'static>(
    job: impl FnOnce(&mut JNIEnv, &JObject) -> Result<T, String> + Send + 'static,
) -> impl Future<Output = Result<T, String>> {
    let (reply, answer) = oneshot::channel();
    dispatch(move |env, activity, _webview| {
        let done = job(env, activity);
        if env.exception_check().unwrap_or(false) {
            let _ = env.exception_describe();
            let _ = env.exception_clear();
        }
        let _ = reply.send(done);
    });
    async move {
        answer
            .await
            .unwrap_or_else(|_| Err("the share sheet did not open".to_string()))
    }
}

/// An instance of the app's `class` (dotted), built with the activity. Through the
/// activity's class loader: `#[ffi]`'s own `FindClass` sees only the boot classes here.
pub(super) fn app_object(
    env: &mut JNIEnv,
    activity: &JObject,
    class: &str,
) -> JniResult<GlobalRef> {
    let loader = env
        .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?
        .l()?;
    let name = env.new_string(class)?;
    let class = env
        .call_method(
            &loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[JValue::Object(&name)],
        )?
        .l()?;
    let object = env.new_object(
        JClass::from(class),
        "(Landroid/app/Activity;)V",
        &[JValue::Object(activity)],
    )?;
    env.new_global_ref(object)
}

/// Names where the provider serves each file and its `content://` URI; writes nothing.
fn stage(env: &mut JNIEnv, activity: &JObject, files: &[ShareFile]) -> Result<Vec<Staged>, String> {
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let bridge = app_object(env, activity, "dev.libero.share.LiberoShare")
        .map(LiberoShare::from_global_ref)
        .map_err(|error| error.to_string())?;
    files
        .iter()
        .map(|file| {
            let path = stage_path(&bridge, file.name.clone()).map_err(|error| error.to_string())?;
            let uri = content_uri(&bridge, path.clone()).map_err(|error| error.to_string())?;
            Ok((path, uri))
        })
        .collect()
}

/// One chooser for the whole of `data`: the staged `uris` as `EXTRA_STREAM`, the
/// text and url joined into `EXTRA_TEXT`, the title as subject.
fn send(
    env: &mut JNIEnv,
    activity: &JObject,
    data: &ShareData,
    uris: &[String],
) -> Result<ShareOutcome, String> {
    let [title, text, url] = text_fields(data);
    let message = [text, url]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
    // A title alone is all the text there is.
    let message = match message.is_empty() && uris.is_empty() {
        true => title.unwrap_or_default().to_string(),
        false => message,
    };
    if message.is_empty() && uris.is_empty() {
        return Ok(ShareOutcome::Unsupported);
    }
    let jni = |error: jni::errors::Error| error.to_string();
    let files = data.files.as_slice();
    let intent = match uris {
        [] => send_intent(env, ACTION_SEND, "text/plain").map_err(jni)?,
        [uri] => {
            let uri = parse_uri(env, uri).map_err(jni)?;
            let intent = send_intent(env, ACTION_SEND, mime(&files[0])).map_err(jni)?;
            stream(env, &intent, &uri).map_err(jni)?;
            intent
        }
        staged => {
            let list = env
                .new_object("java/util/ArrayList", "()V", &[])
                .map_err(jni)?;
            let mut uris = Vec::with_capacity(staged.len());
            for uri in staged {
                let uri = parse_uri(env, uri).map_err(jni)?;
                env.call_method(
                    &list,
                    "add",
                    "(Ljava/lang/Object;)Z",
                    &[JValue::Object(&uri)],
                )
                .map_err(jni)?;
                uris.push(uri);
            }
            let first = mime(&files[0]);
            let mime = match files.iter().all(|file| mime(file) == first) {
                true => first,
                false => "*/*",
            };
            let intent = send_intent(env, ACTION_SEND_MULTIPLE, mime).map_err(jni)?;
            streams(env, &intent, &list, &uris).map_err(jni)?;
            intent
        }
    };
    if !message.is_empty() {
        put(env, &intent, EXTRA_TEXT, &message).map_err(jni)?;
    }
    if let Some(title) = title {
        put(env, &intent, EXTRA_SUBJECT, title).map_err(jni)?;
    }
    choose(env, activity, &intent, title).map_err(jni)?;
    Ok(ShareOutcome::Shared)
}

/// `ACTION_SEND` as `text/plain`, so messengers and notes take it too, in a chooser titled `name`.
pub(super) fn share_text(
    env: &mut JNIEnv,
    activity: &JObject,
    name: &str,
    text: &str,
) -> JniResult<()> {
    let intent = send_intent(env, ACTION_SEND, "text/plain")?;
    put(env, &intent, EXTRA_TEXT, text)?;
    put(env, &intent, EXTRA_SUBJECT, name)?;
    choose(env, activity, &intent, Some(name))
}

/// A new Downloads entry: its `content://` URI and a descriptor to write its bytes through.
struct Download {
    uri: String,
    fd: RawFd,
}

/// Writes `file` to Downloads through MediaStore (no permission, no FileProvider
/// in the app's manifest), then shares that `content://` URI as `EXTRA_STREAM`.
/// The bytes go in on a worker thread, between two main-thread jobs.
pub(super) async fn share_file(file: ShareFile) -> Result<(), String> {
    let file = Arc::new(file);
    let download = on_main_thread({
        let file = Arc::clone(&file);
        move |env, activity| create_download(env, activity, &file)
    })
    .await?;
    let uri = download.uri.clone();
    let written = write_download(download.fd, Arc::clone(&file)).await;
    on_main_thread(move |env, activity| finish_download(env, activity, &file, &uri, written)).await
}

/// Writes `file`'s bytes through the descriptor on a worker thread, which closes it.
async fn write_download(fd: RawFd, file: Arc<ShareFile>) -> Result<(), String> {
    let (reply, answer) = oneshot::channel();
    std::thread::spawn(move || {
        // SAFETY: `detachFd` handed the descriptor over; this thread alone owns it.
        let mut out = unsafe { File::from_raw_fd(fd) };
        let _ = reply.send(
            out.write_all(&file.bytes)
                .map_err(|error| error.to_string()),
        );
    });
    answer
        .await
        .unwrap_or_else(|_| Err("the download was not written".to_string()))
}

/// Shares the written download, or removes its entry when `written` failed.
fn finish_download(
    env: &mut JNIEnv,
    activity: &JObject,
    file: &ShareFile,
    uri: &str,
    written: Result<(), String>,
) -> Result<(), String> {
    let jni = |error: jni::errors::Error| error.to_string();
    let uri = parse_uri(env, uri).map_err(jni)?;
    if let Err(error) = written {
        let resolver = resolver(env, activity).map_err(jni)?;
        let _ = delete(env, &resolver, &uri);
        return Err(error);
    }
    let intent = send_intent(env, ACTION_SEND, mime(file)).map_err(jni)?;
    stream(env, &intent, &uri)
        .and_then(|()| put(env, &intent, EXTRA_SUBJECT, &file.name))
        .and_then(|()| choose(env, activity, &intent, Some(&file.name)))
        .map_err(jni)
}

fn sdk(env: &mut JNIEnv) -> Result<i32, String> {
    env.get_static_field("android/os/Build$VERSION", "SDK_INT", "I")
        .and_then(|sdk| sdk.i())
        .map_err(|error| error.to_string())
}

fn mime(file: &ShareFile) -> &str {
    match file.mime.is_empty() {
        true => OCTET_STREAM,
        false => &file.mime,
    }
}

fn put(env: &mut JNIEnv, intent: &JObject, key: &str, value: &str) -> JniResult<()> {
    let key = env.new_string(key)?;
    let value = env.new_string(value)?;
    env.call_method(
        intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&key), JValue::Object(&value)],
    )?;
    Ok(())
}

fn stream(env: &mut JNIEnv, intent: &JObject, uri: &JObject) -> JniResult<()> {
    let key = env.new_string(EXTRA_STREAM)?;
    env.call_method(
        intent,
        "putExtra",
        "(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;",
        &[JValue::Object(&key), JValue::Object(uri)],
    )?;
    grant_read(env, intent, std::slice::from_ref(uri))
}

fn streams(env: &mut JNIEnv, intent: &JObject, list: &JObject, uris: &[JObject]) -> JniResult<()> {
    let key = env.new_string(EXTRA_STREAM)?;
    env.call_method(
        intent,
        "putParcelableArrayListExtra",
        "(Ljava/lang/String;Ljava/util/ArrayList;)Landroid/content/Intent;",
        &[JValue::Object(&key), JValue::Object(list)],
    )?;
    grant_read(env, intent, uris)
}

/// The URIs also go in as `ClipData`: `createChooser` hands that and the grant on
/// to the chooser, which otherwise cannot query the file names for its preview (3392).
fn grant_read(env: &mut JNIEnv, intent: &JObject, uris: &[JObject]) -> JniResult<()> {
    if let [first, rest @ ..] = uris {
        let label = env.new_string("")?;
        let clip = env
            .call_static_method(
                "android/content/ClipData",
                "newRawUri",
                "(Ljava/lang/CharSequence;Landroid/net/Uri;)Landroid/content/ClipData;",
                &[JValue::Object(&label), JValue::Object(first)],
            )?
            .l()?;
        for uri in rest {
            let item = env.new_object(
                "android/content/ClipData$Item",
                "(Landroid/net/Uri;)V",
                &[JValue::Object(uri)],
            )?;
            env.call_method(
                &clip,
                "addItem",
                "(Landroid/content/ClipData$Item;)V",
                &[JValue::Object(&item)],
            )?;
        }
        env.call_method(
            intent,
            "setClipData",
            "(Landroid/content/ClipData;)V",
            &[JValue::Object(&clip)],
        )?;
    }
    env.call_method(
        intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(FLAG_GRANT_READ_URI_PERMISSION)],
    )?;
    Ok(())
}

fn parse_uri<'a>(env: &mut JNIEnv<'a>, uri: &str) -> JniResult<JObject<'a>> {
    let uri = env.new_string(uri)?;
    env.call_static_method(
        "android/net/Uri",
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&uri)],
    )?
    .l()
}

fn resolver<'a>(env: &mut JNIEnv<'a>, activity: &JObject) -> JniResult<JObject<'a>> {
    env.call_method(
        activity,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[],
    )?
    .l()
}

/// A new, empty `MediaStore.Downloads` entry named for `file` and a descriptor opened to
/// write it; a failure removes the entry.
fn create_download(
    env: &mut JNIEnv,
    activity: &JObject,
    file: &ShareFile,
) -> Result<Download, String> {
    if sdk(env)? < MIN_DOWNLOADS_SDK {
        return Err("files are shared from Android 10 on".to_string());
    }
    let jni = |error: jni::errors::Error| error.to_string();
    let resolver = resolver(env, activity).map_err(jni)?;
    let uri = insert_download(env, &resolver, file).map_err(jni)?;
    match open_descriptor(env, &resolver, &uri) {
        Ok(fd) => {
            let uri = to_string(env, &uri).map_err(jni)?;
            Ok(Download { uri, fd })
        }
        Err(error) => {
            let _ = env.exception_clear();
            let _ = delete(env, &resolver, &uri);
            Err(jni(error))
        }
    }
}

fn to_string(env: &mut JNIEnv, object: &JObject) -> JniResult<String> {
    let text = env
        .call_method(object, "toString", "()Ljava/lang/String;", &[])?
        .l()?;
    Ok(env.get_string(&JString::from(text))?.into())
}

/// A descriptor to write `uri`'s entry through, which the caller owns and closes.
fn open_descriptor(env: &mut JNIEnv, resolver: &JObject, uri: &JObject) -> JniResult<RawFd> {
    let mode = env.new_string("w")?;
    let pfd = env
        .call_method(
            resolver,
            "openFileDescriptor",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
            &[JValue::Object(uri), JValue::Object(&mode)],
        )?
        .l()?;
    if pfd.is_null() {
        return Err(jni::errors::Error::NullPtr("MediaStore gave no descriptor"));
    }
    env.call_method(&pfd, "detachFd", "()I", &[])?.i()
}

fn delete(env: &mut JNIEnv, resolver: &JObject, uri: &JObject) -> JniResult<()> {
    env.call_method(
        resolver,
        "delete",
        "(Landroid/net/Uri;Ljava/lang/String;[Ljava/lang/String;)I",
        &[
            JValue::Object(uri),
            JValue::Object(&JObject::null()),
            JValue::Object(&JObject::null()),
        ],
    )?;
    Ok(())
}

fn insert_download<'a>(
    env: &mut JNIEnv<'a>,
    resolver: &JObject,
    file: &ShareFile,
) -> JniResult<JObject<'a>> {
    let values = env.new_object("android/content/ContentValues", "()V", &[])?;
    for (key, value) in [
        ("_display_name", file.name.as_str()),
        ("mime_type", mime(file)),
    ] {
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
    let uri = env
        .call_method(
            resolver,
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
    Ok(uri)
}

fn send_intent<'a>(env: &mut JNIEnv<'a>, action: &str, mime: &str) -> JniResult<JObject<'a>> {
    // The e2e share scenarios read this line from logcat.
    #[cfg(debug_assertions)]
    dioxus::prelude::debug!("libero share intent: {action} {mime}");
    let action = env.new_string(action)?;
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

/// The system chooser over `intent`, titled `title` or the system's default.
fn choose(
    env: &mut JNIEnv,
    activity: &JObject,
    intent: &JObject,
    title: Option<&str>,
) -> JniResult<()> {
    let title = match title {
        Some(title) => env.new_string(title)?.into(),
        None => JObject::null(),
    };
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
