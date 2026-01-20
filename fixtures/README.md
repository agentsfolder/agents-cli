# Adapter Fixture Format

Each fixture lives under `fixtures/<name>/`.

- `fixtures/<name>/repo/**`
  - A self-contained repository root used as input.
  - Must include a `.agents/` tree and any other files required for rendering.

- `fixtures/<name>/expect/<agent-id>/**`
  - The expected rendered outputs (repo-relative paths) for that adapter.

Optional:

- `fixtures/<name>/matrix.yaml`
  - A future extension describing multiple (mode/profile/backend) cases.
  - When present, expected outputs should be placed under:
    - `fixtures/<name>/expect/<agent-id>/<case-name>/**`
