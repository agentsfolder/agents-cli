# feat-compat: `agents compat` (Compatibility Matrix)

Goal: Print a compatibility matrix per agent adapter including supported surfaces, recommended backend, and policy enforceability coverage.

Depends on: feat-skillpl, feat-outputs
Unblocks: onboarding UX

## Deliverables
- `agents compat` prints a table/matrix:
  - agentId
  - required outputs
  - preferred/fallback backend
  - enforcement notes (exec/network/filesystem) and known limitations

## Implementation Plan
- [ ] Define compatibility data sources
  - [ ] Adapter YAML:
    - [ ] backendDefaults
    - [ ] outputs list
    - [ ] capabilityMapping (optional)
  - [ ] Core knowledge base:
    - [ ] per-agent limitations (hardcoded doc strings for v1)

- [ ] Implement matrix builder
  - [ ] Iterate enabled adapters
  - [ ] Extract:
    - [ ] output paths
    - [ ] logical surfaces
    - [ ] backend defaults
  - [ ] Summarize enforceability:
    - [ ] vfs_container: filesystem enforceable via mounts
    - [ ] network restrictions: advisory
    - [ ] exec restrictions: limited

- [ ] Render output
  - [ ] Human-readable table with stable ordering
  - [ ] `--json` output optional

- [ ] Tests
  - [ ] Snapshot test for compat output (stable ordering)

## Verification
- [ ] `agents compat` includes all enabled adapters and stable output
