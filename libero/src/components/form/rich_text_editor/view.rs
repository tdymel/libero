//! [`RichTextEditor`]: the model's doc rendered into a `contenteditable`, every edit
//! routed through the model (web `beforeinput`), composition reconciled after it ends.

use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use super::input::{Intent, intent, text_diff};
use super::model::{
    Builtin, Commands, Doc, Editor, EditorState, KeyPress, Keymap, MarkKind, NodeKey, Position,
    Record, Selection, UndoStack,
};
use super::offsets::{to_dom, to_model};
use super::render::{RenderCtx, blocks};
use super::surface::{ROOT_ATTR, Report, Surface};
use crate::{
    components::{
        buttons::{ActionIcon, Toolbar, ToolbarGroup, ToolbarSeparator},
        common::{HtmlTag, Input},
        form::{field_props, use_bound, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::{HistoryHandle, UndoHistory, use_element, use_history, use_localization, use_theme},
    platform::mod_is_meta,
    sx::{StaticSx, sx},
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
        .selector(
            "& [data-fence]",
            sx().font_family("monospace")
                .color("text-dimmed")
                .user_select("none"),
        )
        .selector(
            "&[data-empty]::before",
            sx().content("attr(data-placeholder)")
                .color("text-placeholder")
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
    if *keymap.peek() != props.keymap {
        let mut keymap = keymap;
        keymap.set(props.keymap.clone());
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

    let surface = use_hook(|| {
        Surface::start(token.clone(), move |report: Report| {
            if let Some((anchor_key, anchor, head_key, head)) = report.selection {
                if *syncing.peek() || *composing.peek() {
                    return;
                }
                let selection = {
                    let live = editor.peek();
                    model_position(&live, anchor_key, anchor)
                        .zip(model_position(&live, head_key, head))
                        .map(|(anchor, head)| Selection::range(anchor, head))
                };
                if let Some(selection) = selection
                    && selection != editor.peek().state().selection
                {
                    editor.write().select(selection);
                    revision += 1;
                }
            }
            if report.synced {
                syncing.set(false);
            }
            if let Some((key, text)) = report.text {
                reconcile(editor, NodeKey(key), &text, &mut changed);
                generation += 1;
            }
        })
    });

    use_effect(move || {
        let (_, focus) = placed();
        let live = editor.peek();
        let selection = live.state().selection;
        surface.select(
            dom_position(&live, selection.anchor),
            dom_position(&live, selection.head),
            focus,
        );
    });

    let _ = revision();
    let live = editor.read();
    let state = live.state();
    let caret = state.caret().block;
    let source_code = (focused() && state.block(caret).kind.is_code()).then_some(caret);
    let on_code = editable.then(|| {
        EventHandler::new(move |key: NodeKey| {
            let at = editor.peek().doc().get(key).map_or(0, |block| block.len());
            editor
                .write()
                .select(Selection::caret(Position::new(key, at)));
            focused.set(true);
            changed(None, true);
        })
    });
    let content = blocks(
        &live.doc().blocks,
        RenderCtx {
            source_code,
            on_code,
        },
    );
    let empty = is_empty(live.doc());
    let marks = [
        MarkKind::Bold,
        MarkKind::Italic,
        MarkKind::Underline,
        MarkKind::Strike,
        MarkKind::Code,
    ]
    .map(|kind| (kind, state.is_active(kind)));
    let active = move |kind: MarkKind| marks.iter().find(|(k, _)| *k == kind).map(|(_, on)| *on);
    let list = state.list_kind();
    let in_code = state.block_kind().is_code();
    let quote = state.in_quote();
    let (can_undo, can_redo) = (live.can_undo(), live.can_redo());
    drop(live);

    let field = use_field()
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
    let frame = use_field_frame()
        .states(field.states())
        .multiline()
        .prepare();
    let control = use_box()
        .framework_sx(&SURFACE_SX)
        .focus_ring(false)
        .prepare();
    let element = use_element();
    let apple = mod_is_meta();

    let onkeydown = move |event: KeyboardEvent| {
        // Not `is_composing()`: Gboard keeps a composing region open over typed words.
        if *composing.peek() {
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
        let ran = edit(
            &|live| live.handle_key(&keymap.peek(), &commands.peek(), &press, apple),
            false,
        );
        if ran {
            event.prevent_default();
        }
    };

    let onbeforeinput = move |event: Event<BeforeInputData>| {
        let data = event.data();
        match intent(&data.input_type().to_string(), data.data()) {
            Intent::Pass => composing.set(true),
            Intent::Cancel => event.prevent_default(),
            Intent::Type(text) => {
                event.prevent_default();
                edit(&|live| live.type_text(&text), false);
            }
            Intent::Run(builtin) => {
                event.prevent_default();
                edit(&|live| live.run(&commands.peek(), builtin), false);
            }
        }
    };

    let onpaste = move |event: ClipboardEvent| {
        event.prevent_default();
        let Some(text) = event.data().data_transfer().get_as_text() else {
            return;
        };
        edit(
            &|live| live.apply(Record::Step, |state| paste(state, &text)),
            false,
        );
    };

    // Gboard opens composing regions over words the model typed; only text the browser
    // composed itself (`insertCompositionText`) is read back, or a lagging DOM undoes keys.
    let oncompositionend = move |_: CompositionEvent| {
        if *composing.peek() {
            composing.set(false);
            surface.read(editor.peek().state().caret().block.0);
        }
    };

    let surface_element = field
        .aria(control)
        .element(&element)
        .attr(ROOT_ATTR, token.clone())
        .attr("role", "textbox")
        .attr("aria-multiline", "true")
        .attr("aria-readonly", readonly)
        .attr("aria-placeholder", props.placeholder.clone())
        .attr("data-placeholder", props.placeholder.clone())
        .attr("data-empty", empty)
        .attr("contenteditable", if editable { "true" } else { "false" })
        .attr("spellcheck", "false")
        .event("onkeydown", editable.then_some(onkeydown))
        .event("onbeforeinput", editable.then_some(onbeforeinput))
        .event("onpaste", editable.then_some(onpaste))
        .event("oncompositionend", editable.then_some(oncompositionend))
        .event("onfocusin", move |_: FocusEvent| focused.set(true))
        .event("onfocusout", move |_: FocusEvent| focused.set(false))
        .render(
            HtmlTag::Div,
            props.attributes,
            rsx! {
                for generation in [generation()] {
                    div { key: "{generation}", style: "display: contents", {content.clone()} }
                }
            },
        );

    let run = move |builtin: Builtin| {
        move |_: MouseEvent| {
            edit(&|live| live.run(&commands.peek(), builtin), true);
        }
    };
    // Pressing a button must not take focus or the selection from the text.
    let keep = |event: MouseEvent| event.prevent_default();
    let words = use_localization().rich_text_editor;
    let toolbar = (props.toolbar && editable).then(|| {
        rsx! {
            Toolbar { "aria-label": words.toolbar, focus_from: element, onmousedown: keep,
                ToolbarGroup { "aria-label": words.marks,
                    ActionIcon { aria_label: words.bold, selected: active(MarkKind::Bold), onclick: run(Builtin::Bold), icon: pictogram_icons_lucide::bold::outlined }
                    ActionIcon { aria_label: words.italic, selected: active(MarkKind::Italic), onclick: run(Builtin::Italic), icon: pictogram_icons_lucide::italic::outlined }
                    ActionIcon { aria_label: words.underline, selected: active(MarkKind::Underline), onclick: run(Builtin::Underline), icon: pictogram_icons_lucide::underline::outlined }
                    ActionIcon { aria_label: words.strike, selected: active(MarkKind::Strike), onclick: run(Builtin::Strike), icon: pictogram_icons_lucide::strikethrough::outlined }
                    ActionIcon { aria_label: words.code, selected: active(MarkKind::Code), onclick: run(Builtin::Code), icon: pictogram_icons_lucide::code::outlined }
                }
                ToolbarSeparator {}
                ToolbarGroup { "aria-label": words.blocks,
                    ActionIcon { aria_label: words.bullet_list, selected: Some(list == Some(false)), onclick: run(Builtin::BulletList), icon: pictogram_icons_lucide::list::outlined }
                    ActionIcon { aria_label: words.ordered_list, selected: Some(list == Some(true)), onclick: run(Builtin::OrderedList), icon: pictogram_icons_lucide::list_ordered::outlined }
                    ActionIcon { aria_label: words.quote, selected: Some(quote), onclick: run(Builtin::Quote), icon: pictogram_icons_lucide::text_quote::outlined }
                    ActionIcon { aria_label: words.code_block, selected: Some(in_code), onclick: run(Builtin::CodeBlock), icon: pictogram_icons_lucide::square_code::outlined }
                }
                ToolbarSeparator {}
                ToolbarGroup { "aria-label": words.history,
                    ActionIcon { aria_label: words.undo, disabled: !can_undo, onclick: run(Builtin::Undo), icon: pictogram_icons_lucide::undo_2::outlined }
                    ActionIcon { aria_label: words.redo, disabled: !can_redo, onclick: run(Builtin::Redo), icon: pictogram_icons_lucide::redo_2::outlined }
                }
            }
        }
    });

    field.render(rsx! {
        {toolbar}
        {frame.render(surface_element)}
    })
}

/// Plain text at the caret; each line after the first starts a new block.
fn paste(state: &mut EditorState, text: &str) -> bool {
    let text = text.replace("\r\n", "\n");
    let mut changed = state.delete_selection();
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            changed |= state.split_block();
        }
        changed |= state.insert_text(line);
    }
    changed
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
}
