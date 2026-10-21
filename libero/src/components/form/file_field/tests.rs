use super::file_field::crop_unavailable;
use super::rows::chip_cursor;

/// Todo 3917: `file-field` builds without `image-cropper`, and then says why nothing crops.
#[test]
fn crop_without_the_image_cropper_feature_warns() {
    assert!(crop_unavailable(true, false).is_some_and(|note| note.contains("`image-cropper`")));
    assert_eq!(crop_unavailable(true, true), None);
    assert_eq!(crop_unavailable(false, false), None);
}

#[test]
fn the_cursor_clamps_to_the_chips_left() {
    assert_eq!(chip_cursor(Some(1), 3, false), Some(1));
    assert_eq!(chip_cursor(Some(4), 3, false), Some(2));
    assert_eq!(chip_cursor(Some(0), 0, false), None);
    assert_eq!(chip_cursor(None, 3, false), None);
}

/// Todo 245: a cursor left over from the `Input` variant named a chip id
/// the dropzone never draws, through `aria-activedescendant`.
#[test]
fn a_dropzone_has_no_cursor_whatever_is_left_over() {
    assert_eq!(chip_cursor(Some(1), 3, true), None);
}
