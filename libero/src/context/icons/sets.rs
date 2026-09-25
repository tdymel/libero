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
    Star => star,
    Grip => grip_vertical,
    MoveTo => arrow_right_left,
    Volume => volume_2,
    VolumeOff => volume_x,
    Fullscreen => maximize,
    ExitFullscreen => minimize,
    Captions => captions,
    Bold => bold,
    Italic => italic,
    Underline => underline,
    Strikethrough => strikethrough,
    InlineCode => code,
    BulletList => list,
    OrderedList => list_ordered,
    Quote => text_quote,
    CodeBlock => square_code,
    Undo => undo_2,
    Redo => redo_2,
    Link => link,
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
    Star => toggle_star,
    Grip => action_drag_indicator,
    MoveTo => action_swap_horiz,
    Volume => av_volume_up,
    VolumeOff => av_volume_off,
    Fullscreen => navigation_fullscreen,
    ExitFullscreen => navigation_fullscreen_exit,
    Captions => av_closed_caption,
    Bold => editor_format_bold,
    Italic => editor_format_italic,
    Underline => editor_format_underlined,
    Strikethrough => editor_format_strikethrough,
    InlineCode => action_code,
    BulletList => editor_format_list_bulleted,
    OrderedList => editor_format_list_numbered,
    Quote => editor_format_quote,
    CodeBlock => editor_data_object,
    Undo => content_undo,
    Redo => content_redo,
    Link => content_link,
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
    Star => star,
    Grip => grip_vertical,
    MoveTo => arrows_left_right,
    Volume => volume,
    VolumeOff => volume_3,
    Fullscreen => maximize,
    ExitFullscreen => minimize,
    Captions => badge_cc,
    Bold => bold,
    Italic => italic,
    Underline => underline,
    Strikethrough => strikethrough,
    InlineCode => code,
    BulletList => list,
    OrderedList => list_numbers,
    Quote => blockquote,
    CodeBlock => source_code,
    Undo => arrow_back_up,
    Redo => arrow_forward_up,
    Link => link,
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
    Star => star,
    Grip => grip_vertical,
    MoveTo => arrow_left_right,
    Volume => volume_up,
    VolumeOff => volume_mute,
    Fullscreen => fullscreen,
    ExitFullscreen => fullscreen_exit,
    Captions => cc_square,
    Bold => type_bold,
    Italic => type_italic,
    Underline => type_underline,
    Strikethrough => type_strikethrough,
    InlineCode => code,
    BulletList => list_ul,
    OrderedList => list_ol,
    Quote => blockquote_left,
    CodeBlock => code_square,
    Undo => arrow_counterclockwise,
    Redo => arrow_clockwise,
    Link => link_45deg,
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
    Star => star,
    Grip => dots_six_vertical,
    MoveTo => arrows_left_right,
    Volume => speaker_high,
    VolumeOff => speaker_x,
    Fullscreen => corners_out,
    ExitFullscreen => corners_in,
    Captions => closed_captioning,
    Bold => text_b,
    Italic => text_italic,
    Underline => text_underline,
    Strikethrough => text_strikethrough,
    InlineCode => code,
    BulletList => list_bullets,
    OrderedList => list_numbers,
    Quote => quotes,
    CodeBlock => code_block,
    Undo => arrow_u_up_left,
    Redo => arrow_u_up_right,
    Link => link,
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
