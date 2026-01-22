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

- [x] Select snippets
  - [x] Collect snippet IDs from:
    - [x] resolved mode frontmatter `includeSnippets`
    - [x] scope overrides `includeSnippets`
    - [x] (optional) profile overrides if supported (not supported in v1)
  - [x] Deduplicate and stable-sort snippet IDs
  - [x] Load snippet content from `RepoConfig.prompts.snippets`
  - [x] Define behavior for unknown snippet IDs:
    - [x] default: error (prefer correctness)
    - [x] optionally `--on-missing warn` later (not supported in v1)

- [x] Compose deterministic markdown
  - [x] Decide separator conventions:
    - [x] ensure exactly one blank line between sections
    - [x] include headings or banners only if adapters/templates need them (avoid duplicating)
  - [x] Normalize newlines to `\n`
  - [x] Ensure trailing newline

- [x] Apply redaction
  - [x] Interpret `policy.paths.redact` as glob patterns
  - [x] Define redaction scope:
    - [x] For this feature, only redact referenced file-inclusions if prompt templates support inclusion.
    - [x] If v1 prompt composition does not embed file contents, redaction still applies when generating instruction surfaces that may embed file excerpts.
  - [x] Provide a helper `is_redacted(path) -> bool` for later use
  - [x] Define placeholder string for redacted content (deterministic, no leaks)

- [x] Source map hooks
  - [x] Provide `PromptSource` records listing which files contributed to the composed prompt
  - [x] Keep order stable (base, project, snippets...)

- [x] Tests
  - [x] Unit test: snippet selection ordering is stable
  - [x] Unit test: composed markdown separators are stable
  - [x] Unit test: redaction glob matching

## Verification
- [x] Golden test: same fixture produces byte-identical composed prompt
- [x] `agents preview` (once implemented) uses composed prompt in templates (covered by preview composed prompt test)
