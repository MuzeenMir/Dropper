# dropper — Task Board

Owner: Marcus (CTO) / Kai (Lead Dev)
Bucket: `/mnt/c/Projects/sentinel` (filesystem name; product renamed to Dropper 2026-05-07).
Stack: see `CLAUDE.md` for current state.

Per-repo conventions live in `CLAUDE.md`. Read it before opening a ticket here.

---

## WIP

- [T-001] Sync Cargo.toml version + repository URL to v0.1.3 (Marcus) — 2026-05-11 → Review [P0]
  Context: Cargo.toml drifted to `0.1.0-dev` across four releases (v0.1.0–v0.1.3) because release-please `release-type: simple` only tracks `version.txt`. `dropper --version` reported `0.1.0-dev` on every shipped binary.
  Acceptance: Cargo.toml = 0.1.3; `extra-files` added to release-please-config so future bumps stay in sync; `dropper --version` prints `dropper 0.1.3`; tests + clippy clean.
  Branch: fix/version-drift

## Review

(empty)

## Backlog

- [T-002] Wire AllowList into Resolver + decision handler (Kai, owner: Marcus reviews) — 2026-05-11 [P0]
  Context: PR-1 (#26) shipped the allowlist store; nothing on `main` reads or writes it. Resolver ignores `is_allowed`, block-page POST `/decision` is a no-op stub. Block-page action buttons functionally inert.
  Acceptance: Resolver bypass when `is_allowed`; POST `/decision` dispatches keep_blocked / allow_once / allow_forever / forget; unknown action → 400; main.rs constructs + loads `AllowList` from `dirs::config_dir()/dropper/allowlist.toml`; corrupt file logs + continues empty. Full TDD task breakdown in `docs/plans/PR-2-allowlist-wireup.md`.
  Branch: feat/allowlist-wireup (off main, after T-001 lands)

- [T-003] CLI subcommand stubs (Kai) — 2026-05-11 [P1]
  Context: `install`, `doctor`, `tail`, `update` print eprintln + exit 2. T3 NSIS installer needs `install` + `doctor` real. Pre-requisite for v0.1 ship.
  Acceptance: each subcommand returns sensible behavior or a documented "coming soon" with --help integration; no dead `eprintln` stubs in `main.rs`.
  Branch: TBD

## Done

(empty)

## Blocked

(empty)

---

## Ticket entry format

```
- [T-NNN] <one-line summary> (<owner>) — <date|state-change> [P0|P1|P2|P3]
  Context: <2-3 lines on why>
  Acceptance: <bullet list or one-liner>
  Branch: feat/<slug>
```
