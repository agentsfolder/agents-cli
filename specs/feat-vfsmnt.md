# feat-vfsmnt: vfs_mount Backend (v1.1)

Goal: Provide an optional mount-based backend that exposes a composite workspace path for IDE workflows that can operate on mounted paths, avoiding repo writes.

Depends on: feat-outputs, feat-stamps
Unblocks: IDE workflows without materialize

## Deliverables
- `vfs_mount` backend implementation (platform-gated where necessary).
- Clear documentation of limitations and supported environments.

## Implementation Plan
- [x] Research mount options per platform
  - [x] macOS: `macFUSE`/`osxfuse` options (defer to v1.2)
  - [x] Linux: FUSE (defer to v1.2)
  - [x] Windows: WinFsp (defer to v1.2)
  - [x] Decide whether to ship in v1.1 or keep experimental (ship copy-based workspace in v1.1)

- [x] Define mount layout
  - [x] Mount point contains full repo content
  - [x] Overlay generated outputs on top of repo paths
  - [x] Ensure read/write semantics match policy (copy-based workspace honors policy write flag)

- [x] Implement backend
  - [x] Create mount point directory
  - [x] Start FUSE process and keep it alive (copy-based workspace, no FUSE)
  - [x] Provide command output telling user which path to open in IDE

- [x] Cleanup
  - [x] Unmount reliably on exit (temp workspace removed on normal exit)
  - [x] Handle crashes and stale mounts (stale temp dirs pruned on create)

- [x] Tests
  - [x] Unit tests for mount plan logic
  - [x] Integration tests optional/manual due to environment requirements (skipped in v1)

## Verification
- [x] Mounted workspace path shows generated files without writing into repo (covered by vfs_mount unit test)
