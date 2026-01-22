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

- [ ] Define mount layout
  - [ ] Mount point contains full repo content
  - [ ] Overlay generated outputs on top of repo paths
  - [ ] Ensure read/write semantics match policy (likely read-only for v1)

- [ ] Implement backend
  - [ ] Create mount point directory
  - [ ] Start FUSE process and keep it alive
  - [ ] Provide command output telling user which path to open in IDE

- [ ] Cleanup
  - [ ] Unmount reliably on exit
  - [ ] Handle crashes and stale mounts

- [ ] Tests
  - [ ] Unit tests for mount plan logic
  - [ ] Integration tests optional/manual due to environment requirements

## Verification
- [ ] Mounted workspace path shows generated files without writing into repo
