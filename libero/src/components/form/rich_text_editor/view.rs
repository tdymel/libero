//! [`RichTextEditor`]: the model's doc rendered into a `contenteditable`, every edit
//! routed through the model (web `beforeinput`), composition reconciled after it ends.

use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::core::{Runtime, current_scope_id};
use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::dialogs::{LinkArgs, LinkChoice, LinkDialog, announcement, shortcut_rows};
use super::handle::{RichTextHandle, Runner, Status};
use super::input::{EditorInput, Intent, intent, text_diff};
use super::model::{
    Action, BlockKind, Builtin, CommandName, Commands, Doc, Editor, EditorState, Inline, KeyPress,
    Keymap, Mark, MarkKind, NodeKey, NodeRegistry, Position, Record, Selection, UndoStack,
};
use super::node_view::NodeViews;
use super::offsets::{to_dom, to_model};
use super::render::{RenderCtx, blocks};
use super::surface::{Caret, ROOT_ATTR, Report, Surface};
use super::toolbar::{
    Group, Metrics, OVERFLOW, RichTextTool, Tool, hidden as overflow_count, tools,
};
use crate::{
    components::{
        accessibility::use_announcer,
        buttons::{ActionIcon, Button, Toolbar, ToolbarGroup, ToolbarSeparator},
        common::{Glyph, HtmlTag, Input, attr, names_itself, use_name_warning},
        form::{field_props, use_bound, use_field, use_field_frame},
        layout::use_box,
        overlay::{Menu, MenuEntry, MenuItem, Shortcut, ShortcutHelp, use_menu},
        typography::Language,
    },
    context::IconSlot,
    hooks::{
        ElementHandle, HistoryHandle, ModalScope, PopoverOptions, Rect, UndoHistory, listener,
        place, use_element, use_history, use_id, use_localization, use_modal, use_portal,
        use_theme,
    },
    localization::RichTextEditorLabels,
    platform::{Dimensions, ElementApi, mod_is_meta},
    sx::{StaticSx, sx},
    theme::{ANCHOR_COLOR, CODE_FONT_FAMILY, ColorCss, ColorShade, Z_INDEX_POPOVER},
};

impl UndoStack for HistoryHandle<EditorState> {
    fn present(&self) -> Rc<EditorState> {
        self.read(|history| history.present().clone())
    }

    fn push(&mut self, state: EditorState) {
        HistoryHandle::push(self, state);
    }

    fn merge(&mut self, state: EditorState) {
        HistoryHandle::merge(self, state);
    }

    fn seal(&mut self) {
        HistoryHandle::seal(self);
    }

    fn undo(&mut self) -> bool {
        HistoryHandle::undo(self)
    }

    fn redo(&mut self) -> bool {
        HistoryHandle::redo(self)
    }

    fn reset(&mut self, state: EditorState) {
        HistoryHandle::reset(self, state);
    }

    fn can_undo(&self) -> bool {
        self.read(|history| history.can_undo())
    }

    fn can_redo(&self) -> bool {
        self.read(|history| history.can_redo())
    }
}

type LiveEditor = Editor<HistoryHandle<EditorState>>;

/// Typing pauses longer than this start a new undo step.
const GROUP_MS: u64 = 500;

static SURFACE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .min_height("4.5em")
        .padding("0.5rem 0.75rem")
        .outline("none")
        .white_space("pre-wrap")
        .overflow_wrap("anywhere")
        .selector(
            "& > :first-child, & > * > :first-child",
            sx().margin_top("0"),
        )
        .selector(
            "& > :last-child, & > * > :last-child",
            sx().margin_bottom("0"),
        )
        .selector("& pre", sx().font_family("monospace").margin("0"))
        // A shown code block scrolls inside itself: its longest line must not widen the editor (1466).
        .selector("& [data-code='view']", sx().with("contain", "inline-size"))
        // So does the source while editing; the browser scrolls it to the caret (1475).
        .selector(
            "& [data-code='source'] > pre",
            sx().with("contain", "inline-size").overflow_x("auto"),
        )
        // List items hold paragraphs: no paragraph gaps between bullets.
        .selector("& li > p", sx().margin("0"))
        // Inline code, links and quotes as `Code`, `Anchor` and `Blockquote` draw them.
        .selector(
            "& :not(pre) > code",
            sx().background("muted.2")
                .border_radius("4px")
                .padding("0 0.25em")
                .font_family(CODE_FONT_FAMILY.value())
                .font_size("0.875em"),
        )
        .selector("& a", sx().color(ANCHOR_COLOR.value()))
        .selector(
            "& blockquote",
            sx().margin("1em 0")
                .padding("0 0.75rem")
                .border_left(format!(
                    "2px solid {}",
                    ColorCss::MUTED.value(ColorShade::S4)
                )),
        )
        .selector(
            "& [data-fence]",
            sx().font_family("monospace")
                .color("text-dimmed")
                .user_select("none"),
        )
        // The language menu's wrapper, beside the backticks.
        .selector(
            "& [data-fence] > div",
            sx().display("inline-block").margin_inline_start("0.25rem"),
        )
        .selector(
            "&[data-empty]::before",
            sx().content("attr(data-placeholder)")
                .color("text-dimmed")
                .pointer_events("none")
                .position("absolute"),
        )
});

field_props! {
    extends(div);
    pub struct RichTextEditorProps {
        /// The document. `None` leaves the editor uncontrolled.
        #[props(default, into)]
        value: Option<Doc>,
        /// Fires with the new document after every edit; selection moves do not fire.
        #[props(default)]
        onchange: Option<EventHandler<Doc>>,
        /// Rules over the document, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Doc>,
        /// A path binds the editor to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Doc>,
        /// Shown while the document is one empty paragraph.
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows the formatting toolbar above the text.
        #[props(default = true)]
        toolbar: bool,
        /// Key bindings; the default binds the usual editor chords.
        #[props(default)]
        keymap: Keymap,
        /// What the keymap and toolbar run; the default holds every built-in.
        #[props(default)]
        commands: Commands,
        /// Runs commands and reads the state from your own toolbar; see [`use_rich_text_editor`](super::handle::use_rich_text_editor).
        #[props(default, into)]
        handle: Option<RichTextHandle>,
        /// Components by node name, built-ins too (not `code_block`); a custom node
        /// without one draws a plain fallback.
        #[props(default)]
        nodes: NodeViews,
        /// The caller's node types: copy writes them through their codecs, and debug
        /// builds warn about a `nodes` name it lacks.
        #[props(default, into)]
        registry: Option<NodeRegistry>,
        /// Sees each key press (before the keymap) and typed text first; `true` takes it
        /// over, so the editor does nothing with it. Drives an `overlay` from the keyboard.
        #[props(default, into)]
        intercept: Option<Callback<EditorInput, bool>>,
        /// Floats at the caret while `Some`, such as a mention list. It renders in the
        /// page's portal: no ancestor clips it, and it does not inherit the editor's CSS
        /// context. Focus stays in the text: steer it through `intercept`, insert through
        /// the handle's `edit`.
        #[props(default, into)]
        overlay: Option<Element>,
        /// The id of the `overlay`'s highlighted option, so a screen reader announces it.
        #[props(default, into)]
        active_descendant: Option<String>,
        /// How many options the `overlay` lists; a change is announced while it shows.
        #[props(default, into)]
        overlay_results: Option<usize>,
        /// Your buttons, after the block buttons; each runs a command by name.
        #[props(default)]
        tools: Vec<RichTextTool>,
    }
}

/// The last docs this editor emitted: a controlled parent may echo any of them late.
#[derive(Default)]
struct Echoes(VecDeque<Doc>);

impl Echoes {
    const KEPT: usize = 8;

    fn record(&mut self, doc: Doc) {
        if self.0.len() == Self::KEPT {
            self.0.pop_front();
        }
        self.0.push_back(doc);
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    /// Whether `incoming` is neither the live doc nor one of ours: a reset.
    fn foreign(&self, current: &Doc, incoming: &Doc) -> bool {
        incoming != current && !self.0.contains(incoming)
    }
}

fn next_token() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed).to_string()
}

fn dom_position(editor: &LiveEditor, at: Position) -> (u64, usize) {
    let offset = editor
        .doc()
        .get(at.block)
        .map_or(0, |block| to_dom(block.inlines(), at.offset));
    (at.block.0, offset)
}

fn model_position(editor: &LiveEditor, key: u64, units: usize) -> Option<Position> {
    let block = editor
        .doc()
        .get(NodeKey(key))
        .filter(|block| block.is_leaf())?;
    Some(Position::new(block.key, to_model(block.inlines(), units)))
}

fn is_empty(doc: &Doc) -> bool {
    matches!(doc.blocks.as_slice(), [only] if only.is_leaf() && only.len() == 0 && !only.kind.is_code())
}

/// A rich text field: paragraphs, headings, lists, quotes, code blocks and marks,
/// with undo, the usual shortcuts and Markdown typing shortcuts.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{RichTextEditor, rich_text::Doc};
/// # fn app() -> Element {
/// let mut notes = use_signal(Doc::new);
/// rsx! {
///     RichTextEditor {
///         label: "Notes",
///         value: notes(),
///         onchange: move |doc| notes.set(doc),
///     }
///     pre { {notes.read().to_markdown()} }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/rich-text-editor>
#[component]
pub fn RichTextEditor(props: RichTextEditorProps) -> Element {
    let theme = use_theme();
    let size = props.size.copied_or(theme.textarea.size);
    let radius = props.radius.copied_or(theme.textarea.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let readonly = props.readonly.unwrap_or(false);
    let value = bound.value().or_else(|| props.value.clone());
    let editable = !disabled && !readonly && crate::platform::edits_rich_text();

    let initial = value.clone().unwrap_or_default();
    let history = use_history(|| UndoHistory::new(EditorState::new(initial)), GROUP_MS);
    let mut editor = use_hook(|| CopyValue::new(Editor::with_history(history)));
    // Bumped on every change, selection included: the toolbar's state.
    let mut revision = use_signal(|| 0u32);
    // Bumped when the model moved the caret, so the DOM selection follows it.
    let mut placed = use_signal(|| (0u32, false));
    // Bumped after a composition, so the browser's own text nodes are rebuilt.
    let mut generation = use_signal(|| 0u32);
    let mut focused = use_signal(|| false);
    let mut composing = use_hook(|| CopyValue::new(false));
    // While the DOM has not caught up with a model caret, its reports are stale.
    let mut syncing = use_hook(|| CopyValue::new(false));
    let token = use_hook(next_token);
    let mut can_edit = use_hook(|| CopyValue::new(editable));
    if *can_edit.peek() != editable {
        can_edit.set(editable);
    }

    let emit = use_hook(|| CopyValue::new(None::<Rc<dyn Fn(Doc)>>));
    {
        let mut emit_slot = emit;
        let next = bound
            .emit(props.onchange)
            .map(|emit| Rc::new(emit) as Rc<dyn Fn(Doc)>);
        emit_slot.set(next);
    }

    let commands = use_hook(|| CopyValue::new(Commands::default()));
    let keymap = use_hook(|| CopyValue::new(Keymap::default()));
    if *commands.peek() != props.commands {
        let mut commands = commands;
        commands.set(props.commands.clone());
    }
    // Bumped with the keymap: the toolbar's tooltips show its chords.
    let mut keymap_revision = use_hook(|| CopyValue::new(0u32));
    if *keymap.peek() != props.keymap {
        let mut keymap = keymap;
        keymap.set(props.keymap.clone());
        *keymap_revision.write() += 1;
    }
    let registry = use_hook(|| CopyValue::new(None::<NodeRegistry>));
    if *registry.peek() != props.registry {
        let mut registry = registry;
        registry.set(props.registry.clone());
    }
    #[cfg(debug_assertions)]
    {
        let mut checked = use_hook(|| CopyValue::new(None::<(NodeViews, NodeRegistry)>));
        if let Some(registry) = &props.registry {
            let pair = (props.nodes.clone(), registry.clone());
            if checked.peek().as_ref() != Some(&pair) {
                for name in props.nodes.unregistered(registry) {
                    crate::utils::warn(&format!(
                        "NodeViews: \"{name}\" is not a registered node; nodes of that name never reach this view."
                    ));
                }
                checked.set(Some(pair));
            }
        }
    }

    let mut echoes = use_hook(|| CopyValue::new(Echoes::default()));
    // A foreign controlled value replaces the doc; a late echo of our own does not.
    use_effect(use_reactive!(|value| {
        if let Some(doc) = value
            && echoes.peek().foreign(editor.peek().doc(), &doc)
        {
            echoes.write().clear();
            editor.write().reset(doc);
            revision += 1;
        }
    }));

    // After a model edit: tell the caller, re-render, put the DOM caret where the model has it.
    let mut changed = move |before: Option<Doc>, focus: bool| {
        if let Some(before) = before {
            let doc = editor.peek().doc().clone();
            if doc != before
                && let Some(emit) = emit.peek().clone()
            {
                echoes.write().record(doc.clone());
                emit(doc);
            }
        }
        revision += 1;
        syncing.set(true);
        let (count, _) = *placed.peek();
        placed.set((count + 1, focus));
    };

    let mut edit = move |run: &dyn Fn(&mut LiveEditor) -> bool, focus: bool| -> bool {
        let before = editor.peek().doc().clone();
        let done = run(&mut editor.write());
        if done {
            changed(Some(before), focus);
        }
        done
    };

    let words = use_localization().rich_text_editor;
    let announcer = use_announcer();
    let link_modal = use_modal(|scope: ModalScope<LinkArgs, LinkChoice>| {
        rsx! { LinkDialog { args: scope.args(), onchoose: move |choice| scope.resolve(choice) } }
    });
    let fence_menu = use_menu();
    let mut fence_key = use_hook(|| CopyValue::new(false));
    let help_modal = use_modal(move |scope: ModalScope<Vec<Shortcut>>| {
        rsx! { ShortcutHelp { title: words.shortcuts, shortcuts: scope.args() } }
    });

    // Opens a view command's dialog; the caret goes back to the text when it closes.
    let open_view = move |name: &CommandName| -> bool {
        if *name == Builtin::Link.into() {
            let href = link_href(editor.peek().state());
            link_modal
                .open_with(LinkArgs { href })
                .onresult(move |choice| {
                    let mut edit = edit;
                    match choice {
                        Some(LinkChoice::Set(href)) => edit(
                            &|live| {
                                live.apply(Record::Step, |state| {
                                    state.set_link(&href).unwrap_or(false)
                                })
                            },
                            true,
                        ),
                        Some(LinkChoice::Remove) => {
                            edit(&|live| live.run(&commands.peek(), Builtin::Unlink), true)
                        }
                        None => false,
                    };
                    let mut changed = changed;
                    changed(None, true);
                });
            return true;
        }
        if *name == Builtin::CodeLanguage.into() {
            let in_code = editor.peek().state().block_kind().is_code();
            if in_code {
                fence_menu.open();
            }
            return in_code;
        }
        if *name == Builtin::Shortcuts.into() {
            help_modal
                .open_with(shortcut_rows(&keymap.peek(), &words))
                .onresult(move |_| {
                    let mut changed = changed;
                    changed(None, true);
                });
            return true;
        }
        false
    };

    // Every command goes through here: keys, the toolbar and the handle.
    let run_command = move |name: CommandName, focus: bool, announce: bool| -> bool {
        if !*can_edit.peek() {
            return false;
        }
        if matches!(commands.peek().get(&name), Some(Action::View)) {
            return open_view(&name);
        }
        let mut edit = edit;
        let ran = edit(&|live| live.run(&commands.peek(), name.clone()), focus);
        if ran
            && announce
            && let Some(message) = announcement(&name, editor.peek().state(), &words)
        {
            announcer.say(message);
        }
        ran
    };

    let handle = use_hook(|| {
        let handle = props.handle;
        if let Some(handle) = handle {
            let owner = token.clone();
            // The caller's scope sits above the values these read, so they run as the editor's.
            let scope = current_scope_id();
            let as_editor = move |f: &mut dyn FnMut() -> bool| {
                Runtime::try_current().is_some_and(|runtime| runtime.in_scope(scope, f))
            };
            let run = Rc::new(move |name: CommandName| {
                as_editor(&mut || run_command(name.clone(), true, false))
            });
            let edit = Rc::new(move |f: &mut dyn FnMut(&mut EditorState) -> bool| {
                as_editor(&mut || {
                    if !*can_edit.peek() {
                        return false;
                    }
                    let f = std::cell::RefCell::new(&mut *f);
                    let mut edit = edit;
                    edit(
                        &|live| live.apply(Record::Step, |state| (f.borrow_mut())(state)),
                        true,
                    )
                })
            });
            handle.attach(&owner, Runner { run, edit });
        }
        handle
    });
    {
        let token = token.clone();
        use_drop(move || {
            if let Some(handle) = handle {
                handle.detach(&token);
            }
        });
    }
    {
        let token = token.clone();
        use_effect(move || {
            let _ = revision();
            if let Some(handle) = handle {
                let live = editor.peek();
                handle.publish(
                    &token,
                    Status::of(live.state(), live.can_undo(), live.can_redo()),
                    live.state(),
                );
            }
        });
    }

    // Kept current for the surface's held-back input, which runs outside a render.
    let mut intercept = use_hook(|| CopyValue::new(props.intercept));
    intercept.set(props.intercept);
    let taken = move |input: EditorInput| {
        let intercept = *intercept.peek();
        intercept.is_some_and(|intercept| intercept.call(input))
    };
    // A `beforeinput` by its type, data and the last key pressed; `true` cancels it.
    let before_input = move |input_type: &str, data: Option<String>, key: &str| -> bool {
        let (mut composing, mut edit) = (composing, edit);
        match intent(input_type, data) {
            Intent::Pass => {
                composing.set(true);
                false
            }
            Intent::Cancel => true,
            Intent::Type(text) => {
                if !taken(EditorInput::Text(text.clone())) {
                    edit(&|live| live.type_text(&text), false);
                }
                true
            }
            Intent::Run(Builtin::SplitBlock)
                if key != "Enter"
                    && taken(EditorInput::Key(KeyPress {
                        key: "Enter".into(),
                        code: "Enter".into(),
                        ..KeyPress::default()
                    })) =>
            {
                true
            }
            Intent::Run(builtin) => {
                edit(&|live| live.run(&commands.peek(), builtin), false);
                true
            }
            Intent::Delete(delete) => {
                edit(
                    &|live| live.apply(Record::Step, |state| delete.run(state)),
                    false,
                );
                true
            }
        }
    };

    // Android soft keyboards press "Unidentified": their Enter shows up as an insertParagraph.
    let mut last_key = use_hook(|| CopyValue::new(String::new()));
    let mut escaped = use_hook(|| CopyValue::new(false));
    let apple = mod_is_meta();
    // A key press through `intercept` and the keymap; `true` cancels it.
    let mut key_down = move |press: KeyPress| -> bool {
        last_key.set(press.key.clone());
        if taken(EditorInput::Key(press.clone())) {
            return true;
        }
        // Escape then Tab leaves instead of indenting (WCAG 2.1.2, todo 1610).
        let armed = std::mem::replace(&mut *escaped.write(), press.key == "Escape");
        if armed && press.key == "Tab" {
            return false;
        }
        let name = keymap.peek().command_for(&press, apple).cloned();
        name.is_some_and(|name| run_command(name, false, true))
    };

    // The caret's place, kept always; redraws only while an overlay follows it.
    let mut last_caret = use_hook(|| CopyValue::new(None::<Caret>));
    let mut caret_tick = use_signal(|| 0u32);
    let mut wants_caret = use_hook(|| CopyValue::new(false));
    if *wants_caret.peek() != props.overlay.is_some() {
        wants_caret.set(props.overlay.is_some());
    }
    let surface = use_hook(|| {
        Surface::start(token.clone(), move |report: Report| {
            if report.caret.is_some() && *last_caret.peek() != report.caret {
                last_caret.set(report.caret);
                if *wants_caret.peek() {
                    caret_tick += 1;
                }
            }
            let mut clip = None;
            if let Some((anchor_key, anchor, head_key, head)) = report.selection
                && (report.press || !*syncing.peek())
                && !*composing.peek()
            {
                let selection = {
                    let live = editor.peek();
                    model_position(&live, anchor_key, anchor)
                        .zip(model_position(&live, head_key, head))
                        .map(|(anchor, head)| Selection::range(anchor, head))
                };
                if let Some(selection) = selection {
                    if selection != editor.peek().state().selection {
                        editor.write().select(selection);
                        revision += 1;
                    }
                    clip = clip_of(&editor.peek(), &registry.peek());
                }
            }
            if let Some((input_type, data, key)) = report.input
                && *can_edit.peek()
            {
                before_input(&input_type, data, &key);
            }
            if let Some((key, code, ctrl, meta, alt, shift)) = report.key
                && *can_edit.peek()
                && !*composing.peek()
            {
                key_down(KeyPress {
                    key,
                    code,
                    ctrl,
                    meta,
                    alt,
                    shift,
                });
            }
            if report.synced {
                syncing.set(false);
            }
            if let Some((key, text)) = report.text {
                reconcile(editor, NodeKey(key), &text, &mut changed);
                generation += 1;
            }
            if let Some((key, at_end)) = report.code
                && *can_edit.peek()
            {
                enter_code(editor, NodeKey(key), at_end);
                changed(None, true);
            }
            if (report.exit || report.end) && *can_edit.peek() {
                let leave = if report.exit {
                    EditorState::exit_code
                } else {
                    EditorState::exit_end
                };
                let before = editor.peek().doc().clone();
                if editor.write().apply(Record::Step, leave) {
                    focused.set(true);
                    changed(Some(before), true);
                }
            }
            clip
        })
    });

    use_effect(move || {
        let (_, focus) = placed();
        let live = editor.peek();
        let selection = live.state().selection;
        let (anchor, head) = (
            dom_position(&live, selection.anchor),
            dom_position(&live, selection.head),
        );
        surface.select(anchor, head, focus);
        if let Some(markdown) = clip_of(&live, &registry.peek()) {
            surface.clip(anchor, head, markdown);
        }
    });

    let _ = revision();
    let live = editor.read();
    let state = live.state();
    let caret = state.caret().block;
    let source_code = (focused() && state.block(caret).kind.is_code()).then_some(caret);
    let on_code = editable.then(|| {
        EventHandler::new(move |key: NodeKey| {
            enter_code(editor, key, true);
            focused.set(true);
            changed(None, true);
        })
    });
    let empty = is_empty(live.doc());
    let mut buttons = tools(&words);
    for tool in &mut buttons {
        let mark = |kind| Some(state.is_active(kind));
        tool.selected = match tool.builtin {
            Builtin::Bold => mark(MarkKind::Bold),
            Builtin::Italic => mark(MarkKind::Italic),
            Builtin::Underline => mark(MarkKind::Underline),
            Builtin::Strike => mark(MarkKind::Strike),
            Builtin::Code => mark(MarkKind::Code),
            Builtin::Link => mark(MarkKind::Link),
            Builtin::BulletList => Some(state.list_kind() == Some(false)),
            Builtin::OrderedList => Some(state.list_kind() == Some(true)),
            Builtin::Quote => Some(state.in_quote()),
            Builtin::CodeBlock => Some(state.block_kind().is_code()),
            _ => None,
        };
        tool.disabled = match tool.builtin {
            Builtin::Undo => !live.can_undo(),
            Builtin::Redo => !live.can_redo(),
            _ => false,
        };
    }
    let custom_tools: Vec<(RichTextTool, Option<bool>)> = props
        .tools
        .iter()
        .map(|tool| (tool.clone(), tool.active.map(|active| active(state))))
        .collect();
    let block_kind = state.block_kind().clone();
    drop(live);

    let language_menu = use_menu();
    let code_language = match &block_kind {
        BlockKind::CodeBlock { language } => Some(language.clone()),
        _ => None,
    };
    let language_label = code_language.as_deref().map(|language| match language {
        "" => words.plain_text.to_string(),
        name => Language::label_of(name).unwrap_or(name).to_string(),
    });
    let language_items: Vec<MenuEntry> = {
        let current = code_language.clone().unwrap_or_default();
        // A typed name the catalog does not know stays listed, so it shows as chosen.
        let unknown = (!current.is_empty() && Language::label_of(&current).is_none())
            .then(|| (current.clone(), current.clone()));
        std::iter::once((words.plain_text.to_string(), String::new()))
            .chain(Language::catalog().map(|(label, name)| (label.into(), name.into())))
            .chain(unknown)
            .map(|(label, name): (String, String)| {
                let on = Language::label_of(&name) == Language::label_of(&current)
                    && (name.is_empty() == current.is_empty());
                MenuItem::new(label)
                    .radio(on)
                    .onselect(move |_| {
                        let mut edit = edit;
                        let name = name.clone();
                        edit(
                            &|live| {
                                live.apply(Record::Step, |state| state.set_code_language(&name))
                            },
                            true,
                        );
                    })
                    .into()
            })
            .collect()
    };
    // The same menu on the block's opening fence, also without a toolbar.
    let fence = source_code.filter(|_| editable).map(|_| {
        let shown = match code_language.as_deref() {
            Some("") | None => words.plain_text.to_string(),
            Some(name) => name.to_string(),
        };
        let mut attributes = fence_menu.a11y_attributes();
        attributes.extend([
            attr("tabindex", "-1"),
            // The name starts with the visible fence text, so voice control finds it.
            attr("aria-label", format!("{shown}, {}", words.code_language)),
            listener("onmousedown", |event: MouseEvent| event.prevent_default()),
            // The button's keys are the menu's, not the text's.
            listener("onkeydown", move |_: KeyboardEvent| fence_key.set(true)),
        ]);
        rsx! {
            Menu { state: fence_menu, items: language_items.clone(),
                Button { attributes, variant: "outlined", color: "ink", size: "xs", "{shown}" }
            }
        }
    });
    let content = {
        let live = editor.peek();
        blocks(
            &live.doc().blocks,
            RenderCtx {
                source_code,
                on_code,
                views: &props.nodes,
                fence: fence.as_ref(),
            },
        )
    };

    // `for` names only a labelable element, and the surface is a `div`.
    let mut field = use_field()
        .labelled_by()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(bound.check(&props.validate, value.clone()))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();
    // Read on focus: Tab indents a list, so say the way out (WCAG 2.1.2, todo 1610).
    let leave_hint = format!("{}-leave-hint", field.id());
    if editable {
        field.describe_also(leave_hint.clone());
    }
    use_name_warning(
        field.label_id().is_some() || names_itself(&props.attributes),
        "RichTextEditor: no `label`, `aria-label` or `aria-labelledby`, so it is announced as just \"edit text\".",
    );
    let frame = use_field_frame()
        .states(field.states())
        .multiline()
        .prepare();
    let control = use_box()
        .framework_sx(&SURFACE_SX)
        .focus_ring(false)
        .prepare();
    let element = use_element();

    let onkeydown = move |event: KeyboardEvent| {
        // Not `is_composing()`: Gboard keeps a composing region open over typed words.
        if std::mem::take(&mut *fence_key.write()) || *composing.peek() {
            return;
        }
        let modifiers = event.modifiers();
        let press = KeyPress {
            key: event.key().to_string(),
            code: event.code().to_string(),
            ctrl: modifiers.ctrl(),
            meta: modifiers.meta(),
            alt: modifiers.alt(),
            shift: modifiers.shift(),
        };
        if key_down(press) {
            event.prevent_default();
        }
    };

    let onbeforeinput = move |event: Event<BeforeInputData>| {
        let data = event.data();
        let key = std::mem::take(&mut *last_key.write());
        if before_input(&data.input_type().to_string(), data.data(), &key) {
            event.prevent_default();
        }
    };

    // Markdown back into nodes: the editor's own copy, else text read a paragraph per line.
    let onpaste = move |event: ClipboardEvent| {
        event.prevent_default();
        let transfer = event.data().data_transfer();
        let (text, markdown) = match transfer.get_data("text/markdown") {
            Some(text) if !text.is_empty() => (text, true),
            _ => match transfer.get_as_text() {
                Some(text) => (text, false),
                None => return,
            },
        };
        edit(
            &|live| live.apply(Record::Step, |state| state.paste(&text, markdown)),
            false,
        );
    };

    // The selection as Markdown, in both plain text and `text/markdown` (todo 1258);
    // `false` when nothing was selected. A WebView's event is a copy: the script writes there.
    let copy = move |event: &ClipboardEvent| -> bool {
        let registry = registry.peek().clone().unwrap_or_default();
        let Some(markdown) = selected_markdown(&editor.peek(), &registry) else {
            return false;
        };
        if cfg!(target_arch = "wasm32") {
            let transfer = event.data().data_transfer();
            if transfer.set_data("text/plain", &markdown).is_err() {
                return false;
            }
            let _ = transfer.set_data("text/markdown", &markdown);
            event.prevent_default();
        }
        true
    };
    let oncopy = move |event: ClipboardEvent| {
        copy(&event);
    };
    // The browser's own cut would edit the DOM behind the model.
    let oncut = move |event: ClipboardEvent| {
        event.prevent_default();
        if copy(&event) {
            edit(
                &|live| live.apply(Record::Step, |state| state.delete_selection()),
                false,
            );
        }
    };

    // Gboard opens composing regions over words the model typed; only text the browser
    // composed itself (`insertCompositionText`) is read back, or a lagging DOM undoes keys.
    let oncompositionend = move |_: CompositionEvent| {
        if *composing.peek() {
            composing.set(false);
            surface.read(editor.peek().state().caret().block.0);
        }
    };

    let overlay_id = use_id();
    let open = props.overlay.is_some();
    let surface_element = field
        .aria(control)
        .element(&element)
        .attr(ROOT_ATTR, token.clone())
        .attr("role", "textbox")
        .attr("aria-labelledby", field.label_id())
        .attr("aria-multiline", "true")
        // A read-only surface is not `contenteditable` but stays a tab stop for reading.
        .attr("tabindex", (!disabled).then_some("0"))
        .attr("aria-disabled", disabled.then_some("true"))
        // Combobox-like while the overlay is open; `aria-expanded` is not allowed on a textbox.
        .attr("aria-autocomplete", open.then_some("list"))
        .attr("aria-controls", open.then(|| overlay_id.cloned()))
        .attr(
            "aria-activedescendant",
            props.active_descendant.clone().filter(|_| open),
        )
        .attr("aria-readonly", readonly)
        .attr("aria-placeholder", props.placeholder.clone())
        .attr("data-placeholder", props.placeholder.clone())
        .attr("data-empty", empty)
        .attr("contenteditable", if editable { "true" } else { "false" })
        .attr("spellcheck", "false")
        .event("onkeydown", editable.then_some(onkeydown))
        .event("onbeforeinput", editable.then_some(onbeforeinput))
        .event("onpaste", editable.then_some(onpaste))
        .event("oncopy", editable.then_some(oncopy))
        .event("oncut", editable.then_some(oncut))
        .event("oncompositionend", editable.then_some(oncompositionend))
        .event("onfocusin", move |_: FocusEvent| focused.set(true))
        // The fence's language menu takes focus from the text; the source stays meanwhile.
        .event("onfocusout", move |_: FocusEvent| {
            if !fence_menu.is_open() {
                focused.set(false);
            }
        })
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                for generation in [generation()] {
                    div { key: "{generation}", style: "display: contents", {content.clone()} }
                }
            },
        );

    // The fence's menu closed: focus back in the text gets the caret, focus elsewhere ends editing.
    let mut fence_was_open = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        let open = fence_menu.is_open();
        let was_open = std::mem::replace(&mut *fence_was_open.write(), open);
        if open || !was_open {
            return;
        }
        if element.is_focused() || element.query_selector(":focus").is_ok() {
            changed(None, true);
        } else {
            focused.set(false);
        }
    });

    let run = move |builtin: Builtin| {
        move |_: MouseEvent| {
            run_command(builtin.into(), true, true);
        }
    };
    // Pressing a button must not take focus or the selection from the text.
    let keep = |event: MouseEvent| event.prevent_default();
    let block_menu = use_menu();
    let heading = |level: u8| words.heading.replace("{level}", &level.to_string());
    let block_label = match block_kind {
        BlockKind::Heading { level } => heading(level),
        BlockKind::CodeBlock { .. } => words.code_block.to_string(),
        _ => words.paragraph.to_string(),
    };
    let block_items: Vec<MenuEntry> = {
        use Builtin as B;
        let paragraph = std::iter::once((
            B::Paragraph,
            words.paragraph.to_string(),
            block_kind == BlockKind::Paragraph,
        ));
        let headings = [
            B::Heading1,
            B::Heading2,
            B::Heading3,
            B::Heading4,
            B::Heading5,
            B::Heading6,
        ]
        .into_iter()
        .zip(1u8..)
        .map(|(builtin, level)| {
            (
                builtin,
                heading(level),
                block_kind == BlockKind::Heading { level },
            )
        });
        paragraph
            .chain(headings)
            .map(|(builtin, label, on)| {
                MenuItem::new(label)
                    .radio(on)
                    .onselect(move |_| {
                        run_command(builtin.into(), true, true);
                    })
                    .into()
            })
            .collect()
    };
    // The wrapper's width over the measured parts: the buttons a narrow bar moves into More.
    let mut bar_width = use_signal(|| None::<f64>);
    let metrics = use_signal(|| None::<Metrics>);
    let (bold, italic, block_button) = (use_element(), use_element(), use_element());
    // The widest language label seen, so moving between code blocks never reflows the bar.
    let mut language_width = use_signal(|| None::<f64>);
    let crowded = bar_width().map_or(0, |width| {
        let measured = metrics().unwrap_or_default();
        let language = match language_label {
            Some(_) => language_width().unwrap_or(measured.block_type),
            None => 0.0,
        };
        overflow_count(
            width,
            Metrics {
                language,
                ..measured
            },
            custom_tools.len(),
        )
    });
    // Reads start here, not in the task: Blitz fails a read made inside one.
    let measure = move || {
        let reads = (
            bold.dimensions(),
            bold.client_offset(),
            italic.client_offset(),
            block_button.dimensions(),
        );
        spawn(async move {
            let (Ok(icon), Ok((from, _)), Ok((to, _)), Ok(block)) =
                (reads.0.await, reads.1.await, reads.2.await, reads.3.await)
            else {
                return;
            };
            if icon.width <= 0.0 {
                return;
            }
            let mut metrics = metrics;
            let known = metrics.peek().unwrap_or_default();
            // Bold and Italic on two rows of a wrapped bar give no gap.
            let gap = (to - from).abs() - icon.width;
            let next = Some(Metrics {
                icon: icon.width,
                gap: if (0.0..icon.width).contains(&gap) {
                    gap
                } else {
                    known.gap
                },
                // The widest label seen, so moving the caret never reflows the bar.
                block_type: block
                    .width
                    .max(metrics.peek().map_or(0.0, |m| m.block_type)),
                language: 0.0,
            });
            if *metrics.peek() != next {
                metrics.set(next);
            }
        });
    };
    let more_menu = use_menu();
    let hidden = &OVERFLOW[..crowded];
    let more_items: Vec<MenuEntry> = buttons
        .iter()
        .filter(|tool| hidden.contains(&tool.builtin))
        .map(|tool| {
            let builtin = tool.builtin;
            let item = MenuItem::new(tool.label)
                .leading(rsx! { Glyph { slot: tool.slot, icon: tool.icon } })
                .disabled(tool.disabled)
                .onselect(move |_| {
                    run_command(builtin.into(), true, true);
                });
            match tool.selected {
                Some(on) => item.checkbox(on),
                None => item,
            }
            .into()
        })
        .collect();
    let shown = |group: Group| -> Vec<Tool> {
        buttons
            .iter()
            .filter(|tool| tool.group == group && !hidden.contains(&tool.builtin))
            .copied()
            .collect()
    };
    let (marks, block_tools, history) = (
        shown(Group::Marks),
        shown(Group::Blocks),
        shown(Group::History),
    );
    // A theme or a new label resizes Bold or the block type button: the parts are read again.
    let sized = move |handle: ElementHandle| {
        vec![
            listener("onmounted", handle.mount()),
            listener("onresize", move |_: Event<ResizeData>| measure()),
        ]
    };
    let button = move |tool: Tool| {
        let attributes = match tool.builtin {
            Builtin::Bold => sized(bold),
            Builtin::Italic => vec![listener("onmounted", italic.mount())],
            _ => Vec::new(),
        };
        let shortcut = keymap
            .peek()
            .chords_for(tool.builtin)
            .first()
            .map(|chord| chord.to_string().to_lowercase());
        rsx! {
            ActionIcon {
                key: "{tool.label}",
                attributes,
                aria_label: tool.label,
                tooltip: true,
                shortcut,
                selected: tool.selected,
                disabled: tool.disabled,
                onclick: run(tool.builtin),
                Glyph { slot: tool.slot, icon: tool.icon }
            }
        }
    };
    let custom_button = move |tool: RichTextTool, selected: Option<bool>| {
        let shortcut = keymap
            .peek()
            .chords_for(tool.command.clone())
            .first()
            .map(|chord| chord.to_string().to_lowercase());
        let command = tool.command.clone();
        rsx! {
            ActionIcon {
                key: "{tool.command}",
                aria_label: tool.label,
                tooltip: true,
                shortcut,
                selected,
                onclick: move |_| {
                    run_command(command.clone(), true, false);
                },
                {tool.icon}
            }
        }
    };
    let onresize = move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_content_box_size()
            && *bar_width.peek() != Some(size.width)
        {
            bar_width.set(Some(size.width));
        }
        if metrics.peek().is_none() {
            measure();
        }
    };
    let mut block_attributes = block_menu.a11y_attributes();
    block_attributes.extend(sized(block_button));
    let mut language_attributes = language_menu.a11y_attributes();
    language_attributes.push(listener("onresize", move |event: Event<ResizeData>| {
        if let Ok(size) = event.get_border_box_size()
            && size.width > language_width.peek().unwrap_or(0.0)
        {
            language_width.set(Some(size.width));
        }
    }));
    let look = ToolbarLook {
        words,
        tools: buttons
            .iter()
            .map(|tool| (tool.builtin, tool.selected, tool.disabled))
            .collect(),
        custom_tools: custom_tools.clone(),
        block_kind: block_kind.clone(),
        crowded,
        menus: [
            block_menu.a11y_attributes(),
            language_menu.a11y_attributes(),
            more_menu.a11y_attributes(),
        ],
        keymap: *keymap_revision.peek(),
    };
    let toolbar = (props.toolbar && editable).then(|| {
        rsx! {
            ToolbarSlot { look, bar: rsx! {
            div { onresize,
                Toolbar { "aria-label": words.toolbar, focus_from: element, onmousedown: keep,
                    ToolbarGroup { "aria-label": words.marks,
                        for tool in marks { {button(tool)} }
                    }
                    ToolbarSeparator {}
                    ToolbarGroup { "aria-label": words.blocks,
                        Menu { state: block_menu, items: block_items,
                            Button {
                                attributes: block_attributes,
                                "aria-label": "{words.block_type}: {block_label}",
                                variant: "standard",
                                color: "ink",
                                "{block_label}"
                            }
                        }
                        if let Some(language) = language_label {
                            Menu { state: language_menu, items: language_items,
                                Button {
                                    attributes: language_attributes,
                                    "aria-label": "{words.language}: {language}",
                                    variant: "standard",
                                    color: "ink",
                                    "{language}"
                                }
                            }
                        }
                        for tool in block_tools { {button(tool)} }
                    }
                    if !custom_tools.is_empty() {
                        ToolbarSeparator {}
                        ToolbarGroup { "aria-label": words.custom_tools,
                            for (tool, selected) in custom_tools {
                                {custom_button(tool, selected)}
                            }
                        }
                    }
                    if !history.is_empty() {
                        ToolbarSeparator {}
                        ToolbarGroup { "aria-label": words.history,
                            for tool in history { {button(tool)} }
                        }
                    }
                    if !more_items.is_empty() {
                        Menu { state: more_menu, items: more_items,
                            ActionIcon { attributes: more_menu.a11y_attributes(), aria_label: words.more,
                                Glyph { slot: IconSlot::More, icon: lucide::ellipsis_vertical::outlined }
                            }
                        }
                    }
                }
            }
            } }
        }
    });

    // Said on change only; a closed overlay resets it, so reopening says it again (WCAG 4.1.3).
    let results = props.overlay_results.filter(|_| props.overlay.is_some());
    let mut said_results = use_hook(|| CopyValue::new(None::<usize>));
    use_effect(use_reactive!(|results| {
        if *said_results.peek() != results {
            said_results.set(results);
            if let Some(message) = results_announcement(results, &words) {
                announcer.say(message);
            }
        }
    }));

    let mut overlay_size = use_signal(|| None::<Dimensions>);
    let overlay = props.overlay.clone().map(|overlay| {
        let _ = caret_tick();
        let style = match (*last_caret.peek(), overlay_size()) {
            (Some(caret), Some(size)) => {
                let anchor = Rect {
                    x: caret.x,
                    y: caret.y,
                    width: 0.0,
                    height: caret.height,
                };
                let viewport = Dimensions {
                    width: caret.width,
                    height: caret.viewport_height,
                };
                let placed = place(
                    anchor,
                    size,
                    viewport,
                    &PopoverOptions::new(4.0, 8.0),
                    caret.rtl,
                );
                (placed.x, placed.y, "visible")
            }
            // Unmeasured: laid out hidden first, so the placer knows its size.
            _ => (0.0, 0.0, "hidden"),
        };
        // Every property each time: a style string's diff keeps a dropped one.
        let (left, top, visibility) = style;
        let layer = Z_INDEX_POPOVER.value();
        rsx! {
            div {
                id: overlay_id,
                "data-overlay": token.clone(),
                style: "position: fixed; z-index: {layer}; left: {left}px; top: {top}px; visibility: {visibility}",
                onmousedown: keep,
                // A resize observer may report late or not at all for a fresh box: read it once.
                onmounted: move |event: MountedEvent| {
                    // Blitz fails a read from a task; it never edits, so never shows one.
                    if !crate::platform::edits_rich_text() {
                        return;
                    }
                    let mounted = event.data();
                    spawn(async move {
                        if let Ok(rect) = mounted.get_client_rect().await
                            && overlay_size.peek().is_none()
                        {
                            overlay_size.set(Some(Dimensions { width: rect.width(), height: rect.height() }));
                        }
                    });
                },
                onresize: move |event: Event<ResizeData>| {
                    // The wrapper has no padding or border: its content box is its size.
                    if let Ok(size) = event.get_content_box_size() {
                        let next = Some(Dimensions { width: size.width, height: size.height });
                        if *overlay_size.peek() != next {
                            overlay_size.set(next);
                        }
                    }
                },
                {overlay}
            }
        }
    });

    // Through the portal: an ancestor's overflow would clip it in place (todo 2067).
    use_portal(overlay);

    field.render(rsx! {
        {toolbar}
        {frame.render(surface_element)}
        {announcer.render()}
        if editable {
            span { id: leave_hint, hidden: true, {words.leave_hint} }
        }
    })
}

/// Everything the toolbar shows. Its handlers read only stable handles, so a bar drawn from
/// an equal look is still current.
#[derive(Clone, PartialEq)]
struct ToolbarLook {
    words: RichTextEditorLabels,
    tools: Vec<(Builtin, Option<bool>, bool)>,
    custom_tools: Vec<(RichTextTool, Option<bool>)>,
    /// The block type and code language menus' radios and labels.
    block_kind: BlockKind,
    crowded: usize,
    menus: [Vec<Attribute>; 3],
    keymap: u32,
}

#[derive(Props, Clone)]
struct ToolbarSlotProps {
    look: ToolbarLook,
    bar: Element,
}

/// Compares the look alone: `bar` never compares equal.
impl PartialEq for ToolbarSlotProps {
    fn eq(&self, other: &Self) -> bool {
        self.look == other.look
    }
}

/// The toolbar, skipped while its look holds: typing redrew every button and tooltip (todo 2002).
#[component]
fn ToolbarSlot(props: ToolbarSlotProps) -> Element {
    props.bar
}

/// What an open overlay's option count says; `None` is silent.
fn results_announcement(results: Option<usize>, words: &RichTextEditorLabels) -> Option<String> {
    results.map(|count| match count {
        0 => words.nothing_found.to_string(),
        count => (words.results)(count),
    })
}

/// The href of the link around the caret.
fn link_href(state: &EditorState) -> Option<String> {
    let (key, from, _) = state.link_at_caret()?;
    let mut at = 0;
    let mark = state.block(key).inlines().iter().find_map(|inline| {
        let start = at;
        at += inline.len();
        match inline {
            Inline::Text { marks, .. } if start <= from && from < at => {
                marks.get(MarkKind::Link).cloned()
            }
            _ => None,
        }
    })?;
    match mark {
        Mark::Link { href, .. } => Some(href.as_str().to_string()),
        _ => None,
    }
}

/// The selection as Markdown; `None` when it is collapsed.
fn selected_markdown(live: &LiveEditor, registry: &NodeRegistry) -> Option<String> {
    let state = live.state();
    if state.selection.is_collapsed() {
        return None;
    }
    let markdown = state.selected_doc().to_markdown_with(registry);
    Some(markdown.trim_end_matches('\n').to_string())
}

/// What the script copies in a WebView, where Rust's clipboard data never reaches the event
/// (todo 2106); `None` on the web, whose `oncopy` writes it.
fn clip_of(live: &LiveEditor, registry: &Option<NodeRegistry>) -> Option<String> {
    if cfg!(target_arch = "wasm32") {
        return None;
    }
    selected_markdown(live, &registry.clone().unwrap_or_default())
}

/// Puts the caret at the start or end of code block `key`, which then renders as source.
fn enter_code(mut editor: CopyValue<LiveEditor>, key: NodeKey, at_end: bool) {
    let at = match editor.peek().doc().get(key) {
        Some(block) if at_end => block.len(),
        Some(_) => 0,
        None => return,
    };
    editor
        .write()
        .select(Selection::caret(Position::new(key, at)));
}

/// Takes the text the browser composed into leaf `key` into the model.
fn reconcile(
    mut editor: CopyValue<LiveEditor>,
    key: NodeKey,
    dom: &str,
    changed: &mut impl FnMut(Option<Doc>, bool),
) {
    let Some(old) = editor.peek().doc().get(key).map(|block| block.text()) else {
        return;
    };
    let Some((from, to, inserted)) = text_diff(&old, dom) else {
        return;
    };
    let before = editor.peek().doc().clone();
    editor.write().apply(Record::Merge, |state| {
        state.select(Selection::range(
            Position::new(key, from),
            Position::new(key, to),
        ));
        state.delete_selection();
        state.insert_text(&inserted);
        true
    });
    changed(Some(before), false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_echoes_of_typing_do_not_reset() {
        let mut editor = Editor::new(Doc::new());
        let mut echoes = Echoes::default();
        let mut sent = Vec::new();
        for c in ["a", "b", "c"] {
            editor.type_text(c);
            echoes.record(editor.doc().clone());
            sent.push(editor.doc().clone());
        }
        // The parent echoes each one a keystroke or more late.
        for doc in &sent {
            assert!(!echoes.foreign(editor.doc(), doc));
        }
        assert!(editor.can_undo());
        assert_eq!(editor.doc().plain_text(), "abc");
    }

    #[test]
    fn a_reloaded_doc_is_no_reset_and_a_new_one_is() {
        let mut editor = Editor::new(Doc::new());
        editor.type_text("x");
        let echoes = Echoes::default();
        let json = serde_json::to_string(editor.doc()).unwrap();
        let reloaded: Doc = serde_json::from_str(&json).unwrap();
        assert!(!echoes.foreign(editor.doc(), &reloaded));
        assert!(echoes.foreign(editor.doc(), &Doc::new()));
    }

    #[test]
    fn an_overlay_count_says_its_results_in_each_language() {
        let english = RichTextEditorLabels::ENGLISH;
        let german = RichTextEditorLabels::GERMAN;
        assert_eq!(results_announcement(None, &english), None);
        assert_eq!(
            results_announcement(Some(0), &english).as_deref(),
            Some("No results")
        );
        assert_eq!(
            results_announcement(Some(1), &english).as_deref(),
            Some("1 result")
        );
        assert_eq!(
            results_announcement(Some(3), &english).as_deref(),
            Some("3 results")
        );
        assert_eq!(
            results_announcement(Some(0), &german).as_deref(),
            Some("Keine Ergebnisse")
        );
        assert_eq!(
            results_announcement(Some(1), &german).as_deref(),
            Some("1 Ergebnis")
        );
        assert_eq!(
            results_announcement(Some(2), &german).as_deref(),
            Some("2 Ergebnisse")
        );
    }
}
