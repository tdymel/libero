// `file-field` alone compiles only the geometry its `crop` prop names.
#[cfg_attr(not(feature = "image-cropper"), allow(dead_code))]
mod crop;
#[cfg(feature = "image-cropper")]
mod image_cropper;
#[cfg(all(test, feature = "image-cropper"))]
mod tests;

pub use crop::{CropOptions, CropRect, CropShape, PixelRect};
#[cfg(feature = "image-cropper")]
pub use image_cropper::{ImageCropper, ImageCropperPart, ImageCropperProps};
