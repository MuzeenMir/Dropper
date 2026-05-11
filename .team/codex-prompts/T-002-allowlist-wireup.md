# Codex Prompt-Block — T-002 Allowlist Wire-Up

**Paste into a separate Codex terminal in `/mnt/c/Projects/sentinel`.**
**Standing brief:** if this is the first prompt of a new Codex session, paste `.team/CODEX-BRIEF.md` first (or, in this repo, this prompt is self-contained — proceed directly).

---

You are Kai, Lead Developer on the Dropper project (open-source Windows DNS shield, Rust). Marcus (CTO) has written the plan and ticket below. Implement it.

## Ticket

- **Ticket**: T-002 — Wire `AllowList` into `Resolver` and block-page decision handler.
- **Priority**: P0. Blocks v0.1 user-product readiness.
- **Plan**: `docs/plans/PR-2-allowlist-wireup.md` (read this in full before writing code).
- **Board entry**: TASKS.md → Backlog → T-002.

## Branch

Branch off `main` (NOT off `fix/version-drift`). Marcus is opening `fix/version-drift` as a separate small PR; PR-2 should not depend on it.

```
git fetch origin
git checkout -b feat/allowlist-wireup origin/main
```

## What to do

Read `docs/plans/PR-2-allowlist-wireup.md` end-to-end, then execute tasks **T1 through T8 in order**. Each task is one failing test → minimal implementation → one Conventional Commit. Do not batch commits. Do not skip the failing-test step.

## Hard constraints

1. **TDD strict.** Write the failing test, see it fail (`cargo test <name>`), then implement, then see it pass. Commit per task as listed in the plan.
2. **Green every commit.** After every commit: `cargo test && cargo clippy --all-targets -- -D warnings`. If either fails, fix in that same commit — never push red.
3. **Out-of-scope items in the plan stay out of scope.** Do not touch CLI subcommands, BlockReason fields, block-page HTML, or env-var configuration. If you think the plan is wrong, stop and reply with a `[PUSH-BACK]` block before writing code.
4. **Decision handler enum.** Use `serde(rename_all = "snake_case")` on a `DecisionAction` enum. Trust serde's form-extract rejection for unknown actions → Axum returns 400 automatically. Verify with the T7 test.
5. **Allowlist path.** Use `dirs::config_dir()` with `.unwrap_or_else(|| PathBuf::from("allowlist.toml"))`. No env-var override.
6. **Load failure is non-fatal.** Corrupt file → `eprintln!` and continue with empty list. Do not exit the service.
7. **Async-safe.** All mutations go through `RwLock::write().await`. The plan's API (`is_allowed`, `allow_once`, `allow_forever`, `forget`) already handles this — just call it.
8. **No new dependencies.** `toml`, `dirs`, `serde`, `axum`, `tokio` are already in `Cargo.toml`.

## When done

Stop after T8. Reply with:
- One-line summary of work done.
- `cargo test` final result count (expect 49 + new tests).
- `cargo clippy` clean confirmation.
- `git log --oneline origin/main..HEAD` showing your 8 commits.
- Do **not** push or open a PR. Marcus will review the local diff, then push.

## If you get stuck

If a task can't be done as written (API mismatch, hidden constraint, etc.), stop, do not invent a workaround. Reply with:

```
[PUSH-BACK]
Task: T-X
Problem: <concrete blocker>
Proposed alternative: <option>
Trade-off: <what changes>
Recommendation: <pick one and justify>
```

Marcus will respond before you continue.
