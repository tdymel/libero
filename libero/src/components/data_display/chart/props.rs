use crate::{
    components::common::{input_from_str, parts_enum},
    theme::{ChartTable, Curve, LegendPosition},
};

input_from_str!(Curve);
input_from_str!(LegendPosition);
input_from_str!(ChartTable);

parts_enum! {
    /// A chart's inner parts, for its `parts` prop.
    pub enum ChartPart {
        Title = "title" => "& > [data-slot='title']",
        Description = "description" => "& > [data-slot='description']",
        /// The plot and its legend, side by side or stacked.
        Body = "body" => "& > [data-slot='body']",
        Plot = "plot" => "& > [data-slot='body'] > [data-slot='plot']",
        Legend = "legend" => "& > [data-slot='body'] > [data-slot='legend']",
        LegendItem = "legend-item" => "& > [data-slot='body'] > [data-slot='legend'] > [data-slot='legend-item']",
        /// What the plot shows without data: `empty`, or the localized "No data".
        Empty = "empty" => "& > [data-slot='body'] > [data-slot='plot'] > [data-slot='empty']",
        /// The data table's scroll box, hidden unless `table: "visible"`.
        Table = "table" => "& > [data-slot='table']",
    }
}

/// Declares a chart's `Props`: [`base_props!`](crate::components::common::base_props)
/// with `parts(ChartPart)` and the props every chart shares, after its own.
macro_rules! chart_props {
    (
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident {
            $($fields:tt)*
        }
    ) => {
        crate::components::common::base_props! {
            parts(super::ChartPart);
            $(#[$struct_meta])*
            $vis struct $name {
                $($fields)*
                /// Shown above the chart, and its accessible name.
                #[props(default, into)]
                title: Option<String>,
                /// A sentence under the title: what to read from the chart.
                #[props(default, into)]
                description: Option<String>,
                /// Unset, at the bottom when there is more than one series.
                #[props(default, into)]
                legend: Input<crate::theme::LegendPosition>,
                /// Horizontal guide lines. Unset, the theme's.
                #[props(default)]
                grid: Option<bool>,
                /// How values are written on the axis, in the summary and in the table.
                #[props(default)]
                value_format: super::ValueFormat,
                /// After every value, as `"kg"`.
                #[props(default, into)]
                unit: Option<String>,
                /// Width over height of the plot. Unset, the theme's.
                #[props(default)]
                aspect_ratio: Option<f32>,
                /// A placeholder instead of the plot, and `aria-busy` on the chart.
                #[props(default)]
                loading: bool,
                /// Shown in the plot when there is no value at all.
                #[props(default)]
                empty: Option<Element>,
                /// The data as a table: hidden for screen readers by default.
                #[props(default, into)]
                table: Input<crate::theme::ChartTable>,
            }
        }
    };
}

pub(super) use chart_props;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<ChartPart>(),
            [
                ("title", "& > [data-slot='title']"),
                ("description", "& > [data-slot='description']"),
                ("body", "& > [data-slot='body']"),
                ("plot", "& > [data-slot='body'] > [data-slot='plot']"),
                ("legend", "& > [data-slot='body'] > [data-slot='legend']"),
                (
                    "legend-item",
                    "& > [data-slot='body'] > [data-slot='legend'] > [data-slot='legend-item']"
                ),
                (
                    "empty",
                    "& > [data-slot='body'] > [data-slot='plot'] > [data-slot='empty']"
                ),
                ("table", "& > [data-slot='table']"),
            ]
        );
    }
}
