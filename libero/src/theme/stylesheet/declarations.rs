use super::{palette::push_palette_declarations, rebase::rebase_text_on_derived_surfaces};
use crate::{
    css::{CssDeclaration, ToCssDeclarations},
    theme::{Color, HexColor, SizeCss, Theme},
    tokens::{Ends, Sizes},
};

pub(super) fn theme_declarations(theme: &Theme) -> Vec<CssDeclaration> {
    // Exhaustive (no `..`): a new `Theme` field won't compile until it's handled.
    let Theme {
        spacing,
        radius,
        elevation,
        font_size,
        // Measured against the palette, so declared with it below.
        gradient: _,
        dialog,
        drawer,
        sidebar,
        flex,
        grid,
        carousel,
        scroller,
        center,
        container,
        aspect_ratio,
        collapse,
        transition,
        accordion,
        float,
        overlay,
        z_index,
        popover,
        progress_bar,
        focus_ring,
        paper,
        divider,
        splitter,
        blockquote,
        burger,
        button,
        chip,
        badge,
        alert,
        switch,
        checkbox,
        radio,
        field,
        form,
        fieldset,
        combobox,
        slider,
        rating,
        list,
        data_list,
        table,
        timeline,
        tree,
        tabs,
        stepper,
        title,
        text,
        tooltip,
        code,
        code_block,
        header,
        icon,
        action_icon,
        qr_code,
        image,
        image_list,
        lightbox,
        avatar,
        avatar_group,
        kbd,
        menu,
        menubar,
        pagination,
        // Chrome only - read by the component, never a var.
        theme_switcher: _,
        repository: _,
        direction_toggle: _,
        tldr: _,
        anchor,
        file_field,
        pin_field,
        color_picker,
        color_swatch,
        chrono_picker,
        primary,
        secondary,
        error,
        warning,
        info,
        success,
        neutral,
        muted,
        ink,
        surface,
        // Plain values read from Rust - no CSS vars of their own.
        floating_window: _,
        phone_field: _,
        tags_field: _,
        text_field: _,
        textarea: _,
        number_field: _,
        chrono_field: _,
        time_picker: _,
        native_select: _,
        select: _,
        multi_select: _,
        cascader: _,
        autocomplete: _,
        segmented_control: _,
        password_field: _,
        color_field: _,
        scroll_area: _,
        mark: _,
        hover_card: _,
        tour: _,
        nav_link: _,
        bottom_navigation: _,
        loader,
        indicator,
        skeleton,
        spotlight,
        marquee,
        notifications,
        // Emitted by global_reset_scopes, not as a `:root` var.
        font_smoothing: _,
    } = theme;

    let mut declarations = Vec::new();
    push_scale_declarations(&mut declarations, spacing, radius, elevation, font_size);
    push_component_declarations(
        &mut declarations,
        &[
            dialog,
            drawer,
            sidebar,
            flex,
            grid,
            carousel,
            scroller,
            center,
            container,
            aspect_ratio,
            collapse,
            transition,
            accordion,
            float,
            overlay,
            z_index,
            popover,
            progress_bar,
            focus_ring,
            paper,
            divider,
            splitter,
            blockquote,
            burger,
            button,
            chip,
            badge,
            alert,
            loader,
            indicator,
            skeleton,
            spotlight,
            marquee,
            notifications,
            lightbox,
            switch,
            checkbox,
            radio,
            field,
            form,
            fieldset,
            file_field,
            pin_field,
            color_picker,
            color_swatch,
            chrono_picker,
            combobox,
            slider,
            rating,
            list,
            data_list,
            table,
            timeline,
            tree,
            tabs,
            stepper,
            title,
            text,
            tooltip,
            code,
            code_block,
            header,
            icon,
            action_icon,
            qr_code,
            kbd,
            menu,
            menubar,
            pagination,
            image,
            image_list,
            avatar,
            avatar_group,
            anchor,
        ],
    );
    let ends = Ends {
        surface: *surface,
        ink: *ink,
    };
    // A `Paper` that is not a hex (a `var()`, a gradient) cannot be measured.
    let cards: Vec<HexColor> = HexColor::parse(paper.background).into_iter().collect();
    push_palette_declarations(
        &mut declarations,
        ends,
        &cards,
        [
            (Color::Primary, *primary),
            (Color::Secondary, *secondary),
            (Color::Error, *error),
            (Color::Warning, *warning),
            (Color::Info, *info),
            (Color::Success, *success),
            (Color::Neutral, *neutral),
            (Color::Muted, *muted),
        ],
    );
    declarations.extend(crate::theme::gradient_theme_declarations(theme));
    rebase_text_on_derived_surfaces(&mut declarations, ends);
    declarations
}

/// The size scales, plus the breakpoints.
fn push_scale_declarations(
    declarations: &mut Vec<CssDeclaration>,
    spacing: &Sizes<u8>,
    radius: &Sizes<u8>,
    elevation: &Sizes<&'static str>,
    font_size: &Sizes<&'static str>,
) {
    declarations.extend(spacing.to_css_declarations(SizeCss::SPACING, "px"));
    declarations.extend(radius.to_css_declarations(SizeCss::RADIUS, "px"));
    declarations.extend(elevation.to_css_declarations(SizeCss::SHADOW, ""));
    declarations.extend(font_size.to_css_declarations(SizeCss::FONT_SIZE, ""));
    push_breakpoint_declarations(declarations);
}

fn push_component_declarations(
    declarations: &mut Vec<CssDeclaration>,
    components: &[&dyn ToCssDeclarations],
) {
    for component in components {
        declarations.extend(component.to_css_declarations());
    }
}

// Literals, not a themed scale: `@media` can't read custom properties.
fn push_breakpoint_declarations(declarations: &mut Vec<CssDeclaration>) {
    for size in crate::theme::Size::ALL {
        declarations.push(SizeCss::BREAKPOINT.declare(size, size.breakpoint_value()));
    }
}
