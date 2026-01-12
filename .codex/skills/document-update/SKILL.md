---
name: document-update
description: Update documentation to match code and design changes in this repo. Use when code or design changes cause drift in Rust doc comments (module-level //! and item-level ///), non-doc comments, and README.md. Focus on intent, rationale, invariants, and relationships between major symbols rather than restating obvious names or signatures.
---

# Document Update

## Overview
Keep docs and comments aligned with the current code by updating Rust doc comments, non-doc comments, and README without changing behavior or APIs.

## Workflow

### 1) Establish scope and sources of truth
- Identify which files changed or are likely out of sync (Rust modules, public types/functions, README).
- Treat current code as the source of truth; do not change code while updating docs.
- If doc changes imply code changes are needed, stop and ask.

### 2) Read existing tone and structure
- Read the local section/module/file to match tone, language, and formatting.
- If the file mixes languages, prefer the surrounding section’s language.
- Keep README concise; do not inflate with speculative content.

### 3) Update Rust doc comments
- **Module-level `//!`**: Describe the module’s role, then explain relationships and differences between its major symbols (types/functions/modules) and how they fit together.
- **Item-level `///`**: Explain design intent, invariants, constraints, tradeoffs, and non-obvious behavior. Avoid repeating obvious names, fields, or signatures.
- Add/adjust examples only when they clarify non-obvious usage or invariants.

### 4) Update non-doc comments (`//`, `/* */`)
- Use comments for rationale and high-level flow of complex logic.
- Avoid restating line-by-line behavior; prefer intent and “why.”
- Remove or rewrite comments that are now misleading or redundant.

### 5) Update README
- Ensure summary, usage pointers, and examples align with current API and behavior.
- Keep the README brief; link to docs.rs or other references when appropriate.

### 6) Final consistency check
- No code changes.
- No stale terminology or outdated file/module names.
- Comments do not contradict each other or the code.

## Quality Checklist
- Doc comments explain intent and invariants, not just “what.”
- Module docs explain relationships/differences between key symbols.
- Non-doc comments describe rationale or flow for complex sections.
- README aligns with current public API and behavior.
- Language and tone match the existing file.

## References
- `references/doc-style.md` for project-specific tone and conventions.
