use super::backend;

/// Whether a `<table>` lays out its `<caption>`. Blitz skips it, so `Table`
/// draws the caption before the table there instead.
pub(crate) fn lays_out_captions() -> bool {
    backend::LAYS_OUT_CAPTIONS
}

/// Whether a `width: 100%` table still widens to its content's minimum. Blitz
/// holds it at the width and its cells overflow where no scroller reaches
/// them, so `Table` takes `min-width: 100%` there.
pub(crate) fn widens_sized_tables() -> bool {
    backend::WIDENS_SIZED_TABLES
}
