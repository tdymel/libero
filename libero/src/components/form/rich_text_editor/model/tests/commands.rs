use super::{show, state};
use crate::components::form::rich_text_editor::model::{
    Action, Builtin, Chord, Commands, CustomContent, Doc, Editor, KeyPress, Keymap, NodeRegistry,
    NodeSpec, Record, RegistryError,
};
use crate::hooks::UndoHistory;

fn editor(markdown: &str) -> Editor {
    let state = state(markdown);
    Editor::with_history(UndoHistory::new(state))
}

fn press(chord: &str) -> KeyPress {
    let chord = Chord::parse(chord).unwrap();
    KeyPress {
        key: chord.key.clone(),
        code: String::new(),
        ctrl: chord.ctrl || chord.primary,
        meta: chord.meta,
        alt: chord.alt,
        shift: chord.shift,
    }
}

#[test]
fn every_builtin_has_a_command() {
    let commands = Commands::builtin();
    for builtin in Builtin::ALL {
        assert!(commands.get(&(*builtin).into()).is_some(), "{builtin:?}");
    }
}

#[test]
fn every_default_chord_runs_a_known_command() {
    let commands = Commands::builtin();
    for (chord, name) in Keymap::default().bindings() {
        assert!(commands.get(name).is_some(), "{chord} runs unknown {name}");
    }
}

#[test]
fn a_symbol_chord_matches_with_the_shift_its_layout_needs() {
    let keymap = Keymap::default();
    let german_slash = KeyPress {
        ctrl: true,
        shift: true,
        code: "Digit7".into(),
        ..KeyPress::new("/")
    };
    assert_eq!(
        keymap.command_for(&german_slash, false),
        Some(&Builtin::Shortcuts.into())
    );
    // A letter keeps strict modifiers: Mod+Shift+b is quote, not bold.
    assert_eq!(
        keymap.command_for(&press("Mod+Shift+b"), false),
        Some(&Builtin::Quote.into())
    );
}

#[test]
fn view_commands_change_nothing_in_the_model() {
    let mut editor = editor("hel|lo");
    let commands = Commands::builtin();
    assert!(matches!(
        commands.get(&Builtin::Link.into()),
        Some(Action::View)
    ));
    assert!(!editor.run(&commands, Builtin::Link));
    assert!(!editor.handle_key(&Keymap::default(), &commands, &press("Mod+k"), false));
}

#[test]
fn chords_parse_and_print() {
    let chord = Chord::parse("Mod+Shift+Z").unwrap();
    assert!(chord.primary && chord.shift && !chord.alt);
    assert_eq!(chord.key, "z");
    assert_eq!(chord.to_string(), "Mod+Shift+z");
    assert_eq!(Chord::parse("Ctrl++").unwrap().key, "+");
    assert!(Chord::parse("Hyper+x").is_err());
    assert!(Chord::parse("").is_err());
}

#[test]
fn mod_is_cmd_on_apple_and_ctrl_elsewhere() {
    let chord = Chord::parse("Mod+b").unwrap();
    let ctrl_b = KeyPress {
        ctrl: true,
        ..KeyPress::new("b")
    };
    let cmd_b = KeyPress {
        meta: true,
        ..KeyPress::new("b")
    };
    assert!(chord.matches(&ctrl_b, false) && !chord.matches(&ctrl_b, true));
    assert!(chord.matches(&cmd_b, true) && !chord.matches(&cmd_b, false));
    let shifted = KeyPress {
        shift: true,
        ..ctrl_b
    };
    assert!(
        !chord.matches(&shifted, false),
        "extra modifiers do not match"
    );
}

#[test]
fn shifted_digits_match_by_code_but_letters_follow_the_layout() {
    let keymap = Keymap::default();
    let shift_seven = KeyPress {
        key: "&".into(),
        code: "Digit7".into(),
        ctrl: true,
        shift: true,
        ..KeyPress::default()
    };
    assert_eq!(
        keymap.command_for(&shift_seven, false),
        Some(&Builtin::OrderedList.into())
    );
    // German layout: the key labelled `y` sits where US `z` is.
    let german_y = KeyPress {
        key: "y".into(),
        code: "KeyZ".into(),
        ctrl: true,
        ..KeyPress::default()
    };
    assert_eq!(
        keymap.command_for(&german_y, false),
        Some(&Builtin::Redo.into())
    );
}

#[test]
fn a_caller_rebinds_and_adds_commands() {
    let mut commands = Commands::builtin();
    commands.register("shout", |state| {
        let text = state.selected_text().to_uppercase();
        !text.is_empty() && state.insert_text(&text)
    });
    let mut keymap = Keymap::default();
    keymap.bind(Chord::parse("Mod+b").unwrap(), "shout");
    keymap.unbind_command(Builtin::Italic);

    let mut editor = editor("a|bc|d");
    assert!(editor.handle_key(&keymap, &commands, &press("Mod+b"), false));
    assert_eq!(show(editor.state()), "aBC|d");
    assert!(!editor.handle_key(&keymap, &commands, &press("Mod+i"), false));
    assert_eq!(keymap.chords_for("shout").len(), 1);
    assert!(editor.undo());
    assert_eq!(show(editor.state()), "a|bc|d");
}

#[test]
fn unknown_commands_and_no_ops_report_false() {
    let commands = Commands::builtin();
    let mut editor = editor("|a");
    assert!(!editor.run(&commands, "nope"));
    assert!(
        !editor.run(&commands, Builtin::Indent),
        "Tab outside a list is left to focus"
    );
    assert!(!editor.can_undo());
}

#[test]
fn typing_merges_and_structure_steps() {
    let commands = Commands::builtin();
    let mut editor = editor("|");
    for c in ["h", "e", "y"] {
        editor.type_text(c);
    }
    editor.run(&commands, Builtin::SplitBlock);
    editor.type_text("x");
    assert_eq!(show(editor.state()), "hey\n\nx|");
    editor.undo();
    editor.undo();
    assert_eq!(show(editor.state()), "hey|");
    editor.undo();
    assert_eq!(show(editor.state()), "|");
    assert!(!editor.can_undo());
    editor.redo();
    assert_eq!(show(editor.state()), "hey|");
}

#[test]
fn moving_the_caret_starts_a_new_undo_group() {
    let mut editor = editor("ab|");
    editor.type_text("c");
    let first = editor.doc().first_leaf();
    let mut moved = editor.state().clone();
    moved.set_caret(crate::components::form::rich_text_editor::model::Position::new(first, 0));
    editor.select(moved.selection);
    editor.type_text("x");
    editor.undo();
    assert_eq!(show(editor.state()), "abc|");
}

#[test]
fn selection_only_commands_leave_no_undo_step() {
    let commands = Commands::builtin();
    let mut editor = editor("a|b");
    assert!(editor.run(&commands, Builtin::SelectAll));
    assert!(!editor.can_undo());
}

#[test]
fn custom_actions_choose_their_record() {
    let mut commands = Commands::empty();
    commands.register_action(
        "type_a",
        Action::Edit(
            std::rc::Rc::new(|state| state.insert_text("a")),
            Record::Merge,
        ),
    );
    let mut editor = editor("|");
    editor.run(&commands, "type_a");
    editor.run(&commands, "type_a");
    editor.undo();
    assert_eq!(show(editor.state()), "|");
}

#[test]
fn markdown_shortcuts_make_blocks() {
    let cases = [
        ("# ", "# |"),
        ("### ", "### |"),
        ("- ", "- |"),
        ("* ", "- |"),
        ("> ", "> |"),
        ("1. ", "1. |"),
        ("4) ", "4. |"),
        ("```rust ", "```rust\n|\n```"),
        ("--- ", "---\n\n|"),
    ];
    for (typed, expected) in cases {
        let mut editor = editor("|");
        for c in typed.chars() {
            editor.type_text(&c.to_string());
        }
        assert_eq!(show(editor.state()), expected, "typed {typed:?}");
    }
}

#[test]
fn markdown_shortcuts_mark_text() {
    let cases = [
        ("a **b** ", "a **b** |"),
        ("a *b* ", "a *b* |"),
        ("`c`", "`c|`"),
        ("~~s~~", "~~s|~~"),
        ("** b**", "\\*\\* b\\*\\*|"),
        ("2 * 3 *", "2 \\* 3 \\*|"),
    ];
    for (typed, expected) in cases {
        let mut editor = editor("|");
        for c in typed.chars() {
            editor.type_text(&c.to_string());
        }
        assert_eq!(show(editor.state()), expected, "typed {typed:?}");
    }
}

#[test]
fn undo_after_a_shortcut_brings_the_syntax_back() {
    let mut editor = editor("|");
    for c in "# ".chars() {
        editor.type_text(&c.to_string());
    }
    editor.undo();
    assert_eq!(show(editor.state()), "\\# |");
}

#[test]
fn no_shortcuts_inside_code() {
    let mut editor = editor("```\n|\n```");
    for c in "# **x**".chars() {
        editor.type_text(&c.to_string());
    }
    assert_eq!(show(editor.state()), "```\n# **x**|\n```");
}

#[test]
fn the_registry_refuses_builtin_names_and_builds_custom_nodes() {
    let mut registry = NodeRegistry::default();
    assert_eq!(
        registry
            .register(NodeSpec::block("paragraph", CustomContent::Inline))
            .err(),
        Some(RegistryError::Builtin("paragraph".into()))
    );
    registry
        .register(NodeSpec::block("callout", CustomContent::Blocks).attr("tone", "info"))
        .unwrap();
    let mut doc = Doc::new();
    let callout = registry
        .new_block(&mut doc, "callout", Default::default())
        .unwrap();
    assert_eq!(
        callout.children().len(),
        1,
        "a container starts with a paragraph"
    );
    doc.blocks.push(callout);
    doc.blocks[0].inlines_mut().push(
        crate::components::form::rich_text_editor::model::Inline::Node {
            name: "ghost".into(),
            attrs: Default::default(),
        },
    );
    assert_eq!(registry.unknown_in(&doc), vec!["ghost".to_string()]);
    assert!(
        registry
            .new_block(&mut doc, "paragraph", Default::default())
            .is_none()
    );
}
