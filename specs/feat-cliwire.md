# feat-cliwire: Wire CLI scaffolding

Goal: Integrate existing CLI scaffolding into execution paths so it is exercised in normal flows.

Depends on: feat-cliapp
Unblocks: production readiness

## Implementation Plan
- [x] Use `Backend::as_str` + `AppContext.quiet` in CLI output flow.
- [x] Use importer discovery + `ImportInputs.source_path` in `agents import`.
- [x] Use runner registry for known agents in `agents run`.
- [x] Use doctor check abstractions (`DoctorCheck`, `CheckResult`, `FixResult`, `DoctorReport::add`).

## Verification
- [x] `cargo test -p agents-cli` passes.
