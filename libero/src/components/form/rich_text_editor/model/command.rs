//! Named commands and the keymap: chords bound to command names, as data.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

use super::doc::BlockKind;
use super::mark::Mark;
use super::state::EditorState;

/// A command's name. Built-in ones come from [`Builtin`]; a caller's are any other string.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandName(pub Cow<'static, str>);

impl CommandName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for CommandName {
    fn from(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }
}

impl From<String> for CommandName {
    fn from(name: String) -> Self {
        Self(Cow::Owned(name))
    }
}

impl From<Builtin> for CommandName {
    fn from(builtin: Builtin) -> Self {
        Self(Cow::Borrowed(builtin.name()))
    }
}

impl fmt::Display for CommandName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

macro_rules! builtins {
    ($($variant:ident => $name:literal,)*) => {
        /// The editor's own commands.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[non_exhaustive]
        pub enum Builtin { $($variant,)* }

        impl Builtin {
            pub const ALL: &[Builtin] = &[$(Builtin::$variant,)*];

            pub fn name(self) -> &'static str {
                match self { $(Builtin::$variant => $name,)* }
            }
        }
    };
}

builtins! {
    Bold => "bold",
    Italic => "italic",
    Underline => "underline",
    Strike => "strike",
    Code => "code",
    Unlink => "unlink",
    Paragraph => "paragraph",
    Heading1 => "heading1",
    Heading2 => "heading2",
    Heading3 => "heading3",
    Heading4 => "heading4",
    Heading5 => "heading5",
    Heading6 => "heading6",
    CodeBlock => "code_block",
    BulletList => "bullet_list",
    OrderedList => "ordered_list",
    Quote => "quote",
    Indent => "indent",
    Outdent => "outdent",
    Rule => "rule",
    HardBreak => "hard_break",
    SplitBlock => "split_block",
    DeleteBackward => "delete_backward",
    DeleteForward => "delete_forward",
    SelectAll => "select_all",
    Undo => "undo",
    Redo => "redo",
    Link => "link",
    Shortcuts => "shortcuts",
    CodeLanguage => "code_language",
    ExitBlock => "exit_block",
}

/// A doc transform: `true` when it changed the state.
pub type EditFn = Rc<dyn Fn(&mut EditorState) -> bool>;

/// How a command's change lands in the undo history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Record {
    /// An undo step of its own.
    Step,
    /// Joins the open group, as typing does.
    Merge,
    /// No entry: moves the selection only.
    Skip,
}

#[derive(Clone)]
#[non_exhaustive]
pub enum Action {
    Edit(EditFn, Record),
    Undo,
    Redo,
    /// Opens the editor's own UI (a dialog); the pure model does nothing for it.
    View,
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(_, record) => write!(f, "Edit({record:?})"),
            Self::Undo => f.write_str("Undo"),
            Self::Redo => f.write_str("Redo"),
            Self::View => f.write_str("View"),
        }
    }
}

/// Command names to actions. Starts with the built-ins; a caller adds or replaces any.
#[derive(Clone, Debug)]
pub struct Commands {
    actions: BTreeMap<CommandName, Action>,
}

impl Default for Commands {
    fn default() -> Self {
        Self::builtin()
    }
}

fn edit(f: impl Fn(&mut EditorState) -> bool + 'static) -> Action {
    Action::Edit(Rc::new(f), Record::Step)
}

impl Commands {
    pub fn empty() -> Self {
        Self {
            actions: BTreeMap::new(),
        }
    }

    /// Every built-in. Shares its functions with other calls, so two compare equal.
    pub fn builtin() -> Self {
        thread_local! {
            static BUILTIN: Commands = Commands::fresh_builtin();
        }
        BUILTIN.with(Clone::clone)
    }

    fn fresh_builtin() -> Self {
        let mut commands = Self::empty();
        for builtin in Builtin::ALL {
            commands
                .actions
                .insert((*builtin).into(), builtin_action(*builtin));
        }
        commands
    }

    /// Adds a command that transforms the state, one undo step per run.
    pub fn register(
        &mut self,
        name: impl Into<CommandName>,
        run: impl Fn(&mut EditorState) -> bool + 'static,
    ) -> &mut Self {
        self.actions.insert(name.into(), edit(run));
        self
    }

    pub fn register_action(&mut self, name: impl Into<CommandName>, action: Action) -> &mut Self {
        self.actions.insert(name.into(), action);
        self
    }

    pub fn remove(&mut self, name: impl Into<CommandName>) -> Option<Action> {
        self.actions.remove(&name.into())
    }

    pub fn get(&self, name: &CommandName) -> Option<&Action> {
        self.actions.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &CommandName> {
        self.actions.keys()
    }
}

/// Same names running the same functions: what a component prop needs to skip re-renders.
impl PartialEq for Commands {
    fn eq(&self, other: &Self) -> bool {
        self.actions.len() == other.actions.len()
            && self
                .actions
                .iter()
                .zip(&other.actions)
                .all(|((a, x), (b, y))| a == b && same_action(x, y))
    }
}

fn same_action(a: &Action, b: &Action) -> bool {
    match (a, b) {
        (Action::Edit(f, r), Action::Edit(g, s)) => Rc::ptr_eq(f, g) && r == s,
        (Action::Undo, Action::Undo)
        | (Action::Redo, Action::Redo)
        | (Action::View, Action::View) => true,
        _ => false,
    }
}

fn builtin_action(builtin: Builtin) -> Action {
    use Builtin as B;
    match builtin {
        B::Bold => edit(|state| state.toggle_mark(Mark::Bold)),
        B::Italic => edit(|state| state.toggle_mark(Mark::Italic)),
        B::Underline => edit(|state| state.toggle_mark(Mark::Underline)),
        B::Strike => edit(|state| state.toggle_mark(Mark::Strike)),
        B::Code => edit(|state| state.toggle_mark(Mark::Code)),
        B::Unlink => edit(EditorState::remove_link),
        B::Paragraph => edit(|state| state.set_text_kind(BlockKind::Paragraph)),
        B::Heading1 => edit(|state| state.toggle_heading(1)),
        B::Heading2 => edit(|state| state.toggle_heading(2)),
        B::Heading3 => edit(|state| state.toggle_heading(3)),
        B::Heading4 => edit(|state| state.toggle_heading(4)),
        B::Heading5 => edit(|state| state.toggle_heading(5)),
        B::Heading6 => edit(|state| state.toggle_heading(6)),
        B::CodeBlock => edit(EditorState::toggle_code_block),
        B::BulletList => edit(|state| state.toggle_list(false)),
        B::OrderedList => edit(|state| state.toggle_list(true)),
        B::Quote => edit(EditorState::toggle_quote),
        B::Indent => edit(EditorState::indent),
        B::Outdent => edit(EditorState::outdent),
        B::Rule => edit(EditorState::insert_rule),
        B::ExitBlock => edit(EditorState::exit_code),
        B::HardBreak => edit(EditorState::insert_hard_break),
        // Not in `split_block` itself: a pasted fence line or blank line stays text.
        B::SplitBlock => edit(|state| {
            state.selection.is_collapsed() && (state.fence_rule() || state.exit_code_on_blank_end())
                || state.split_block()
        }),
        B::DeleteBackward => edit(EditorState::delete_backward),
        B::DeleteForward => edit(EditorState::delete_forward),
        B::SelectAll => Action::Edit(Rc::new(EditorState::select_all), Record::Skip),
        B::Undo => Action::Undo,
        B::Redo => Action::Redo,
        B::Link | B::Shortcuts | B::CodeLanguage => Action::View,
    }
}

/// A key press as the keymap sees it: `key` is `KeyboardEvent.key`, `code` its `code`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyPress {
    pub key: String,
    pub code: String,
    pub ctrl: bool,
    pub meta: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyPress {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ..Self::default()
        }
    }
}

/// A key with modifiers, written `"Mod+Shift+z"`. `Mod` is Cmd on Apple, Ctrl elsewhere.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chord {
    pub key: String,
    pub primary: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChordError(pub String);

impl fmt::Display for ChordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a chord: {:?}", self.0)
    }
}

impl std::error::Error for ChordError {}

impl std::str::FromStr for Chord {
    type Err = ChordError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let error = || ChordError(text.to_string());
        let mut chord = Chord {
            key: String::new(),
            primary: false,
            ctrl: false,
            meta: false,
            alt: false,
            shift: false,
        };
        let parts: Vec<&str> = match text.strip_suffix("++") {
            Some(rest) => rest.split('+').chain(["+"]).collect(),
            None => text.split('+').collect(),
        };
        let (key, modifiers) = parts.split_last().ok_or_else(error)?;
        for modifier in modifiers {
            let flag = match modifier.to_ascii_lowercase().as_str() {
                "mod" => &mut chord.primary,
                "ctrl" | "control" => &mut chord.ctrl,
                "meta" | "cmd" => &mut chord.meta,
                "alt" | "option" => &mut chord.alt,
                "shift" => &mut chord.shift,
                _ => return Err(error()),
            };
            *flag = true;
        }
        if key.is_empty() {
            return Err(error());
        }
        chord.key = normalize_key(key);
        Ok(chord)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (on, name) in [
            (self.primary, "Mod"),
            (self.ctrl, "Ctrl"),
            (self.meta, "Meta"),
            (self.alt, "Alt"),
            (self.shift, "Shift"),
        ] {
            if on {
                write!(f, "{name}+")?;
            }
        }
        f.write_str(&self.key)
    }
}

fn normalize_key(key: &str) -> String {
    match key.chars().count() {
        1 => key.to_lowercase(),
        _ => key.to_string(),
    }
}

/// The key a `code` names on a US layout: `KeyZ` -> `z`, `Digit7` -> `7`.
fn key_of_code(code: &str) -> Option<String> {
    code.strip_prefix("Key")
        .or_else(|| code.strip_prefix("Digit"))
        .filter(|rest| rest.len() == 1)
        .map(str::to_lowercase)
}

impl Chord {
    pub fn parse(text: &str) -> Result<Self, ChordError> {
        text.parse()
    }

    /// Whether `press` is this chord. `apple` maps `Mod` to Cmd. A symbol key matches
    /// with Shift too unless the chord names Shift: `/` is Shift+7 on a German layout.
    pub fn matches(&self, press: &KeyPress, apple: bool) -> bool {
        if normalize_key(&press.key) != self.key {
            return false;
        }
        let symbol = !self.key.chars().all(char::is_alphanumeric);
        self.modifiers_match(press, apple)
            || (symbol && !self.shift && press.shift && {
                let unshifted = KeyPress {
                    shift: false,
                    ..press.clone()
                };
                self.modifiers_match(&unshifted, apple)
            })
    }

    /// Matches by `code` when the key itself is not a plain letter or digit:
    /// Shift+7 reports `&`, Alt+c on a Mac `ç`. Never overrides the layout's letters,
    /// nor AltGr's characters off a Mac (Ctrl+Alt there): AltGr+2 types `²` in German.
    fn matches_by_code(&self, press: &KeyPress, apple: bool) -> bool {
        let plain =
            press.key.chars().count() == 1 && press.key.chars().all(|c| c.is_ascii_alphanumeric());
        let alt_graph = !apple && press.ctrl && press.alt;
        !plain
            && !alt_graph
            && self.modifiers_match(press, apple)
            && key_of_code(&press.code).as_deref() == Some(self.key.as_str())
    }

    fn modifiers_match(&self, press: &KeyPress, apple: bool) -> bool {
        let ctrl = self.ctrl || (self.primary && !apple);
        let meta = self.meta || (self.primary && apple);
        press.ctrl == ctrl
            && press.meta == meta
            && press.alt == self.alt
            && press.shift == self.shift
    }
}

/// Chords bound to command names. Defaults follow common editors; a caller rebinds freely.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keymap {
    bindings: Vec<(Chord, CommandName)>,
}

impl Default for Keymap {
    fn default() -> Self {
        use Builtin as B;
        let defaults: [(&str, Builtin); 33] = [
            ("Mod+b", B::Bold),
            ("Mod+i", B::Italic),
            ("Mod+u", B::Underline),
            ("Mod+Shift+s", B::Strike),
            ("Mod+e", B::Code),
            ("Mod+Alt+0", B::Paragraph),
            ("Mod+Alt+1", B::Heading1),
            ("Mod+Alt+2", B::Heading2),
            ("Mod+Alt+3", B::Heading3),
            ("Mod+Alt+4", B::Heading4),
            ("Mod+Alt+5", B::Heading5),
            ("Mod+Alt+6", B::Heading6),
            ("Mod+Alt+c", B::CodeBlock),
            ("Mod+Shift+8", B::BulletList),
            ("Mod+Shift+7", B::OrderedList),
            ("Mod+Shift+b", B::Quote),
            ("Tab", B::Indent),
            ("Shift+Tab", B::Outdent),
            ("Shift+Enter", B::HardBreak),
            ("Enter", B::SplitBlock),
            ("Backspace", B::DeleteBackward),
            ("Delete", B::DeleteForward),
            ("Mod+a", B::SelectAll),
            ("Mod+z", B::Undo),
            ("Mod+Shift+z", B::Redo),
            ("Mod+y", B::Redo),
            ("Mod+Shift+u", B::Unlink),
            ("Mod+Shift+Enter", B::Rule),
            ("Mod+Enter", B::ExitBlock),
            ("Mod+Shift+x", B::Strike),
            ("Mod+k", B::Link),
            ("Mod+/", B::Shortcuts),
            ("Mod+Shift+l", B::CodeLanguage),
        ];
        let mut keymap = Self::empty();
        for (chord, builtin) in defaults {
            keymap.bind(chord.parse().expect("a valid default chord"), builtin);
        }
        keymap
    }
}

impl Keymap {
    pub fn empty() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Binds `chord`, replacing what it ran before.
    pub fn bind(&mut self, chord: Chord, command: impl Into<CommandName>) -> &mut Self {
        self.unbind(&chord);
        self.bindings.push((chord, command.into()));
        self
    }

    pub fn unbind(&mut self, chord: &Chord) -> &mut Self {
        self.bindings.retain(|(bound, _)| bound != chord);
        self
    }

    /// Drops every chord of `command`.
    pub fn unbind_command(&mut self, command: impl Into<CommandName>) -> &mut Self {
        let command = command.into();
        self.bindings.retain(|(_, bound)| *bound != command);
        self
    }

    pub fn command_for(&self, press: &KeyPress, apple: bool) -> Option<&CommandName> {
        let found = |matches: fn(&Chord, &KeyPress, bool) -> bool| {
            self.bindings
                .iter()
                .find(|(chord, _)| matches(chord, press, apple))
                .map(|(_, command)| command)
        };
        found(Chord::matches).or_else(|| found(Chord::matches_by_code))
    }

    /// The chords that run `command`, for tooltips and a shortcut list.
    pub fn chords_for(&self, command: impl Into<CommandName>) -> Vec<&Chord> {
        let command = command.into();
        self.bindings
            .iter()
            .filter(|(_, bound)| *bound == command)
            .map(|(chord, _)| chord)
            .collect()
    }

    pub fn bindings(&self) -> &[(Chord, CommandName)] {
        &self.bindings
    }
}
