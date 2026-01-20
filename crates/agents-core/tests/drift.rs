use std::fs;

use agents_core::model::manifest::BackendKind;
use agents_core::model::{DriftDetection, DriftMethod, StampMethod};
use agents_core::stamps::{apply_stamp, classify, compute_sha256_hex, DriftStatus, StampMeta};

fn meta_for(content_without_stamp: &str) -> StampMeta {
    StampMeta {
        generator: "agents".to_string(),
        adapter_agent_id: "a".to_string(),
        manifest_spec_version: "0.1".to_string(),
        mode: "default".to_string(),
        policy: "safe".to_string(),
        backend: BackendKind::VfsContainer,
        profile: None,
        content_sha256: compute_sha256_hex(content_without_stamp),
    }
}

#[test]
fn classify_missing_unmanaged_clean_drifted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.txt");

    let drift = DriftDetection {
        method: Some(DriftMethod::Sha256),
        stamp: Some(StampMethod::Comment),
    };

    // Missing
    let status = classify(&path, "hi\n", &drift).unwrap();
    assert_eq!(status, DriftStatus::Missing);

    // Unmanaged
    fs::write(&path, "hi\n").unwrap();
    let status = classify(&path, "hi\n", &drift).unwrap();
    assert_eq!(status, DriftStatus::Unmanaged);

    // Clean
    let meta = meta_for("hi\n");
    let stamped = apply_stamp("hi\n", &meta, StampMethod::Comment).unwrap();
    fs::write(&path, stamped).unwrap();
    let status = classify(&path, "hi\n", &drift).unwrap();
    assert_eq!(status, DriftStatus::Clean);

    // Drifted
    let meta2 = meta_for("hi\n");
    let stamped2 = apply_stamp("bye\n", &meta2, StampMethod::Comment).unwrap();
    fs::write(&path, stamped2).unwrap();
    let status = classify(&path, "hi\n", &drift).unwrap();
    assert_eq!(status, DriftStatus::Drifted);
}

#[test]
fn classify_mtime_only_behaves_like_sha256_for_now() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.txt");

    let drift = DriftDetection {
        method: Some(DriftMethod::MtimeOnly),
        stamp: Some(StampMethod::Comment),
    };

    let meta = meta_for("hi\n");
    let stamped = apply_stamp("hi\n", &meta, StampMethod::Comment).unwrap();
    fs::write(&path, stamped).unwrap();

    let status = classify(&path, "hi\n", &drift).unwrap();
    assert_eq!(status, DriftStatus::Clean);

    let status = classify(&path, "bye\n", &drift).unwrap();
    assert_eq!(status, DriftStatus::Drifted);
}
