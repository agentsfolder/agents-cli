# feat-prompts: Prompt Composition + Redaction

Goal: Compose canonical prompt text from `.agents/prompts/**` according to the resolved mode/scope/profile configuration, applying redaction rules and producing a deterministic prompt bundle for adapters.

Depends on: feat-resolv
Unblocks: feat-templ, feat-shared

## Deliverables
- `PromptComposer` that returns `EffectivePrompts` including:
  - base prompt
  - project prompt
  - selected snippet texts in stable order
  - a single composed "instructions" text (if needed by templates)
- Redaction applied according to effective policy.

## Implementation Plan
- [x] Define prompt structures
  - [x] `PromptId` (string) for snippets
  - [x] `EffectivePrompts`:
    - [x] `base_md: String`
    - [x] `project_md: String`
    - [x] `snippets: Vec<Snippet { id, path, md }>`
    - [x] `composed_md: String` (base + project + snippets)

- [ ] Select snippets
  - [ ] Collect snippet IDs from:
    - [ ] resolved mode frontmatter `includeSnippets`
    - [ ] scope overrides `includeSnippets`
    - [ ] (optional) profile overrides if supported
  - [ ] Deduplicate and stable-sort snippet IDs
  - [ ] Load snippet content from `RepoConfig.prompts.snippets`
  - [ ] Define behavior for unknown snippet IDs:
    - [ ] default: error (prefer correctness)
    - [ ] optionally `--on-missing warn` later

- [ ] Compose deterministic markdown
  - [ ] Decide separator conventions:
    - [ ] ensure exactly one blank line between sections
    - [ ] include headings or banners only if adapters/templates need them (avoid duplicating)
  - [ ] Normalize newlines to `\n`
  - [ ] Ensure trailing newline

- [ ] Apply redaction
  - [ ] Interpret `policy.paths.redact` as glob patterns
  - [ ] Define redaction scope:
    - [ ] For this feature, only redact referenced file-inclusions if prompt templates support inclusion.
    - [ ] If v1 prompt composition does not embed file contents, redaction still applies when generating instruction surfaces that may embed file excerpts.
  - [ ] Provide a helper `is_redacted(path) -> bool` for later use
  - [ ] Define placeholder string for redacted content (deterministic, no leaks)

- [ ] Source map hooks
  - [ ] Provide `PromptSource` records listing which files contributed to the composed prompt
  - [ ] Keep order stable (base, project, snippets...)

- [ ] Tests
  - [ ] Unit test: snippet selection ordering is stable
  - [ ] Unit test: composed markdown separators are stable
  - [ ] Unit test: redaction glob matching

## Verification
- [ ] Golden test: same fixture produces byte-identical composed prompt
- [ ] `agents preview` (once implemented) uses composed prompt in templates
