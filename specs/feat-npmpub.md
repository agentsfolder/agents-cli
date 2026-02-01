# feat-npmpub: npm Packaging

Goal: Publish the `agents` CLI to npm so users can `npm install -g agents` and run `agents`.

Depends on: feat-cliapp
Unblocks: distribution for Node/npm users

## Deliverables
- `package.json` with `bin` mapping for `agents`.
- Postinstall script builds the Rust binary and installs it into the npm package.
- Cross-platform wrapper that runs the bundled binary.
- npm ignore list for Rust build artifacts.
- cargo-dist config for prebuilt binaries + npm/homebrew packaging.

## Implementation Plan
- [x] Add `package.json` with metadata, `bin`, and scripts.
- [x] Add install script to build and copy the `agents` binary.
- [x] Add JS wrapper to execute the bundled binary.
- [x] Add `.npmignore` to avoid publishing cargo artifacts.
- [x] Update README with npm install instructions.
- [x] Configure cargo-dist for multi-platform binaries and npm packaging.
- [x] Add GitHub Actions workflow for cargo-dist on tags.
- [ ] Add Homebrew distribution via cargo-dist (deferred for npm-only release).
- [x] Wire npm publish token into dist workflow.
- [x] Align release tag with crate/package version.

## Verification
- [x] `npm pack` includes the wrapper and binary scripts.
- [x] `npm install -g` followed by `agents --help` works (manual).
- [x] `cargo dist build` produces binaries for supported platforms (CI).
- [ ] `git push --tags` triggers cargo-dist release workflow (manual).
