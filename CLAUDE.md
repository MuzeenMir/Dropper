# SENTINEL — Claude Code project context

Use file for Claude Code sessions in repo. App code under **`sentinel-core/`** (not repo root — flatten to root = Phase 0 task, see below).

## SENTINEL v2 Revamp — Active Phases

**Reference:** `sentinel-core/docs/revamp/` (README, SRS-002, SDD-002, SDP-002, GIT-RESTRUCTURE, CLAUDE-DESIGN-WORKFLOW). Driver: `CODE-REVIEW-main-2026-04-18.md` audit — v1 ~60% real, ~40% scaffolding; chronic CI failures, schema drift, marketing-grade claims.

**Architecture target:** 11 services → 4 + LLM Gateway
- `console` ← api-gateway + auth-service + dashboard
- `controller` ← alert-service + policy-orchestrator (read) + audit
- `analyzer` ← ai-engine + xai-service + Bytewax stream
- `collector` ← data-collector + agent-grpc + sensor skeletons (Falco/Suricata/Wazuh/OpenSCAP)
- `llm-gateway` ← new (Gemma 4, TurboQuant); Phase 1 = shell returning 410

**Phase 0 (4 wks, active):** stabilize — 8 split CI workflows, idempotent migrations, honest README, secrets sweep, SBOM+cosign, OTel pilot, git flatten `sentinel-core/`→root, CODEOWNERS, commitlint, trunk-based. Exit: 7 consecutive green days on `main`.

**Phase 1 (8 wks, blocked on Phase 0):** consolidate behind `USE_V2_*` JWT flags — shared `backend/_lib/` (cim, tenancy, otel, audit, llm_client), Helm scaffold, PG16 + pgvector + RLS, Kafka 3 per-tenant topics, Redis 7, Tempo. Exit: `sentinel-internal` canary runs 14 days on v2, zero P0/P1 regressions.

**Hard constraints:**
- No LLM output reach enforcement adapters — write actions need human approval.
- Audit log append-only at Postgres role level (not app code).
- DRL demoted to research; no Kubernetes role permissions.
- Python 3.12+ backend, FastAPI (Flask sunset by Phase 2); TypeScript 5.x strict, React 18.
- Conventional Commits mandatory; squash-merge only; signed commits on `main`.
- Two-person rule for OPA bundles, model promotions, Helm prod values, RLS policies, audit schemas.

**Do not (Phase 0/1):** decommission v1 services, remove compliance-engine, touch drl-engine beyond archival, land real LLM inference (Phase 2+).

## What this is

SENTINEL = server/endpoint security platform: telemetry collection, AI-assisted detection, policy orchestration, compliance reporting. **Shipping scope (v1):** Flask microservices, React admin console, Kafka/Flink stream processing, ML-based anomaly detection, DRL policy prototype (demoted), Terraform AWS deployment. Marketing claims ("enterprise-grade", production-ready compliance) **not** accurate for v1 — describe v2 target.

## Repository layout

| Area | Path | Notes |
|------|------|--------|
| Backend (Flask microservices) | `sentinel-core/backend/<service>/` | Each service: `app.py`, `requirements.txt`, often Dockerfile |
| Admin UI | `sentinel-core/frontend/admin-console/` | React 18, TypeScript, Vite |
| Stream processing | `sentinel-core/stream-processing/flink-jobs/` | Apache Flink (Python), Kafka-oriented jobs |
| Training | `sentinel-core/training/` | ML / DRL training scripts |
| Infrastructure | `sentinel-core/infrastructure/terraform/` | AWS Terraform |
| Cursor / Bugbot | `.cursor/` | Rules, skills, BUGBOT — parallel to file for Cursor |

## Architecture (data flow)

- Client traffic → **API Gateway** (auth, rate limits, routing) → domain services.
- Collectors + pipelines → **Kafka** → Flink → features; **AI engine** integrates via HTTP and/or Kafka consumption.
- **DRL engine** → policy decisions → **Policy orchestrator** → firewall adapters.
- **Compliance engine** + **XAI** support compliance reporting and explainability.

## Conventions (non-negotiable)

- **No secrets in source**: env vars and `.env.example` placeholders only; never commit keys, passwords, tokens.
- **Safe input handling**: no `eval` / `exec` on untrusted data; parameterized SQL; auth + RBAC on APIs.
- **Errors**: log + handle failures; avoid bare `except:` or silent swallow without logging.
- **Dependencies**: Prefer MIT/Apache/BSD; avoid GPL/AGPL unless approved.
- **Tests**: Backend + stream-processing changes should include or update tests where applicable.

## Documentation

- Specification index: `sentinel-core/docs/SPECIFICATIONS.md`
- Full specs may live in `sentinel-core/docs/specifications/` (often gitignored; distributed separately).
- Quick refs: `sentinel-core/docs/security.md`, `sentinel-core/docs/api-reference.md`, `sentinel-core/docs/ml-models.md`
- Human overview: `sentinel-core/readme.md`
- Cursor-oriented agent summary: `AGENTS.md` (root)

## Common commands

Most work assume `cd sentinel-core` unless noted.

**Stack (Docker)**

```bash
cd sentinel-core
cp .env.example .env   # then edit
docker compose up -d
```

Typical URLs after compose (see `readme.md` for current ports): admin console ~`http://localhost:3000`, API gateway ~`http://localhost:8080`, docs ~`http://localhost:8080/docs`.

**Frontend (`sentinel-core/frontend/admin-console/`)**

```bash
npm install
npm run dev          # Vite dev server
npm run build
npm run test         # Vitest
npm run lint
npm run type-check
```

**Python backend**

- Per-service virtualenv + `pip install -r requirements.txt` under `sentinel-core/backend/<service>/`.
- Run patterns vary by service; check each service's `readme` or `Dockerfile` for entrypoint.

## Git

- **Canonical remote**: `https://github.com/MuzeenMir/sentinel` — use `origin` → that repo for pushes unless user says otherwise.

## gstack — Browser & QA Skills

**Setup (run once):** `bash scripts/install-gstack.sh`

Use `/browse` skill from gstack for all web browsing. Never use `mcp__claude-in-chrome__*` tools.

Available gstack skills: `/office-hours`, `/plan-ceo-review`, `/plan-eng-review`, `/plan-design-review`, `/design-consultation`, `/design-shotgun`, `/design-html`, `/review`, `/ship`, `/land-and-deploy`, `/canary`, `/benchmark`, `/browse`, `/connect-chrome`, `/qa`, `/qa-only`, `/design-review`, `/setup-browser-cookies`, `/setup-deploy`, `/retro`, `/investigate`, `/document-release`, `/codex`, `/cso`, `/autoplan`, `/plan-devex-review`, `/devex-review`, `/careful`, `/freeze`, `/guard`, `/unfreeze`, `/gstack-upgrade`, `/learn`.

## Optional local overrides

For personal Claude Code notes not shared, use **`CLAUDE.local.md`** in project root + add to `.gitignore` if created.

## Skill routing

When user request match available skill, invoke via Skill tool. Skill has multi-step workflows, checklists, quality gates — better than ad-hoc answer. When in doubt, invoke skill. False positive cheaper than false negative.

Key routing rules:
- Product ideas, "is this worth building", brainstorming → invoke /office-hours
- Strategy, scope, "think bigger", "what should we build" → invoke /plan-ceo-review
- Architecture, "does this design make sense" → invoke /plan-eng-review
- Design system, brand, "how should this look" → invoke /design-consultation
- Design review of plan → invoke /plan-design-review
- Developer experience of plan → invoke /plan-devex-review
- "Review everything", full review pipeline → invoke /autoplan
- Bugs, errors, "why is this broken", "wtf", "this doesn't work" → invoke /investigate
- Test site, find bugs, "does this work" → invoke /qa (or /qa-only for report only)
- Code review, check diff, "look at my changes" → invoke /review
- Visual polish, design audit, "this looks off" → invoke /design-review
- Developer experience audit, try onboarding → invoke /devex-review
- Ship, deploy, create PR, "send it" → invoke /ship
- Merge + deploy + verify → invoke /land-and-deploy
- Configure deployment → invoke /setup-deploy
- Post-deploy monitoring → invoke /canary
- Update docs after shipping → invoke /document-release
- Weekly retro, "how'd we do" → invoke /retro
- Second opinion, codex review → invoke /codex
- Safety mode, careful mode, lock down → invoke /careful or /guard
- Restrict edits to directory → invoke /freeze or /unfreeze
- Upgrade gstack → invoke /gstack-upgrade
- Save progress, "save my work" → invoke /context-save
- Resume, restore, "where was I" → invoke /context-restore
- Security audit, OWASP, "is this secure" → invoke /cso
- Make PDF, document, publication → invoke /make-pdf
- Launch real browser for QA → invoke /open-gstack-browser
- Import cookies for authenticated testing → invoke /setup-browser-cookies
- Performance regression, page speed, benchmarks → invoke /benchmark
- Review what gstack learned → invoke /learn
- Tune question sensitivity → invoke /plan-tune
- Code quality dashboard → invoke /health