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
- [ ] Decide container runtime support
  - [ ] Primary: Docker CLI (`docker`)
  - [ ] Optional future: Podman
  - [ ] Implement `DockerRuntime` wrapper:
    - [ ] check availability (`docker --version`)
    - [ ] check daemon reachable (`docker info`)

- [ ] Define runtime contract
  - [ ] Workspace mount:
    - [ ] host repo -> container `/workspace`
  - [ ] Generated outputs mount:
    - [ ] host temp dir -> container `/__agents_out`
  - [ ] Injection strategy (pick one and document):
    - [ ] A) start container with writable layer and copy outputs to target paths at startup
    - [ ] B) mount individual files onto exact target paths (harder cross-platform)
  - [ ] Prefer A for portability.

- [ ] Container image strategy
  - [ ] Decide default image (lightweight with shells + git)
  - [ ] Allow override via env/config (future)
  - [ ] Ensure target agent binary is available:
    - [ ] Either preinstalled in image OR mounted from host
    - [ ] For v1, simplest: require agent binary present on host and run on host? (conflicts with container)
    - [ ] If container must run the agent, define how agent is installed (document prerequisite)

- [ ] Command execution
  - [ ] Build deterministic `docker run` invocation:
    - [ ] set workdir `/workspace`
    - [ ] pass env vars required by skills (later)
    - [ ] mount repo read-only by default
    - [ ] optionally mount a writable scratch for temp files
  - [ ] Entry script:
    - [ ] copies `/__agents_out/*` into `/workspace` at correct repo-relative paths
    - [ ] prints minimal banner if verbose
    - [ ] execs agent command

- [ ] Policy enforcement hooks (best-effort)
  - [ ] If policy denies filesystem writes:
    - [ ] mount `/workspace` read-only
  - [ ] Network restrictions:
    - [ ] document limitations (docker cannot easily restrict per-host without extra setup)
  - [ ] Exec restrictions:
    - [ ] enforced by wrapper for the commands it runs; agent internal exec is advisory

- [ ] Error handling
  - [ ] Missing docker => actionable error
  - [ ] Failed container start => include stderr, hint to run `docker info`

- [ ] Tests
  - [ ] Unit tests for docker command building (pure string/args)
  - [ ] Integration tests gated behind env var (optional): runs a dummy container

## Verification
- [ ] `agents run <agent> --backend vfs_container` starts docker container and agent sees injected files
