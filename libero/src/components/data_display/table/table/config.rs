//! The props that seed or control the table's state, handed to [`use_table`](super::super::use_table::use_table).

use std::mem::take;

use super::super::use_table::TableConfig;
use super::TableProps;

/// Takes the state props out of `props`, which reads none of them afterwards.
pub(super) fn table_config<T: Clone + PartialEq + 'static>(
    props: &mut TableProps<T>,
) -> TableConfig {
    TableConfig {
        sort: take(&mut props.sort),
        default_sort: take(&mut props.default_sort),
        onsortchange: props.onsortchange,
        selection: take(&mut props.selection),
        default_selection: take(&mut props.default_selection),
        onselectionchange: props.onselectionchange,
        page: props.page,
        default_page: props.default_page,
        onpagechange: props.onpagechange,
        page_size: props.page_size,
        default_page_size: props
            .default_page_size
            .or(props.page_sizes.first().copied())
            .unwrap_or(10),
        onpagesizechange: props.onpagesizechange,
        hidden_columns: take(&mut props.hidden_columns),
        default_hidden_columns: take(&mut props.default_hidden_columns),
        onhiddencolumnschange: props.onhiddencolumnschange,
        quick_filter: take(&mut props.quick_filter),
        default_quick_filter: take(&mut props.default_quick_filter),
        onquickfilterchange: props.onquickfilterchange,
        column_filters: take(&mut props.column_filters),
        default_column_filters: take(&mut props.default_column_filters),
        oncolumnfilterschange: props.oncolumnfilterschange,
        filter_logic: props.filter_logic,
        default_filter_logic: props.default_filter_logic,
        onfilterlogicchange: props.onfilterlogicchange,
        pinned_columns: take(&mut props.pinned_columns),
        default_pinned_columns: take(&mut props.default_pinned_columns),
        onpinnedcolumnschange: props.onpinnedcolumnschange,
        expanded: take(&mut props.expanded),
        default_expanded: take(&mut props.default_expanded),
        onexpandedchange: props.onexpandedchange,
        column_order: take(&mut props.column_order),
        default_column_order: take(&mut props.default_column_order),
        oncolumnorderchange: props.oncolumnorderchange,
        column_widths: take(&mut props.column_widths),
        default_column_widths: take(&mut props.default_column_widths),
        oncolumnwidthschange: props.oncolumnwidthschange,
        density: props.density,
        default_density: props.default_density,
        ondensitychange: props.ondensitychange,
        row_grouping: take(&mut props.row_grouping),
        default_row_grouping: take(&mut props.default_row_grouping),
        onrowgroupingchange: props.onrowgroupingchange,
        expanded_groups: take(&mut props.expanded_groups),
        default_expanded_groups: take(&mut props.default_expanded_groups),
        onexpandedgroupschange: props.onexpandedgroupschange,
        expanded_nodes: take(&mut props.expanded_nodes),
        default_expanded_nodes: take(&mut props.default_expanded_nodes),
        onexpandednodeschange: props.onexpandednodeschange,
    }
}
