# Document Style Notes (typing_engine)

## Tone and language
- Match the surrounding file’s language and tone.
- README is concise and English; keep it short and factual.
- Some inline comments are Japanese; keep Japanese where the surrounding block is Japanese.

## Rust doc comment focus
- Prefer intent, invariants, and constraints over restating names/signatures.
- For module docs (`//!`): describe the module’s role and how key symbols relate or differ.
- For item docs (`///`): explain rationale, tradeoffs, and non-obvious behavior.

## Non-doc comments
- Explain “why” or the high-level flow in complex logic.
- Avoid line-by-line narration of obvious code.

## Safety
- Do not change code or behavior while updating documentation.
- If an accurate doc update requires a code change, stop and ask.
