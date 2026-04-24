# SENTINEL Revamp — Backlog

Items out of scope for the current phase but tracked so they don't get lost. When a phase opens that covers an item, move it into that phase's SDP section and delete it here. New items land at the bottom with a date and a one-line rationale.

Format per item: `- [P<phase>] <area> — <one line>. <Why deferred.> (added YYYY-MM-DD)`

Phase tags:
- `P0` — stabilize (weeks 1–4)
- `P1` — consolidate (weeks 5–12)
- `P2` — sensor migration, real LLM inference, compliance decommission starts
- `P3+` — enterprise GA, SSO/SCIM, billing, SOC2, marketplace
- `P?` — unassigned; needs triage

## Phase 0 (stabilize) — deferred items

- [P0] docs — auto-generated ADR index via `scripts/gen_adr_index.sh`. Manual table in `docs/adr/README.md` until then. (added 2026-04-19)
- [P0] ci — reusable workflow composite action extraction after 8 split workflows settle. Premature before the split lands. (added 2026-04-19)
- [P0] governance — migrate CODEOWNERS entries from `@MuzeenMir` placeholder to org-scoped teams (`@MuzeenMir/sentinel-backend`, `…-frontend`, `…-security`, `…-infra`). Blocked on org team creation. (added 2026-04-19)
- [P0] migrations — split `20260313_001_enterprise_schema.py` into per-domain revisions if the audit shows it is doing too much. Decide during Slice 3. (added 2026-04-19)
- [P0] migrations — schema consolidation per `docs/DB-MIGRATION-DRIFT-AUDIT.md` D-1 through D-7: rename `audit_log`→`audit_logs`, add missing core tables (alerts/threats/firewall_policies/network_logs/training_data/rl_agent_states/system_config) via a new `20260417_001_consolidate_schema.py`; SSO/SCIM/MFA tables via `20260417_002_sso_scim.py`; enable Postgres RLS via `20260417_003_enable_rls.py`; then strip `init.sql` to extensions + default tenant only and drop `db.create_all()` at service boot. Slice 3 handled *idempotency* only — this is the follow-up. (added 2026-04-19)
- [P0] repo-layout — git flatten `sentinel-core/` → repo root (Slice 6). Deferred past Phase 0 exit: high blast-radius (every Dockerfile/compose/CI path + every open branch breaks), cosmetic payoff (solo contributor right now), and risks burning the "7 consecutive green days on `main`" exit gate. Execute between Phase 0 exit and Phase 1 service-consolidation kickoff — that's the natural seam. Tooling plan: single atomic commit via `git mv`, path-rewrite sed sweep across `.github/workflows/*.yml`, `docker-compose.yml`, all `Dockerfile`s, `alembic.ini`, and `CLAUDE.md`/`AGENTS.md`/`CODEOWNERS`. (added 2026-04-19)

## Phase 1 (consolidate) — deferred items

- [P1] console — SCIM v2 provisioning endpoints. Scaffolded in UI mocks; backend lands P1 week 5+. (added 2026-04-19)
- [P1] llm-gateway — vLLM backend wiring (currently a placeholder returning 410). Real inference is Phase 2. (added 2026-04-19)
- [P1] analyzer — Bytewax pipeline only covers OS audit + Falco in P1 scope. Suricata and Wazuh adapters land alongside sensor migration in P2. (added 2026-04-19)
- [P1] collector — Falco / Suricata / Wazuh / OpenSCAP adapter *skeletons* only. Full adapters = P2. (added 2026-04-19)
- [P1] contract-tests — fuzzing against pinned v1 OpenAPI is nice-to-have; baseline deny-list is sufficient for the canary. (added 2026-04-19)

## Phase 2+ — deferred items

- [P2] sensors — decommission `xdp-collector` and `ebpf-lib` after Falco/Suricata adapters cover parity. Must not break existing telemetry customers. (added 2026-04-19)
- [P2] llm-gateway — TurboQuant loader for Gemma 4. Edge llama.cpp path per ADR-010. (added 2026-04-19)
- [P2] analyzer — real-time inference on Bytewax with tenant-scoped feature store. (added 2026-04-19)
- [P3] enterprise — SSO (SAML + OIDC), SCIM v2, billing, usage metering, marketplace listing. (added 2026-04-19)
- [P3] compliance — SOC2 Type II audit. Prerequisite: audit hash-chain and RLS evidence. (added 2026-04-19)
- [P6] compliance — decommission v1 `compliance-engine` after v2 replacement has certification. (added 2026-04-19)

## Pivot (post-MVP) — deferred items

Source: `~/.gstack/projects/MuzeenMir-sentinel/ceo-plans/2026-04-24-pivot-transition.md`. When v2 revamp/ archives in week 2, move these to `docs/PIVOT-BACKLOG.md` at repo root.

- [Pivot-W5+] sdk — Node SDK (OpenAI + Anthropic wrap). Python-only proves MVP; Node when first JS-shop design partner asks. (added 2026-04-24)
- [Pivot-W5+] benchmark — Datadog + Wiz competitor runners in sentinel-agent-bench repo. Splunk-only proves the point for Show HN; broader coverage later. (added 2026-04-24)
- [Pivot-W5+] infra — one-command Fly.io self-host CLI (Postgres provisioning + secret gen + API-key bootstrap UX). Week 3-4 ships 5-command documented path; true CLI post-MVP. (added 2026-04-24)
- [Pivot-M2] sdk — LangChain wrap. Order 3 in design-doc framework support; lands once Python + Node core proven. (added 2026-04-24)
- [Pivot-M2] detection — prompt-injection chain rule. Requires prompt + tool call correlation; design-doc differentiation rule #3. (added 2026-04-24)
- [Pivot-M2] detection — recursive tool-loop cost rule. Requires call-sequence + cost-per-call linkage; differentiation rule #2. (added 2026-04-24)
- [Pivot-M3] ml — re-apply ai-engine anomaly code on real agent traces once 3 design partners produce sufficient volume. ML does not transfer from endpoint to agent data per design doc. (added 2026-04-24)
- [Pivot-M6] compliance — SOC2 Type 1 audit. Prerequisite: RLS + audit hash-chain. (added 2026-04-24)
- [Pivot-M12] compliance — SOC2 Type 2 audit. Prerequisite: Type 1 + 6-month operating window. (added 2026-04-24)
- [Pivot-P3] enterprise — multi-tenant RLS + SSO (SAML + OIDC) + billing. (added 2026-04-24)
- [Pivot-P3] ux — block-rule UI in trace viewer. MVP is read-only. (added 2026-04-24)
- [Pivot-P3] billing — per-call micro-billing. MVP is flat SaaS tiers. (added 2026-04-24)
- [Pivot-P3] infra — EU region deploy. Defer until first EU customer. (added 2026-04-24)
- [Pivot-P6] compliance — successor to v1 compliance-engine (which is being killed in week 2). Revisit after SOC2 Type 2. (added 2026-04-24)

## Archived (v2 revamp, deferred indefinitely as of 2026-04-24 pivot)

Per `ceo-plans/2026-04-24-pivot-transition.md` — entire v2 11-service consolidation plan (SRS-002 / SDD-002 / SDP-002) is deferred indefinitely. When week 2 demolition lands, the revamp/ directory moves to docs/archive/v2-revamp/ and these docs become historical record only. Items below remain referenced but not scheduled:

- All Phase 1 consolidate items (console/controller/analyzer/collector/llm-gateway shell) above.
- All Phase 2+ sensor migration, compliance-engine decommission, enterprise GA items above.

## Unassigned

- [P?] drl-engine — long-term fate post-archival. Research track or full removal? Decide after Phase 2. (added 2026-04-19)
- [P?] frontend — decide on shadcn/ui vs. custom component library for `frontend/console/`. Currently: custom. (added 2026-04-19)
- [P?] infra — Terraform → Pulumi migration proposed elsewhere; no ADR yet. (added 2026-04-19)
- [P?] observability — Grafana dashboards-as-code (grafonnet or Terraform provider). (added 2026-04-19)
- [P?] security — runtime WAF placement (Envoy filter vs. sidecar). (added 2026-04-19)

## Rules

- Do not use this file as a to-do list for the *current* phase. That's what SDP-002 and GitHub issues are for.
- Every item has a phase tag and a rationale. "We should do X eventually" without a reason gets rejected.
- When an item is pulled into an active phase, delete it from here — don't cross it out. Git history is the record.
