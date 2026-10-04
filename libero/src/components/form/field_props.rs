/// A field's `Props` struct: [`base_props`](crate::components::common::base_props)
/// plus the props every field shares. No `value`, handler or `placeholder`.
///
/// ```ignore
/// # // Not compiled: `field_props!` is crate-internal, so a doc-test cannot name it.
/// field_props! {
///     extends(input);
///     pub struct TextFieldProps {
///         #[props(default, into)]
///         value: Option<String>,
///     }
/// }
/// ```
///
/// `without(readonly);` (not with `extends`) and `without(radius);` drop those
/// props. Separate dispatch arms: an optional `extends` is a macro ambiguity.
///
/// The `parts` prop takes [`FieldPart`]; a leading `parts(TextareaPart);` swaps
/// in a field's own enum, declared by [`field_parts_enum!`].
macro_rules! field_props {
    (@build [$part:ty] [$($extra_extends:tt)*] [$($readonly:ident)?] [$($radius:ident)?]
        $(#[$struct_meta:meta])*
        $vis:vis struct $name:ident $(< $($generic:ident : $bound:path),+ $(,)? >)? {
            $($fields:tt)*
        }
    ) => {
        crate::components::common::base_props! {
            @build [$($extra_extends)*] [$part]
            $(#[$struct_meta])*
            $vis struct $name $(< $($generic: $bound),+ >)? {
                $($fields)*
                /// The field's caption, above the control.
                #[props(default, into)]
                label: crate::components::form::Caption,
                /// Between the label and the control. What to enter.
                #[props(default, into)]
                description: crate::components::form::Caption,
                /// Under the control. Formatting rules, constraints, counters.
                #[props(default, into)]
                helper: crate::components::form::Caption,
                /// Validation state, under the helper. A bare `&str` is an error.
                #[props(default, into)]
                status: Input<crate::components::form::FieldStatus>,
                #[props(default, into)]
                size: Input<crate::theme::Size>,
                $(
                    /// Corner radius, independent of `size`.
                    #[props(default, into)]
                    $radius: Input<crate::theme::Size>,
                )?
                /// `None` is "not stated", so a `Fieldset` can cascade into it.
                #[props(default)]
                disabled: Option<bool>,
                /// Marks the field required: the asterisk, and the state for assistive technology.
                #[props(default)]
                required: Option<bool>,
                $(
                    /// Focusable and posted, but not editable. `None` is "not stated".
                    #[props(default)]
                    $readonly: Option<bool>,
                )?
            }
        }
    };

    (@with [$part:ty] extends($($extra_extends:ident),+ $(,)?); without(radius); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [$part] [$(, extends = $extra_extends)+] [readonly] [] $($rest)*);
    };

    (@with [$part:ty] extends($($extra_extends:ident),+ $(,)?); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [$part] [$(, extends = $extra_extends)+] [readonly] [radius] $($rest)*);
    };

    (@with [$part:ty] without(readonly); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [$part] [] [] [radius] $($rest)*);
    };

    (@with [$part:ty] without(radius); $($rest:tt)*) => {
        crate::components::form::field_props!(@build [$part] [] [readonly] [] $($rest)*);
    };

    (@with [$part:ty] $($rest:tt)*) => {
        crate::components::form::field_props!(@build [$part] [] [readonly] [radius] $($rest)*);
    };

    (parts($part:ty); $($rest:tt)*) => {
        crate::components::form::field_props!(@with [$part] $($rest)*);
    };

    ($($rest:tt)*) => {
        crate::components::form::field_props!(@with [crate::components::form::FieldPart] $($rest)*);
    };
}

pub(crate) use field_props;

/// Declares a field's part enum: the shared caption parts, after `framed` the
/// frame's too, then the field's own, as `parts_enum!` takes them.
///
/// ```ignore
/// # // Not compiled: `field_parts_enum!` is crate-internal.
/// field_parts_enum! {
///     pub enum TextareaPart framed {
///         Counter = "counter" => "& > [data-slot='frame'] > [data-slot='counter']",
///     }
/// }
/// ```
///
/// The field's own variants end with a comma.
macro_rules! field_parts_enum {
    (@build [$($frame:tt)*] $(#[$meta:meta])* $vis:vis enum $name:ident { $($own:tt)* }) => {
        crate::components::common::parts_enum! {
            $(#[$meta])*
            $vis enum $name {
                /// The label above the control.
                Label = "label" => "& > [data-slot='label']",
                /// The required asterisk, in the label.
                Required = "required" => "& > [data-slot='label'] > [data-slot='required']",
                /// The caption between the label and the control.
                Description = "description" => "& > [data-slot='description']",
                $($frame)*
                $($own)*
                /// The caption under the control.
                Helper = "helper" => "& > [data-slot='helper']",
                /// The validation message.
                Status = "status" => "& > [data-slot='status']",
            }
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident framed { $($own:tt)* }
    ) => {
        crate::components::form::field_parts_enum! {
            // Each frame part is matched one level deeper too: a list field's
            // frame sits in its dropdown's anchor, and Blitz wraps the control
            // in a placeholder cell.
            @build [
                /// The bordered box around the control.
                Frame = "frame" => "& > [data-slot='frame'], & > * > [data-slot='frame']",
                /// The slot before the control: an icon, a prefix.
                Leading = "leading" => "& > [data-slot='frame'] > [data-slot='leading'], & > * > [data-slot='frame'] > [data-slot='leading']",
                /// The element the label names.
                Control = "control" => "& > [data-slot='frame'] > [data-slot='control'], & > [data-slot='frame'] > * > [data-slot='control'], & > * > [data-slot='frame'] > [data-slot='control'], & > * > [data-slot='frame'] > * > [data-slot='control']",
                /// The slot after the control: a chevron, a toggle.
                Trailing = "trailing" => "& > [data-slot='frame'] > [data-slot='trailing'], & > * > [data-slot='frame'] > [data-slot='trailing']",
            ]
            $(#[$meta])* $vis enum $name { $($own)* }
        }
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident { $($own:tt)* }
    ) => {
        crate::components::form::field_parts_enum! {
            @build [] $(#[$meta])* $vis enum $name { $($own)* }
        }
    };
}

pub(crate) use field_parts_enum;

field_parts_enum! {
    /// A framed field's inner parts, for its `parts` prop. Each is matched from
    /// the field's root by child selectors, so a field nested in a slot keeps its own.
    pub enum FieldPart framed {}
}

#[cfg(test)]
mod tests {
    use crate::components::{
        common::Part,
        form::{
            CascaderPart, CheckboxPart, ChipPart, FieldPart, FieldsetPart, FileFieldPart,
            PhoneFieldPart, RadioGroupPart, RadioPart, RatingPart, SegmentedControlPart,
            SelectPart, SliderPart, SwitchPart, TagsFieldPart, TextareaPart,
        },
    };

    fn table<P: Part>() -> Vec<&'static str> {
        P::ALL.iter().map(|part| part.slot()).collect()
    }

    /// Every alternative of a selector starts at the root and ends on its slot.
    fn assert_well_formed<P: Part + std::fmt::Debug>() {
        for part in P::ALL {
            let target = format!("[data-slot='{}']", part.slot());
            for alternative in part.selector().split(", ") {
                assert!(alternative.starts_with("& "), "{part:?}: {alternative}");
                assert!(alternative.ends_with(&target), "{part:?}: {alternative}");
            }
        }
    }

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_tables_are_stable() {
        let caption = |own: &[&'static str]| {
            let mut slots = vec!["label", "required", "description"];
            slots.extend(own);
            slots.extend(["helper", "status"]);
            slots
        };
        let framed = |own: &[&'static str]| {
            let mut slots = vec!["frame", "leading", "control", "trailing"];
            slots.extend(own);
            caption(&slots)
        };

        assert_eq!(table::<FieldPart>(), framed(&[]));
        assert_eq!(table::<TextareaPart>(), framed(&["counter"]));
        assert_eq!(table::<SelectPart>(), framed(&["value", "chip"]));
        assert_eq!(table::<CascaderPart>(), framed(&["value"]));
        assert_eq!(table::<PhoneFieldPart>(), framed(&["country", "dial"]));
        assert_eq!(table::<TagsFieldPart>(), framed(&["tag"]));
        assert_eq!(
            table::<FileFieldPart>(),
            caption(&["frame", "control", "browse", "chip", "trailing", "card"])
        );
        assert_eq!(table::<CheckboxPart>(), caption(&["control", "box"]));
        assert_eq!(table::<RadioPart>(), caption(&["control", "circle", "dot"]));
        assert_eq!(
            table::<RadioGroupPart>(),
            caption(&["control", "circle", "dot"])
        );
        assert_eq!(
            table::<SwitchPart>(),
            caption(&["control", "track", "thumb"])
        );
        assert_eq!(
            table::<SliderPart>(),
            caption(&[
                "control",
                "track",
                "bar",
                "bars",
                "segments",
                "segment",
                "segment-fill",
                "mark",
                "mark-label",
                "thumb"
            ])
        );
        assert_eq!(
            table::<RatingPart>(),
            caption(&["control", "symbol", "glyph", "fill"])
        );
        assert_eq!(
            table::<SegmentedControlPart>(),
            caption(&["control", "segment"])
        );
        assert_eq!(
            table::<ChipPart>(),
            ["chip-icon", "chip-trailing", "new-tab"]
        );
        assert_eq!(
            table::<FieldsetPart>(),
            ["legend", "description", "helper", "status"]
        );
    }

    #[test]
    fn every_selector_targets_its_own_slot() {
        assert_well_formed::<FieldPart>();
        assert_well_formed::<TextareaPart>();
        assert_well_formed::<SelectPart>();
        assert_well_formed::<CascaderPart>();
        assert_well_formed::<PhoneFieldPart>();
        assert_well_formed::<TagsFieldPart>();
        assert_well_formed::<FileFieldPart>();
        assert_well_formed::<CheckboxPart>();
        assert_well_formed::<RadioPart>();
        assert_well_formed::<RadioGroupPart>();
        assert_well_formed::<SwitchPart>();
        assert_well_formed::<SliderPart>();
        assert_well_formed::<RatingPart>();
        assert_well_formed::<SegmentedControlPart>();
        assert_well_formed::<ChipPart>();
        assert_well_formed::<FieldsetPart>();
    }

    /// The frame parts' selectors are the shared field's, so `FieldPart`
    /// styles recast onto a `Select` (`NativeSelect`'s fallback) still match.
    #[test]
    fn shared_parts_keep_one_selector_across_enums() {
        let shared = |part: FieldPart| part.selector();
        assert_eq!(SelectPart::Frame.selector(), shared(FieldPart::Frame));
        assert_eq!(SelectPart::Control.selector(), shared(FieldPart::Control));
        assert_eq!(TextareaPart::Label.selector(), shared(FieldPart::Label));
        assert_eq!(SliderPart::Status.selector(), shared(FieldPart::Status));
    }
}
