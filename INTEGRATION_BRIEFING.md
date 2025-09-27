# Meteor Integration Briefing (2025-09-27)

This file summarizes the integration-readiness context for agents working inside the Meteor repository. The original evidence lives outside the repo, so treat this as the authoritative snapshot while you execute the current task tree.

## Current Focus
- Complete the integration readiness tickets recorded in `docs/procs/TASKS.txt` at the top of the file under **Integration Readiness Tickets (2025-09-27)**.
- Coordinate sequencing with the RSB hardening effort (ticket `RSB-BUGS-01`). Meteor work may begin now, but expect to re-run tests once the RSB crate publishes its patched release.

## Ticket Summaries

### MET-BUGS-01 — Reduce High-Risk Unwraps in Runtime (P1)
- Replace panic-prone `unwrap()`/`expect()` calls in runtime, parsers, and CLI code with typed error handling.
- Align Meteor’s error semantics with the REBEL pattern: lower-level functions return `Result`; CLI handlers map errors to exit codes/messages before returning to `dispatch!`.
- Target fewer than 30 remaining unwraps, each justified with inline comments.
- Add regression tests covering malformed meteor strings, CLI failures, and other cases that previously panicked.

### MET-QOL-02 — Normalize Worktree Ahead of Integration (P1)
- Clean the repository: commit or stash the ~10 modified files and the untracked planning doc so `git status` is clean on `main`.
- Document any preserved local artifacts (e.g., via repo notes or README updates) if you intentionally keep them.

### MET-QOL-01 — Resolve Compiler Warnings (P2)
- Drive `cargo build --all-features` to zero warnings.
- If a warning cannot be eliminated, explicitly suppress it and document the rationale (e.g., in code comments or release notes).

## Testing Expectations
- Continue to use `./bin/test.sh` for organized test execution (`./bin/test.sh run sanity`, `./bin/test.sh run uat`, etc.).
- Any new tests must follow the enforced naming and directory conventions described in `meteor/docs/ref/rsb/HOWTO_TEST.md`; mirror that structure under Meteor’s `tests/` hierarchy.

## Downstream Coordination
- After MET-BUGS-01 lands, share the updated error contracts with whoever owns the ProntoDB integration guide (`PDB-PLAN-01`) so future agents can rely on consistent behaviour when integrating Meteor with ProntoDB.
- Note that ProntoDB engineering work is currently deferred until that usage contract is written. Your changes should keep Meteor stable as a library even without ProntoDB validation in flight.

## Reference
- Additional architectural guidance for RSB (function ordinality, module spec, testing policy) lives in the RSB repo under `docs/tech/`. Consult those files if you need deeper rationale. The most relevant documents are:
  - `meteor/docs/ref/rsb/RSB_ARCH.md`
  - `meteor/docs/ref/rsb/MODULE_SPEC.md`
  - `meteor/docs/ref/rsb/HOWTO_TEST.md`
  - `meteor/docs/ref/features/FEATURES_FS.md`

Feel free to reach out to the integration lead if new blockers appear or if you discover additional unwraps that need coordination with other projects.
