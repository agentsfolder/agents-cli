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
- [x] Define compatibility data sources
  - [x] Adapter YAML:
    - [x] backendDefaults
    - [x] outputs list
    - [x] capabilityMapping (optional)
  - [x] Core knowledge base:
    - [x] per-agent limitations (hardcoded doc strings for v1)

- [x] Implement matrix builder
  - [x] Iterate enabled adapters
  - [x] Extract:
    - [x] output paths
    - [x] logical surfaces
    - [x] backend defaults
  - [x] Summarize enforceability:
    - [x] vfs_container: filesystem enforceable via mounts
    - [x] network restrictions: advisory
    - [x] exec restrictions: limited

- [x] Render output
  - [x] Human-readable table with stable ordering
  - [x] `--json` output optional

- [x] Tests
  - [x] Snapshot test for compat output (stable ordering)

## Verification
- [x] `agents compat` includes all enabled adapters and stable output (covered by compat snapshot test)
