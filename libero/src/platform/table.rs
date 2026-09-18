use super::backend;

/// Whether a `<table>` lays out its `<caption>`. Blitz skips it, so `Table`
/// draws the caption before the table there instead.
pub(crate) fn lays_out_captions() -> bool {
    backend::LAYS_OUT_CAPTIONS
}
