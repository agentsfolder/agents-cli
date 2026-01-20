use agents_core::model::manifest::BackendKind;
use agents_core::model::StampMethod;
use agents_core::stamps::{
    apply_stamp, parse_stamp, stamp_rendered_output, strip_existing_stamp, StampMeta,
};

fn meta_for(hash: &str) -> StampMeta {
    StampMeta {
        generator: "agents".to_string(),
        adapter_agent_id: "a".to_string(),
        manifest_spec_version: "0.1".to_string(),
        mode: "default".to_string(),
        policy: "safe".to_string(),
        backend: BackendKind::VfsContainer,
        profile: Some("ci".to_string()),
        content_sha256: hash.to_string(),
    }
}

#[test]
fn comment_stamp_round_trip_and_idempotent() {
    let meta = meta_for("abc");
    let content = "hello\nworld\n";

    let stamped = apply_stamp(content, &meta, StampMethod::Comment).unwrap();
    let parsed = parse_stamp(&stamped).unwrap();
    assert_eq!(parsed.meta, meta);

    let (stripped, found) = strip_existing_stamp(&stamped);
    assert_eq!(stripped, content);
    assert_eq!(found.unwrap().meta, meta);

    let stamped2 = stamp_rendered_output(&stamped, &meta, StampMethod::Comment).unwrap();
    assert_eq!(stamped2, stamped);
}

#[test]
fn frontmatter_stamp_round_trip_and_idempotent() {
    let meta = meta_for("abc");

    // No existing frontmatter.
    let content = "hello\n";
    let stamped = apply_stamp(content, &meta, StampMethod::Frontmatter).unwrap();
    let parsed = parse_stamp(&stamped).unwrap();
    assert_eq!(parsed.meta, meta);

    let (stripped, _) = strip_existing_stamp(&stamped);
    assert_eq!(stripped, content);

    let stamped2 = stamp_rendered_output(&stamped, &meta, StampMethod::Frontmatter).unwrap();
    assert_eq!(stamped2, stamped);

    // Existing frontmatter should be preserved.
    let content2 = "---\ntitle: hi\n---\nbody\n";
    let stamped3 = apply_stamp(content2, &meta, StampMethod::Frontmatter).unwrap();
    let (stripped2, _) = strip_existing_stamp(&stamped3);
    assert_eq!(stripped2, content2);
}

#[test]
fn json_field_stamp_round_trip_and_idempotent_single_line() {
    let meta = meta_for("abc");
    let content = "{\"a\":1}";

    let stamped = apply_stamp(content, &meta, StampMethod::JsonField).unwrap();
    let parsed = parse_stamp(&stamped).unwrap();
    assert_eq!(parsed.meta, meta);

    let (stripped, _) = strip_existing_stamp(&stamped);
    assert_eq!(stripped.replace(" ", ""), content.replace(" ", ""));

    let stamped2 = stamp_rendered_output(&stamped, &meta, StampMethod::JsonField).unwrap();
    assert_eq!(stamped2, stamped);
}

#[test]
fn json_field_stamp_round_trip_and_idempotent_multiline() {
    let meta = meta_for("abc");
    let content = "{\n  \"a\": 1\n}\n";

    // A tiny JSONC-style comment should be tolerated by insertion/removal.
    let content_jsonc = "{\n  // comment\n  \"a\": 1\n}\n";

    let stamped = apply_stamp(content, &meta, StampMethod::JsonField).unwrap();
    let parsed = parse_stamp(&stamped).unwrap();
    assert_eq!(parsed.meta, meta);

    let (stripped, _) = strip_existing_stamp(&stamped);
    assert_eq!(stripped, content);

    let stamped2 = stamp_rendered_output(&stamped, &meta, StampMethod::JsonField).unwrap();
    assert_eq!(stamped2, stamped);

    // JSONC: insertion should ignore comment and still work.
    let stamped_jsonc = apply_stamp(content_jsonc, &meta, StampMethod::JsonField).unwrap();
    let parsed_jsonc = parse_stamp(&stamped_jsonc).unwrap();
    assert_eq!(parsed_jsonc.meta, meta);
}
