use std::fs;

use agents_core::vfsmnt::{create_workspace, OverlayFile, VfsMountOptions};

#[test]
fn vfs_mount_workspace_overlays_outputs() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_root = tmp.path().join("repo");
    fs::create_dir_all(repo_root.join("dir")).unwrap();
    fs::write(repo_root.join("dir/file.txt"), "repo").unwrap();

    let overlays = vec![OverlayFile {
        rel_path: "out/gen.txt".to_string(),
        bytes: b"generated".to_vec(),
    }];

    let workspace = create_workspace(
        &repo_root,
        &overlays,
        &VfsMountOptions {
            deny_writes: false,
            verbose: false,
        },
    )
    .unwrap();

    let base = fs::read_to_string(workspace.path().join("dir/file.txt")).unwrap();
    assert_eq!(base, "repo");

    let overlay = fs::read_to_string(workspace.path().join("out/gen.txt")).unwrap();
    assert_eq!(overlay, "generated");

    assert!(!repo_root.join("out/gen.txt").is_file());
}
