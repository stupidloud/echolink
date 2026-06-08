# Echolink Agent Rules

## Code Change Safety
- Before any Rust change, read 20x the amount of changed code in surrounding context to check ownership, borrows, and lifetimes.
- Frontend changes: run `npm test` before committing.
- Rust ownership: after modifying a variable inside a closure, verify it's still valid outside.

## CI Workflow
- Set repo public BEFORE pushing.
- Wait for CI jobs to actually start (steps > 0) before switching back to private.
- Never cancel a running CI unless user asks.
