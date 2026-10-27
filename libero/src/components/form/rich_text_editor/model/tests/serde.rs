use crate::components::form::rich_text_editor::model::{BlockKind, Doc, EditorState, NodeKey};

#[test]
fn json_round_trips() {
    let doc = Doc::from_markdown(
        "# T\n\n**b** *i* <u>u</u> ~~s~~ `c` [l](https://a.example \"t\")\\\nx\n\n- a\n  - b\n\n7. c\n\n> q\n\n---\n\n```rust\nfn x() {}\n```",
    );
    let json = serde_json::to_string(&doc).unwrap();
    let back: Doc = serde_json::from_str(&json).unwrap();
    assert_eq!(back.rekeyed(), doc.rekeyed());
}

/// An inserted rule equals its reload, so an echoed value is no reset (todo 4484).
#[test]
fn an_inserted_rule_equals_its_reload() {
    let mut state = EditorState::new(Doc::from_markdown("a"));
    assert!(state.insert_rule());
    let back: Doc = serde_json::from_str(&serde_json::to_string(&state.doc).unwrap()).unwrap();
    assert_eq!(back, state.doc);
    assert_eq!(Doc::from_markdown(&state.doc.to_markdown()), state.doc);
}

#[test]
fn json_is_compact_and_has_no_keys() {
    let doc = Doc::from_markdown("**hi**\n\n- a");
    let json = serde_json::to_value(&doc).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "version": 1,
            "blocks": [
                {
                    "type": "paragraph",
                    "inlines": [{"type": "text", "text": "hi", "marks": [{"type": "bold"}]}],
                },
                {
                    "type": "list",
                    "ordered": false,
                    "children": [{"type": "list_item", "children": [
                        {"type": "paragraph", "inlines": [{"type": "text", "text": "a"}]},
                    ]}],
                },
            ],
        })
    );
}

#[test]
fn task_items_store_checked_and_old_items_load_as_plain() {
    let old = r#"{"blocks":[{"type":"list","ordered":false,"children":[
        {"type":"list_item","children":[{"type":"paragraph","inlines":[{"type":"text","text":"a"}]}]}
    ]}]}"#;
    let doc: Doc = serde_json::from_str(old).unwrap();
    assert_eq!(doc.blocks[0].children()[0].kind, BlockKind::list_item());

    let doc = Doc::from_markdown("- [ ] a\n- [x] b\n- c");
    let items: Vec<_> = serde_json::to_value(&doc).unwrap()["blocks"][0]["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["checked"].clone())
        .collect();
    assert_eq!(
        items,
        [
            serde_json::json!(false),
            serde_json::json!(true),
            serde_json::Value::Null
        ]
    );
    let back: Doc = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
    assert_eq!(back.rekeyed(), doc.rekeyed());
}

#[test]
fn loading_gives_fresh_keys_and_repairs_runs_and_emptiness() {
    let json = r#"{"blocks":[
        {"type":"paragraph","inlines":[{"type":"text","text":"a"},{"type":"text","text":"b"},{"type":"text","text":""}]},
        {"type":"heading","level":2}
    ]}"#;
    let mut doc: Doc = serde_json::from_str(json).unwrap();
    assert_eq!(doc.version, Doc::VERSION);
    assert_eq!(doc.blocks[0].inlines().len(), 1);
    assert_ne!(doc.blocks[0].key, doc.blocks[1].key);
    let fresh = doc.key();
    assert!(
        doc.leaves().iter().all(|key| *key != fresh),
        "new keys never collide"
    );

    let empty: Doc = serde_json::from_str("{}").unwrap();
    assert_eq!(empty.blocks.len(), 1);
    assert_eq!(empty.blocks[0].kind, BlockKind::Paragraph);
}

#[test]
fn a_stored_heading_level_loads_clamped_and_writes_markdown() {
    let json = r#"{"blocks":[
        {"type":"heading","level":0,"inlines":[{"type":"text","text":"low"}]},
        {"type":"heading","level":7,"inlines":[{"type":"text","text":"high"}]}
    ]}"#;
    let doc: Doc = serde_json::from_str(json).unwrap();
    assert_eq!(doc.blocks[0].kind, BlockKind::heading(1));
    assert_eq!(doc.blocks[1].kind, BlockKind::heading(6));
    assert_eq!(doc.to_markdown(), "# low\n\n###### high\n");
}

#[test]
fn loading_repairs_the_structure() {
    let json = r#"{"blocks":[
        {"type":"heading","level":9,"inlines":[{"type":"text","text":"h"}]},
        {"type":"list","ordered":false,"children":[{"type":"paragraph"}]},
        {"type":"quote"}
    ]}"#;
    let doc: Doc = serde_json::from_str(json).unwrap();
    assert_eq!(doc.blocks[0].kind, BlockKind::heading(6));
    assert_eq!(doc.blocks[1].children()[0].kind, BlockKind::list_item());
    assert_eq!(
        doc.blocks[1].children()[0].children()[0].kind,
        BlockKind::Paragraph
    );
    assert_eq!(doc.blocks[2].children()[0].kind, BlockKind::Paragraph);
    assert_eq!(doc.leaves().len(), 3);
}

#[test]
fn custom_blocks_load_without_their_registry() {
    let json =
        r#"{"blocks":[{"type":"custom","name":"embed","attrs":{"url":"x"},"content":"atom"}]}"#;
    let doc: Doc = serde_json::from_str(json).unwrap();
    assert_eq!(doc.leaves(), vec![NodeKey(1)]);
    assert_eq!(
        serde_json::to_string(&doc).unwrap(),
        format!(r#"{{"version":1,{}"#, &json[1..])
    );
}

/// A stored item outside a list is wrapped in one, so Shift+Tab finds a list (todo 4274).
#[test]
fn a_root_list_item_loads_in_a_list_and_outdents() {
    let json = r#"{"blocks":[{"type":"list_item","children":[{"type":"paragraph","inlines":[{"type":"text","text":"a"}]}]},{"type":"paragraph","inlines":[{"type":"text","text":"b"}]}]}"#;
    let doc: Doc = serde_json::from_str(json).unwrap();
    assert_eq!(doc.to_markdown(), "- a\n\nb\n");
    let mut state = EditorState::new(doc);
    assert!(state.outdent());
    assert_eq!(state.doc.to_markdown(), "a\n\nb\n");
    let quoted = r#"{"blocks":[{"type":"quote","children":[{"type":"list_item","children":[]}]}]}"#;
    let doc: Doc = serde_json::from_str(quoted).unwrap();
    assert!(matches!(
        doc.blocks[0].children()[0].kind,
        BlockKind::List { .. }
    ));
}

/// Content in the field its kind does not hold still loads (todo 4276).
#[test]
fn content_in_the_other_field_is_kept() {
    let quote = r#"{"blocks":[{"type":"quote","inlines":[{"type":"text","text":"q"}]}]}"#;
    let doc: Doc = serde_json::from_str(quote).unwrap();
    assert_eq!(doc.to_markdown(), "> q\n");
    let both = r#"{"blocks":[{"type":"paragraph","inlines":[{"type":"text","text":"a"}],"children":[{"type":"paragraph","inlines":[{"type":"text","text":"b"}]}]}]}"#;
    let doc: Doc = serde_json::from_str(both).unwrap();
    assert_eq!(doc.blocks[0].text(), "a\nb");
    let heading = r#"{"blocks":[{"type":"heading","level":2,"children":[{"type":"paragraph","inlines":[{"type":"text","text":"h"}]}]}]}"#;
    let doc: Doc = serde_json::from_str(heading).unwrap();
    assert_eq!(doc.to_markdown(), "## h\n");
}

/// An empty stored list gets an item, not a bare paragraph (todo 4277).
#[test]
fn an_empty_list_loads_with_an_item() {
    let doc: Doc = serde_json::from_str(r#"{"blocks":[{"type":"list","ordered":false}]}"#).unwrap();
    let item = &doc.blocks[0].children()[0];
    assert!(item.kind.is_list_item());
    assert_eq!(item.children()[0].kind, BlockKind::Paragraph);
    let mut state = EditorState::new(doc);
    assert!(state.outdent());
}
