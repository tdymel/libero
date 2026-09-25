use crate::{
    css::CssDeclaration,
    theme::{
        ANCHOR_CODE_COLOR, CODE_BLOCK_BACKGROUND, CODE_BLOCK_COPY_HOVER_BACKGROUND,
        CODE_BLOCK_COPY_HOVER_TEXT, CODE_BLOCK_LINE_NUMBER, CODE_BLOCK_MUTED_TEXT,
        CODE_TOK_ATTRIBUTE, CODE_TOK_COMMENT, CODE_TOK_CONSTANT, CODE_TOK_FUNCTION,
        CODE_TOK_HEADING, CODE_TOK_KEYWORD, CODE_TOK_NUMBER, CODE_TOK_STRING, CODE_TOK_TAG,
        CODE_TOK_TYPE, Color, ColorShade, ColorValue, CssVar, HexColor, KBD_BACKGROUND, KBD_COLOR,
    },
    tokens::{Ends, TEXT_CONTRAST},
};

/// Code and `Kbd` text sits on `muted` steps that follow the page, the token hues don't:
/// each text var that falls short takes the smallest readable mix (todo 396).
pub(super) fn rebase_text_on_derived_surfaces(declarations: &mut [CssDeclaration], ends: Ends) {
    let block = CODE_BLOCK_BACKGROUND.name().to_string();
    // Inline `Code`'s `muted.2` background, which `sx` resolves as a fill.
    let inline = ColorValue::Fill(Color::Muted, ColorShade::S2).var_name();
    let code = [block.clone(), inline];
    let kbd = [KBD_BACKGROUND.name().to_string()];
    let hover = [CODE_BLOCK_COPY_HOVER_BACKGROUND.name().to_string()];

    let texts: [(CssVar, &[String], f32); 15] = [
        (ANCHOR_CODE_COLOR, &code[1..], TEXT_CONTRAST),
        (CODE_TOK_KEYWORD, &code, TEXT_CONTRAST),
        (CODE_TOK_STRING, &code, TEXT_CONTRAST),
        (CODE_TOK_COMMENT, &code, TEXT_CONTRAST),
        (CODE_TOK_NUMBER, &code, TEXT_CONTRAST),
        (CODE_TOK_CONSTANT, &code, TEXT_CONTRAST),
        (CODE_TOK_FUNCTION, &code, TEXT_CONTRAST),
        (CODE_TOK_TYPE, &code, TEXT_CONTRAST),
        (CODE_TOK_TAG, &code, TEXT_CONTRAST),
        (CODE_TOK_ATTRIBUTE, &code, TEXT_CONTRAST),
        (CODE_TOK_HEADING, &code, TEXT_CONTRAST),
        (CODE_BLOCK_MUTED_TEXT, &code[..1], TEXT_CONTRAST),
        // 241's approved 4.27:1; the 3:1 floor let palettes drift below it (899).
        (CODE_BLOCK_LINE_NUMBER, &code[..1], 4.27),
        (KBD_COLOR, &kbd, TEXT_CONTRAST),
        // The copy button's icon, a graphic: 1.4.11.
        (CODE_BLOCK_COPY_HOVER_TEXT, &hover, 3.0),
    ];
    for (text, surfaces, floor) in texts {
        rebase_text(declarations, text.name(), surfaces, floor, ends);
    }
}

/// Leaves `name` alone when it or a surface can't be measured (a keyword, a translucent `rgba()`).
fn rebase_text(
    declarations: &mut [CssDeclaration],
    name: &str,
    surfaces: &[String],
    floor: f32,
    ends: Ends,
) {
    let resolve = |name: &str| {
        let mut value = name.to_string();
        for _ in 0..declarations.len() {
            let declared = declarations.iter().find(|d| d.property() == value)?.value();
            match CssVar::parse_value(declared) {
                Some(next) => value = next.to_string(),
                None => return HexColor::parse(declared),
            }
        }
        None
    };
    let Some(text) = resolve(name) else { return };
    let Some(surfaces) = surfaces
        .iter()
        .map(|surface| resolve(surface))
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let readable = text.readable_on(&surfaces, floor, ends);
    // Unchanged, so a var that already reads stays a var.
    if readable == text {
        return;
    }
    if let Some(declaration) = declarations.iter_mut().find(|d| d.property() == name) {
        *declaration = CssDeclaration::new(name, readable.to_string());
    }
}
