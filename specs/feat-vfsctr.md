# feat-vfsctr: vfs_container Backend

Goal: Run target agents in a container overlay so generated outputs appear in the workspace without writing to the host repo.

Depends on: feat-outputs, feat-stamps
Unblocks: feat-runner

## Deliverables
- A backend implementation that:
  - generates outputs into a temp dir
  - starts a container with repo mounted
  - injects generated files into expected paths
  - runs the agent entrypoint within the container

## Implementation Plan
- [x] Decide container runtime support
  - [x] Primary: Docker CLI (`docker`)
  - Podman support (future)
  - [x] Implement `DockerRuntime` wrapper:
    - [x] check availability (`docker --version`)
    - [x] check daemon reachable (`docker info`)

- [x] Define runtime contract
  - [x] Workspace mount:
    - [x] host repo -> container `/__agents_repo` (read-only)
    - [x] container creates writable `/workspace` by copying repo contents
  - [x] Generated outputs mount:
    - [x] host temp dir -> container `/__agents_out` (read-only)
  - [x] Injection strategy (pick one and document):
    - [x] A) start container with writable layer and copy outputs to target paths at startup
    - B) mount individual files onto exact target paths (future; cross-platform complexity)
  - [x] Prefer A for portability.

- [x] Container image strategy
  - [x] Decide default image (lightweight with shells + git)
    - [x] default: `alpine:3.19` (requires `sh` + `tar`)
  - [x] Allow override via env/config (future)
    - [x] env: `AGENTS_VFSCTR_IMAGE`
  - [x] Ensure target agent binary is available:
    - [x] Either preinstalled in image (v1 expectation)
    - [x] Document prerequisite: the agent command must exist in the container image

- [x] Command execution
  - [x] Build deterministic `docker run` invocation:
    - [x] set workdir `/workspace`
    - [x] pass env vars required by skills (later)
    - [x] mount repo read-only by default
    - optionally mount a writable scratch for temp files (future)
  - [x] Entry script:
    - [x] copies `/__agents_out/*` into `/workspace` at correct repo-relative paths
    - [x] prints minimal banner if verbose
    - [x] execs agent command

- [x] Policy enforcement hooks (best-effort)
  - [x] If policy denies filesystem writes:
    - [x] make `/workspace` read-only inside container (best-effort chmod)
  - [x] Network restrictions:
    - [x] best-effort: `docker run --network none` when policy disables network
    - [x] document limitations (docker cannot easily restrict per-host without extra setup)
  - [x] Exec restrictions:
    - [x] enforced by wrapper for the commands it runs; agent internal exec is advisory

- [x] Error handling
  - [x] Missing docker => actionable error
  - [x] Failed container start => include stderr, hint to run `docker info`

- [x] Tests
  - [x] Unit tests for docker command building (pure string/args)
  - [x] Integration tests gated behind env var (optional): runs a dummy container

## Verification
- [x] `AGENTS_DOCKER_TESTS=1 cargo test -p agents-core --test vfsctr` runs a docker container and verifies injected files are visible in `/workspace`
