use super::backend;

/// Whether a `<table>` lays out its `<caption>`. Blitz skips it, so `Table`
/// draws the caption before the table there instead.
pub(crate) fn lays_out_captions() -> bool {
    backend::LAYS_OUT_CAPTIONS
}

/// Whether a `width: 100%` table still widens to its content's minimum. Blitz
/// holds it and its cells overflow, so `Table` takes `min-width: 100%` there.
pub(crate) fn widens_sized_tables() -> bool {
    backend::WIDENS_SIZED_TABLES
}

/// Whether a sticky `<thead>` sticks. Blitz's doesn't, so a `Table` with
/// column groups sticks only its last header row there.
pub(crate) fn sticks_table_heads() -> bool {
    backend::STICKS_TABLE_HEADS
}

/// Whether a `transform` on a `<tr>` paints. Blitz's `tr` has no box, so a
/// `Table`'s rows reorder there by key and buttons only, not by drag.
pub(crate) fn moves_table_rows() -> bool {
    backend::MOVES_TABLE_ROWS
}

/// Whether an absolute box in a padded box of text takes presses where it is
/// drawn. Blitz shifts them by the padding, so `Table` wraps a header's text there.
pub(crate) fn hits_absolute_in_text() -> bool {
    backend::HITS_ABSOLUTE_IN_TEXT
}

/// Whether a `Table` header has a column drag grip. Not on Blitz, where a
/// header grip drag is unreliable: the column menu moves columns there.
pub(crate) fn drags_table_columns() -> bool {
    backend::DRAGS_TABLE_COLUMNS
}
