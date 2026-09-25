use crate::components::form::rich_text_editor::model::{BlockKind, Doc, NodeKey};

#[test]
fn json_round_trips() {
    let doc = Doc::from_markdown(
        "# T\n\n**b** *i* <u>u</u> ~~s~~ `c` [l](https://a.example \"t\")\\\nx\n\n- a\n  - b\n\n7. c\n\n> q\n\n---\n\n```rust\nfn x() {}\n```",
    );
    let json = serde_json::to_string(&doc).unwrap();
    let back: Doc = serde_json::from_str(&json).unwrap();
    assert_eq!(back.rekeyed(), doc.rekeyed());
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
fn loading_repairs_the_structure() {
    let json = r#"{"blocks":[
        {"type":"heading","level":9,"inlines":[{"type":"text","text":"h"}]},
        {"type":"list","ordered":false,"children":[{"type":"paragraph"}]},
        {"type":"quote"}
    ]}"#;
    let doc: Doc = serde_json::from_str(json).unwrap();
    assert_eq!(doc.blocks[0].kind, BlockKind::heading(6));
    assert_eq!(doc.blocks[1].children()[0].kind, BlockKind::ListItem);
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
