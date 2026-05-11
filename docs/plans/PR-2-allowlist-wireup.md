# PR-2 — Wire AllowList into Resolver + Block-Page Decision Handler

**Cluster:** resolver-UX correctness (PR-1 store → PR-2 wire-up → PR-3 forget UI).
**Status:** READY FOR IMPLEMENTATION
**Owner:** Kai (Codex), reviewed by Marcus.
**Ticket:** T-002 in `TASKS.md`.

## Why

PR-1 (`#26`) shipped the `AllowList` store with full unit-test coverage. Nothing on `main` reads or writes to it:

- `Resolver::handle_request` (`src/resolver/mod.rs:82-107`) consults the blocklist but never calls `is_allowed`. Allow-forever entries are dead data.
- `blockpage::handle_decision` (`src/blockpage/mod.rs:151-157`) ignores the form payload and returns `204`. The three action buttons on the block-page are visually wired but functionally inert.
- `main.rs` never constructs an `AllowList`, never loads one from disk.

This is the entire reason PR-1 was scaffolded. PR-2 connects it.

## Goal

After PR-2, the block-page action buttons are real:
- **Keep blocked** → no state change, 204.
- **Allow once** → domain bypasses the blocklist for 30 minutes; next query for that domain is forwarded upstream.
- **Allow forever** → domain bypasses permanently; persisted atomically to `%APPDATA%\dropper\allowlist.toml` (Linux/macOS fallback via `dirs::config_dir()`); survives restart.
- Unknown action → 400.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  main.rs                                                │
│   ├─ persist_path = dirs::config_dir() / "dropper" /    │
│   │                 "allowlist.toml"                    │
│   ├─ allowlist = new_allowlist(persist_path)            │
│   ├─ load(&allowlist) → log count or warn               │
│   ├─ AppState::with_allowlist(allowlist.clone())        │
│   └─ Resolver::new(blocklist, blockpage, allowlist)     │
└─────────────────────────────────────────────────────────┘
              │                                  │
              ▼                                  ▼
  ┌──────────────────────┐         ┌────────────────────────────┐
  │  Resolver            │         │  AppState                  │
  │  - blocklist         │         │  - current: Option<...>    │
  │  - blockpage         │         │  - allowlist: AllowList    │
  │  - upstream          │         └────────────────────────────┘
  │  - allowlist  (NEW)  │                       │
  └──────────────────────┘                       ▼
              │                       ┌─────────────────────────┐
              ▼                       │  POST /decision         │
  handle_request:                     │   match action {        │
    1. is_allowed?    → forward       │     keep_blocked → 204  │
    2. else lookup    → sinkhole      │     allow_once → mutate │
                                      │     allow_forever → ... │
                                      │     forget → ...        │
                                      │     _ → 400             │
                                      │   }                     │
                                      └─────────────────────────┘
```

### Type-level changes

- `AppState`: add `pub allowlist: AllowList` field. Add `AppState::with_allowlist(AllowList) -> Self`. Keep `AppState::new()` for tests (constructs a tmp-path allowlist).
- `Resolver::new` signature gains a third arg: `allowlist: AllowList`. Add `allowlist` field.
- `handle_decision` gains `State<AppState>` extractor.
- Introduce `DecisionAction` enum with `serde(rename_all = "snake_case")` derived deserialization on `DecisionForm.action` (string → enum at the form-extract boundary).

### Allowlist path resolution (main.rs only)

```rust
let persist_path = dirs::config_dir()
    .map(|d| d.join("dropper").join("allowlist.toml"))
    .unwrap_or_else(|| PathBuf::from("allowlist.toml"));
```

No env-var override yet — that's a v0.2 ergonomics item, not a v0.1 blocker.

### Load behavior

- Missing file: log `info!("allowlist: starting empty (no file at {path})")` — already handled by `load`.
- Corrupt file: log error and **continue with empty list**. Don't hard-fail the service. The user can `dropper doctor` (T3) or hand-edit the file to recover.

## Tasks (TDD order)

Every task is one failing test, then one minimal implementation change, then commit. Conventional Commit format. Each commit must keep `cargo test` and `cargo clippy --all-targets -- -D warnings` green.

### T1 — AppState carries AllowList

- **Test (blockpage)**: `appstate_default_constructs_empty_allowlist` — `AppState::new()` works; `is_allowed(&state.allowlist, "anything").await` is `false`.
- **Impl**: add `pub allowlist: AllowList` to `AppState`. Add `AppState::with_allowlist(allowlist: AllowList) -> Self`. `AppState::new()` constructs a tmp-path allowlist (test-only ergonomics).
- **Commit**: `feat(blockpage): AppState carries AllowList handle (PR-2)`

### T2 — Resolver carries AllowList

- **Test (resolver)**: `resolver_new_accepts_allowlist` — `Resolver::new(blocklist, blockpage, allowlist)` compiles and the field is reachable from `handle_request`.
- **Impl**: add `allowlist: AllowList` field; update `Resolver::new` signature; update existing call sites in tests and `main.rs`.
- **Commit**: `feat(resolver): Resolver carries AllowList handle (PR-2)`

### T3 — Allowlisted domain bypasses blocklist

- **Test (resolver)**: build a `Resolver` with a blocklist containing `phish.example` AND an allowlist containing `phish.example`. Drive `handle_request` with a query for `phish.example`. Assert the response was forwarded (or — easier — assert `blockpage.current` was NOT mutated). Use an in-process `ResponseHandler` mock that captures the response.
- **Impl**: in `handle_request`, before `lookup(&self.blocklist, &domain)`, call `if is_allowed(&self.allowlist, &domain).await { return forward(...); }`.
- **Commit**: `feat(resolver): allowlist bypass before blocklist lookup (PR-2)`

### T4 — POST /decision with allow_once mutates allowlist

- **Test (blockpage)**: build router, POST `/decision` with `domain=phish.example&block_id=…&action=allow_once`. Assert response is 204. Assert `is_allowed(&state.allowlist, "phish.example").await` is true.
- **Impl**: introduce `#[derive(Deserialize)] enum DecisionAction { KeepBlocked, AllowOnce, AllowForever, Forget }` with `#[serde(rename_all = "snake_case")]`. Add `State<AppState>` extractor to `handle_decision`. Match on action, call `allow_once`.
- **Commit**: `feat(blockpage): decision handler dispatches allow_once (PR-2)`

### T5 — POST /decision with allow_forever persists to disk

- **Test (blockpage)**: build `AppState::with_allowlist` against a tmp path. POST `/decision` with `action=allow_forever`. Assert 204, assert disk file exists, assert reloading via `load` returns 1 entry.
- **Impl**: handler arm for `AllowForever` calls `allow_forever`. Convert `Result` into 204 on Ok, 500 on Err (anyhow message into log + opaque `StatusCode::INTERNAL_SERVER_ERROR`).
- **Commit**: `feat(blockpage): decision handler dispatches allow_forever (PR-2)`

### T6 — POST /decision with forget arm

- **Test (blockpage)**: pre-populate `allow_forever("phish.example")`. POST `/decision` with `action=forget`. Assert 204, assert `is_allowed` is now false.
- **Impl**: handler arm calls `forget`.
- **Commit**: `feat(blockpage): decision handler dispatches forget (PR-2)`

### T7 — Unknown action returns 400

- **Test (blockpage)**: POST `/decision` with `action=nuke_from_orbit`. Assert response is 400.
- **Impl**: serde rejection on unknown variant → 400 via `axum::extract::rejection::FormRejection` (cleanest: keep the enum and rely on form-extract error → Axum returns 400 automatically). Verify with the test.
- **Commit**: `feat(blockpage): decision handler rejects unknown actions (PR-2)`

### T8 — main.rs constructs + loads + threads AllowList

- **Test**: none directly — covered by `cargo build --release` succeeding and the existing service smoke test if present. Add a doc-test or `#[test]` in `main.rs` only if a clean boundary exists; otherwise skip.
- **Impl**: in `run_service`:
  1. Compute `persist_path` via `dirs::config_dir()` (fallback `PathBuf::from("allowlist.toml")`).
  2. `let allowlist = new_allowlist(persist_path);`
  3. `if let Err(e) = load(&allowlist).await { eprintln!("allowlist: load failed, starting empty: {e:#}"); }`
  4. `let blockpage = AppState::with_allowlist(allowlist.clone());`
  5. `let resolver = Resolver::new(blocklist.clone(), blockpage.clone(), allowlist);`
- **Commit**: `feat(main): construct + load AllowList on service boot (PR-2)`

## Out of scope (do NOT do in this PR)

- CLI subcommands (`install`, `doctor`, `tail`, `update`) — still stubs; PR-3+.
- Toast / tray notification on allowlist mutation — UI work, future.
- Env-var override for allowlist path — v0.2.
- Block-page UI changes (button copy, layout) — separate design pass.
- Listing-date enrichment in `BlockReason` — separate ticket.

## Verification gates

Before opening the PR:
1. `cargo build --release` succeeds.
2. `cargo test` — all 49 existing tests pass, plus 5–6 new ones (T1, T3, T4, T5, T6, T7).
3. `cargo clippy --all-targets -- -D warnings` clean.
4. `./target/release/dropper --version` prints `dropper 0.1.3` (depends on `fix/version-drift` landing first or branching from it).
5. Manual smoke (Marcus, Windows VM): start service, hit block-page, click each action, confirm next query for the same domain behaves correctly (sinkholed / forwarded / re-blocked).

## Branch + commit hygiene

- Branch: `feat/allowlist-wireup` off `main` (after `fix/version-drift` merges, OR explicitly rebase later).
- One commit per task, atomic, conventional. No squash-fix-fix-fix.
- Final commit before review: `docs(plans): mark PR-2 complete in docs/plans/PR-2-allowlist-wireup.md`.

## Risk + rollback

Low risk. The resolver change is one early-return guarded by an `Arc<RwLock>` read. The decision handler change is additive — adds dispatch where there was a no-op. Rollback is `git revert` of the merge commit; on-disk allowlist file is harmless if abandoned.
