# feat-npmpub: npm Packaging

Goal: Publish the `agents` CLI to npm so users can `npm install -g agents` and run `agents`.

Depends on: feat-cliapp
Unblocks: distribution for Node/npm users

## Deliverables
- `package.json` with `bin` mapping for `agents`.
- Postinstall script builds the Rust binary and installs it into the npm package.
- Cross-platform wrapper that runs the bundled binary.
- npm ignore list for Rust build artifacts.

## Implementation Plan
- [x] Add `package.json` with metadata, `bin`, and scripts.
- [x] Add install script to build and copy the `agents` binary.
- [x] Add JS wrapper to execute the bundled binary.
- [x] Add `.npmignore` to avoid publishing cargo artifacts.
- [x] Update README with npm install instructions.

## Verification
- [x] `npm pack` includes the wrapper and binary scripts.
- [x] `npm install -g` followed by `agents --help` works (manual).
