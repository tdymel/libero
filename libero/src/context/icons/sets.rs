//! Whole icon sets, one const constructor per set and style, each behind its crate's feature.
//! A constructor names only its own consts, so an app links just the set it calls.

use super::{IconSet, IconSlot};
#[cfg(feature = "icons-bootstrap")]
use pictogram_icons_bootstrap as bootstrap;
use pictogram_icons_lucide as lucide;
#[cfg(feature = "icons-material")]
use pictogram_icons_material as material;
#[cfg(feature = "icons-phosphor")]
use pictogram_icons_phosphor as phosphor;
#[cfg(feature = "icons-tabler")]
use pictogram_icons_tabler as tabler;

/// `IconSet::<name>()` per style, every slot from `set::<glyph>::<style>`. A slot the set
/// has no glyph for is left out of the table and keeps lucide.
macro_rules! icon_set {
    ($set:ident: [$($style:ident => $name:ident),+ $(,)?] $table:tt) => {
        impl IconSet {
            $(icon_set!(@one $set $style $name $table);)+
        }
    };
    (@one $set:ident $style:ident $name:ident { $($slot:ident => $glyph:ident),* $(,)? }) => {
        #[doc = concat!("Every slot drawn from `pictogram-icons-", stringify!($set), "`, style `", stringify!($style), "`.")]
        #[doc = ""]
        #[doc = "```rust"]
        #[doc = concat!("const ICONS: libero::IconSet = libero::IconSet::", stringify!($name), "();")]
        #[doc = "```"]
        pub const fn $name() -> Self {
            Self::new()$(.with(IconSlot::$slot, $set::$glyph::$style))*
        }
    };
}

// libero's own defaults, for a subtree under a provider of another set.
icon_set!(lucide: [outlined => lucide_outlined] {
    Close => x,
    ChevronDown => chevron_down,
    ChevronUp => chevron_up,
    ChevronLeft => chevron_left,
    ChevronRight => chevron_right,
    ChevronFirst => chevron_first,
    ChevronLast => chevron_last,
    ArrowDown => arrow_down,
    Check => check,
    CheckboxCheck => check,
    CheckboxIndeterminate => minus,
    Plus => plus,
    Minus => minus,
    Eye => eye,
    EyeOff => eye_off,
    Upload => upload,
    EyeDropper => pipette,
    Copy => copy,
    CopyFailed => circle_alert,
    ExternalLink => external_link,
    Person => user,
    Sun => sun,
    Moon => moon,
    SystemScheme => contrast,
    Play => play,
    Pause => pause,
    TextDirectionLtr => pilcrow_right,
    TextDirectionRtl => pilcrow_left,
    Sparkles => sparkles,
    Grip => grip_vertical,
});

#[cfg(feature = "icons-material")]
icon_set!(material: [
    filled => material_filled,
    outlined => material_outlined,
    rounded => material_rounded,
    sharp => material_sharp,
    two_tone => material_two_tone,
] {
    Close => navigation_close,
    ChevronDown => navigation_expand_more,
    ChevronUp => navigation_expand_less,
    ChevronLeft => navigation_chevron_left,
    ChevronRight => navigation_chevron_right,
    ChevronFirst => navigation_first_page,
    ChevronLast => navigation_last_page,
    ArrowDown => navigation_arrow_downward,
    Check => navigation_check,
    CheckboxCheck => navigation_check,
    CheckboxIndeterminate => content_remove,
    Plus => content_add,
    Minus => content_remove,
    Eye => action_visibility,
    EyeOff => action_visibility_off,
    Upload => file_upload,
    EyeDropper => image_colorize,
    Copy => content_content_copy,
    CopyFailed => alert_error_outline,
    ExternalLink => action_open_in_new,
    Person => social_person,
    Sun => device_light_mode,
    Moon => device_dark_mode,
    SystemScheme => image_contrast,
    Play => av_play_arrow,
    Pause => av_pause,
    TextDirectionLtr => editor_format_textdirection_l_to_r,
    TextDirectionRtl => editor_format_textdirection_r_to_l,
    Sparkles => image_auto_awesome,
    Grip => action_drag_indicator,
});

#[cfg(feature = "icons-tabler")]
icon_set!(tabler: [outlined => tabler_outlined] {
    Close => x,
    ChevronDown => chevron_down,
    ChevronUp => chevron_up,
    ChevronLeft => chevron_left,
    ChevronRight => chevron_right,
    ChevronFirst => chevron_left_pipe,
    ChevronLast => chevron_right_pipe,
    ArrowDown => arrow_down,
    Check => check,
    CheckboxCheck => check,
    CheckboxIndeterminate => minus,
    Plus => plus,
    Minus => minus,
    Eye => eye,
    EyeOff => eye_off,
    Upload => upload,
    EyeDropper => color_picker,
    Copy => copy,
    CopyFailed => alert_circle,
    ExternalLink => external_link,
    Person => user,
    Sun => sun,
    Moon => moon,
    SystemScheme => contrast,
    Play => player_play,
    Pause => player_pause,
    TextDirectionLtr => text_direction_ltr,
    TextDirectionRtl => text_direction_rtl,
    Sparkles => sparkles,
    Grip => grip_vertical,
});

// No text direction glyph: the direction toggle keeps lucide's.
#[cfg(feature = "icons-bootstrap")]
icon_set!(bootstrap: [outlined => bootstrap_outlined] {
    Close => x_lg,
    ChevronDown => chevron_down,
    ChevronUp => chevron_up,
    ChevronLeft => chevron_left,
    ChevronRight => chevron_right,
    ChevronFirst => chevron_bar_left,
    ChevronLast => chevron_bar_right,
    ArrowDown => arrow_down,
    Check => check_lg,
    CheckboxCheck => check_lg,
    CheckboxIndeterminate => dash_lg,
    Plus => plus_lg,
    Minus => dash_lg,
    Eye => eye,
    EyeOff => eye_slash,
    Upload => upload,
    EyeDropper => eyedropper,
    Copy => copy,
    CopyFailed => exclamation_circle,
    ExternalLink => box_arrow_up_right,
    Person => person,
    Sun => sun,
    Moon => moon,
    SystemScheme => circle_half,
    Play => play,
    Pause => pause,
    Sparkles => stars,
    Grip => grip_vertical,
});

// No text direction glyph: the direction toggle keeps lucide's.
#[cfg(feature = "icons-phosphor")]
icon_set!(phosphor: [
    regular => phosphor_regular,
    bold => phosphor_bold,
    light => phosphor_light,
    thin => phosphor_thin,
    fill => phosphor_fill,
    duotone => phosphor_duotone,
] {
    Close => x,
    ChevronDown => caret_down,
    ChevronUp => caret_up,
    ChevronLeft => caret_left,
    ChevronRight => caret_right,
    ChevronFirst => caret_double_left,
    ChevronLast => caret_double_right,
    ArrowDown => arrow_down,
    Check => check,
    CheckboxCheck => check,
    CheckboxIndeterminate => minus,
    Plus => plus,
    Minus => minus,
    Eye => eye,
    EyeOff => eye_slash,
    Upload => upload_simple,
    EyeDropper => eyedropper,
    Copy => copy,
    CopyFailed => warning_circle,
    ExternalLink => arrow_square_out,
    Person => user,
    Sun => sun,
    Moon => moon,
    SystemScheme => circle_half,
    Play => play,
    Pause => pause,
    Sparkles => sparkle,
    Grip => dots_six_vertical,
});

#[cfg(test)]
mod tests {
    use super::super::SLOTS;
    use super::*;

    fn empty(set: IconSet) -> Vec<usize> {
        (0..SLOTS).filter(|&slot| set.0[slot].is_none()).collect()
    }

    #[test]
    fn lucide_fills_every_slot() {
        assert!(empty(IconSet::lucide_outlined()).is_empty());
    }

    #[cfg(feature = "icons-material")]
    #[test]
    fn material_fills_every_slot_in_every_style() {
        for set in [
            IconSet::material_filled(),
            IconSet::material_outlined(),
            IconSet::material_rounded(),
            IconSet::material_sharp(),
            IconSet::material_two_tone(),
        ] {
            assert!(empty(set).is_empty());
        }
    }

    #[cfg(feature = "icons-tabler")]
    #[test]
    fn tabler_fills_every_slot() {
        assert!(empty(IconSet::tabler_outlined()).is_empty());
    }

    #[cfg(feature = "icons-bootstrap")]
    #[test]
    fn bootstrap_leaves_only_the_text_direction_to_lucide() {
        let direction = [
            IconSlot::TextDirectionLtr as usize,
            IconSlot::TextDirectionRtl as usize,
        ];
        assert_eq!(empty(IconSet::bootstrap_outlined()), direction);
    }

    #[cfg(feature = "icons-phosphor")]
    #[test]
    fn phosphor_leaves_only_the_text_direction_to_lucide() {
        let direction = [
            IconSlot::TextDirectionLtr as usize,
            IconSlot::TextDirectionRtl as usize,
        ];
        for set in [
            IconSet::phosphor_regular(),
            IconSet::phosphor_bold(),
            IconSet::phosphor_light(),
            IconSet::phosphor_thin(),
            IconSet::phosphor_fill(),
            IconSet::phosphor_duotone(),
        ] {
            assert_eq!(empty(set), direction);
        }
    }
}
