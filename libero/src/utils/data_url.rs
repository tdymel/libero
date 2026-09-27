use dioxus::html::FileData;

/// A `data:` URL of `file`, for an `img`, `video` or `audio` `src`: every
/// renderer shows one. `None` when its bytes cannot be read.
///
/// The URL holds the whole file, a third larger than it: fine for a photo or
/// a short clip, costly for a long recording.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::utils::data_url;
/// #[component]
/// fn Preview(photo: dioxus::html::FileData) -> Element {
///     let src = use_resource(move || {
///         let photo = photo.clone();
///         async move { data_url(&photo).await }
///     });
///     rsx! {
///         if let Some(Some(src)) = src() {
///             img { src, alt: "The photo" }
///         }
///     }
/// }
/// ```
pub async fn data_url(file: &FileData) -> Option<String> {
    let bytes = file.read_bytes().await.ok()?;
    Some(bytes_data_url(
        &file.content_type().unwrap_or_default(),
        &bytes,
    ))
}

/// A `data:` URL of `bytes` typed `content_type`.
pub(crate) fn bytes_data_url(content_type: &str, bytes: &[u8]) -> String {
    format!("data:{content_type};base64,{}", encode_base64(bytes))
}

/// Standard, padded base64, as `atob` reads it.
pub(crate) fn encode_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let triple = chunk.iter().enumerate().fold(0u32, |acc, (at, &byte)| {
            acc | u32::from(byte) << (16 - 8 * at)
        });
        for at in 0..4 {
            match at <= chunk.len() {
                true => text.push(ALPHABET[(triple >> (18 - 6 * at) & 0x3f) as usize] as char),
                false => text.push('='),
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::encode_base64;

    #[test]
    fn base64_pads_as_btoa_does() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
        assert_eq!(encode_base64(&[0xff, 0xfe, 0xfd, 0x00]), "//79AA==");
    }
}
