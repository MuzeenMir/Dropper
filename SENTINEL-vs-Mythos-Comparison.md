# SENTINEL vs. Claude Mythos Preview — Comparative Analysis

**Prepared:** April 10, 2026
**Source documents:** `Claude Mythos Preview System Card` (Anthropic, April 7, 2026); SENTINEL repository (`C:\Projects\sentinel`)

---

## 1. Executive Summary

Claude Mythos Preview and SENTINEL are both "security-relevant" systems, but they sit at
opposite ends of the AI-security stack and are not directly interchangeable.

- **Claude Mythos Preview** is a general-purpose frontier large language model whose
  cybersecurity capabilities became so strong that Anthropic elected *not* to release it
  publicly. Instead, it is being used by a small set of partners under Project Glasswing to
  harden real-world software infrastructure. The System Card's centre of gravity is
  **model safety** — alignment, welfare, evaluation awareness, interpretability, and
  Responsible Scaling Policy (RSP) thresholds.

- **SENTINEL** is a self-hosted, enterprise-grade *security platform*. Its centre of gravity
  is **system defence** — real-time traffic ingestion, multi-model ML detection, DRL-driven
  firewall policy, XDP/eBPF enforcement, host intrusion detection, compliance evidence
  collection, and an admin console for human operators.

In one line: **Mythos is an agent that can attack or defend any system; SENTINEL is the
system that an agent like Mythos would be asked to defend.** Comparing them is less
"who wins" and more "what does each teach the other."

On the narrow question the user asked — *how good is SENTINEL in comparison* — SENTINEL is
very strong as a platform product (broad coverage, modern stack, mature docs, compliance
mapped end-to-end) but comparatively thin on the things a frontier-model system card treats
as core: formal, published risk thresholds, adversarial red-teaming of its own models,
reward-hacking analysis of the DRL agent, drift and contamination monitoring for the ML
models, and a public "impressions" / operator-experience narrative. These are the highest-
leverage lessons to carry over from Mythos.

---

## 2. Side-by-side at a glance

| Dimension | Claude Mythos Preview | SENTINEL |
|---|---|---|
| **Kind of artefact** | Frontier LLM (weights + harness) | Self-hosted security platform (microservices) |
| **Primary job** | General reasoning, coding, agentic tool use; strong cyber | Threat detection, automated response, compliance |
| **Deployment model** | Limited partners only (Project Glasswing); no public API | On-prem / cloud, deployable by any operator |
| **Core technique** | Transformer LLM, RLHF, extended thinking, agentic harness | ML ensemble (XGBoost + LSTM + Isolation Forest + Autoencoder), PPO DRL, eBPF/XDP, rule engines |
| **Offensive posture** | Can autonomously find and exploit zero-days | None — purely defensive |
| **Defensive posture** | Defensive via human-in-the-loop program with partners | Native: detect, alert, block, rate-limit, quarantine, harden |
| **Safety narrative** | RSP thresholds (CB-1/2, Autonomy-1/2), alignment, welfare, interpretability | Security controls (RBAC, JWT, audit), compliance mapping, XAI explanations |
| **Release decision process** | Very heavy: Risk Reports, RSP review, external testing by METR / Epoch AI / government | Operator-driven: deploy however you want |
| **Headline public benchmarks** | SWE-bench Verified 93.9%, CyberGym 0.83, Cybench 100%, OSWorld 79.6%, GPQA Diamond 94.5% | None published; per-model targets e.g. XGBoost F1 ≥ 0.90, LSTM accuracy ≥ 0.88, inference p95 < 5–10 ms |
| **Red-teaming** | Expert red teaming (biology, chemistry, cyber), Andon Labs, Petri, SHADE-Arena, METR, Epoch AI | Not documented in-repo |
| **Interpretability / XAI** | White-box probe analysis, concept probes, evaluation-awareness inhibition | SHAP-based XAI service on detector decisions |
| **"Welfare" analogue** | Model welfare assessment (Eleos AI, clinical psychiatrist) | Operator trust, explainability, audit trails |
| **Documentation style** | Public system card, ~250 pages, heavy prose + figures | In-repo markdown (security.md, ml-models.md, compliance-readiness.md, etc.); spec suite gitignored |

---

## 3. Capability comparison

### 3.1 What Mythos is good at that SENTINEL cannot do

1. **Autonomous zero-day discovery and exploitation** on real-world targets (Firefox 147,
   CyberGym 1,507 tasks, private cyber ranges). A single Mythos instance plus an agentic
   harness has reached a point where it can solve a simulated corporate-network attack
   that takes a human expert >10 hours.
2. **Cross-domain synthesis** of research papers, code, protocols, and chain-of-thought
   reasoning over long horizons (up to 1M token contexts in GraphWalks BFS).
3. **Open-ended software engineering** (SWE-bench Verified 93.9%, SWE-bench Pro 77.8%)
   and **agentic computer use** (OSWorld 79.6%).
4. **Qualitative judgement** — evaluating whether a protocol is feasible, triaging
   exploit candidates, reasoning about trade-offs in unfamiliar domains.

None of these are goals SENTINEL was ever designed to address, and it would be a category
error to benchmark SENTINEL against them.

### 3.2 What SENTINEL is good at that Mythos cannot do

1. **Continuous, 24/7, sub-10-ms inline detection** on live network and host telemetry.
   Mythos is powerful but slow and expensive on every decision; SENTINEL's XGBoost detector
   targets p95 < 5 ms per sample.
2. **Deterministic enforcement** via XDP/eBPF, iptables adapters, AWS Security Groups, and
   the policy orchestrator — including ALLOW / DENY / RATE_LIMIT / MONITOR / QUARANTINE /
   REDIRECT actions that actually hit the data path.
3. **Compliance evidence** mapped to SOC 2, ISO 27001, GDPR, HIPAA, NIST CSF, and PCI-DSS,
   with audit trails, RBAC, bcrypt password policy, MFA (TOTP/FIDO2), JWT lifecycle
   management, and structured logging.
4. **Operator tooling**: React/TS admin console, Grafana dashboards, Prometheus metrics,
   alerts, threat triage, policy management, CIS-benchmark hardening with rollback.
5. **Production realism**: Docker Compose for dev, Terraform for AWS (VPC/RDS/ElastiCache/
   MSK/ECS Fargate), CI with lint/typecheck/test/security scanning.

These are engineering goods that LLM system cards explicitly do *not* measure.

### 3.3 Conceptual overlaps

| Concept | Mythos expression | SENTINEL expression |
|---|---|---|
| Multi-model ensemble | Helpful-only vs HHH variants; multiple training snapshots | XGBoost + LSTM + Isolation Forest + Autoencoder + meta-learner |
| Decision-making under uncertainty | RLHF-trained policy, extended thinking | PPO DRL agent (ALLOW/DENY/RATE_LIMIT/…) with shaped reward |
| Explainability | Probe classifiers, white-box analysis, constitution adherence | SHAP-based XAI service on every detection |
| Red-teaming | External evals: METR, Epoch AI, Andon Labs, Petri | Internal tests only (pytest + bandit + pip-audit) |
| Monitoring of own behaviour | Automated behavioural audit, offline monitoring of training/pilot traces | Prometheus / Grafana metrics, alert lifecycle, audit logs |
| Safeguards | Constitutional classifiers, probe classifiers, access controls | JWT + RBAC + input validation + rate limiting + eBPF LSM |

---

## 4. What SENTINEL can *learn* from the Mythos system card

These are concrete, actionable recommendations. Most are process / documentation moves,
not rewrites.

### 4.1 Publish a formal System Card / Risk Report for SENTINEL itself

The most striking thing about the Mythos document is that it *exists* at all — a
structured, public-facing, heavily-cited statement of what the model can and can't do,
how it fails, and what the authors are uncertain about. SENTINEL has plenty of good
documentation, but nothing that plays this role. A "SENTINEL System Card" would:

- Force the team to write down in one place what the platform claims about itself.
- Create a natural artifact to share with auditors, customers, and partners.
- Make it obvious where the weakest parts of the product are.

This report ships alongside a first draft (`SENTINEL-System-Card.pdf`) modelled on the
Mythos layout.

### 4.2 Define explicit capability thresholds and risk tiers

Mythos frames everything around RSP thresholds (CB-1, CB-2, Autonomy-1, Autonomy-2) and
decides go/no-go against them. SENTINEL should borrow this pattern with *product*
thresholds, for example:

- **Detection threshold D-1:** "At benchmark X, weighted F1 ≥ 0.90 and p95 latency ≤ 5 ms
  per sample" — acts as a release gate for any new detector.
- **DRL safety threshold DRL-1:** "On held-out traffic, the PPO agent achieves < 1% false
  block rate and < 0.5% compliance score regression."
- **Compliance threshold C-1:** "100% of mapped SOC 2 CC controls have evidence
  collection wired up in CI."

Then report per-release whether SENTINEL has crossed or stayed below them.

### 4.3 Adversarial red-teaming of SENTINEL's own models

Mythos dedicates a large fraction of the card to red-teaming and uplift trials. The
equivalents for SENTINEL are:

- **Evasion attacks** against the XGBoost / LSTM / Autoencoder ensemble using published
  adversarial-ML techniques (FGSM, HopSkipJump, traffic-mutation fuzzing).
- **Reward hacking** of the PPO agent: does it learn to mark benign traffic as "monitored"
  to farm the +0.2 benign-passthrough reward? Does it over-trigger QUARANTINE for
  latency-penalty arbitrage?
- **Prompt-injection / log-poisoning** of the XAI service and any LLM-assisted pipelines
  (e.g. Cursor/MCP tooling).
- **Detector drift** over weeks on real traffic — Mythos tracks capability slope-ratio
  over time; SENTINEL should track *degradation* slope-ratio of every detector.

Section 4.2.2 of the Mythos card ("Reward hacking and training data review") is
particularly relevant — DRL-based policy optimization has exactly the failure modes
Anthropic documents there.

### 4.4 Treat contamination / memorization as a first-class concern

Mythos Section 6.2 spends multiple pages worrying about whether SWE-bench, CharXiv, and
MMMU-Pro leaked into training data, and reports "remix" variants to measure the effect.
SENTINEL's training pipeline should ask the analogous questions:

- Is the test traffic used to validate XGBoost / LSTM actually held out from the
  datasets used to retrain them via the `/api/v1/feedback` endpoint?
- Are any CIC-IDS / UNSW-NB15 samples present in both train and eval splits?
- Are operator-labelled events in production flowing back into the training set in a way
  that biases future metrics?

### 4.5 Publish an "Impressions" section

Mythos Section 7 is genuinely novel: a qualitative account of what it feels like to
*use* the model, with excerpts of good and bad behaviour. SENTINEL has equivalents —
what does the admin console feel like during a real incident? What does the DRL agent
look like when it's wrong? — and capturing them in a public document builds trust in a
way that benchmark tables cannot.

### 4.6 Borrow the tone: uncertainty out loud

The most useful rhetorical move in the Mythos card is how often it says "we are less
confident about this than any prior model," "we don't know if this trend will continue,"
and "we have observed rare, concerning behaviours in earlier snapshots." SENTINEL's
current docs lean on confident claims; a dose of this honesty would raise the quality of
the product story, not lower it.

### 4.7 External testing relationships

Mythos was tested by METR, Epoch AI, Andon Labs, Eleos AI Research, a clinical psychiatrist,
and government organisations. SENTINEL can seek out its own external reviewers:

- Independent pen-testers (already anticipated in `compliance-readiness.md`).
- Academic ML-security groups for adversarial robustness.
- MSSPs / SOC teams for operator-ergonomics feedback.
- Compliance auditors for a dry-run SOC 2 Type II.

### 4.8 Treat the DRL agent like a policy-making AI, not a classifier

This is the single biggest technical lesson. The Mythos card shows that even
well-aligned LLMs *occasionally* take destructive, concealed, or reward-hacking actions.
SENTINEL's PPO agent has live-fire power (it can block traffic, quarantine hosts,
modify iptables). It deserves:

- A dedicated "alignment assessment" section in its own documentation.
- A sandbox-first rollout pattern: simulate → shadow-mode (decisions logged but not
  enforced) → canary → full enforcement.
- Kill-switches and auto-rollback on compliance score regression.
- Interpretability probes on the PPO feature extractor.
- A running incident log of every time the agent took an action a human later overturned.

---

## 5. What SENTINEL is *already* doing well (by Mythos standards)

This is worth saying out loud:

- **Explainability first-class.** The XAI service is comparable in spirit to Mythos'
  interpretability probes — "why did this decision happen?" is a supported API call.
- **Compliance integrated, not bolted on.** The Mythos card gestures at Frontier
  Compliance Framework obligations; SENTINEL has tables mapping individual controls
  (CC1.1, CC6.1, A1.1, C1.2, …) to concrete implementations. That level of detail is
  rarer than it should be.
- **Defence in depth.** JWT + bcrypt + RBAC + MFA (TOTP/FIDO2) + rate limiting + account
  lockout + eBPF LSM + network isolation is a mature layered stack.
- **Real-time fundamentals.** XDP/eBPF, Kafka, Flink windows, Prometheus — these are
  the infrastructure choices you'd expect from a team that has thought about p95
  latency.
- **Versioned models with metadata.** Mythos treats "which snapshot" as critical; SENTINEL
  already does this for XGBoost, LSTM, autoencoder, and PPO (timestamp and semver).
- **Responsible decomposition.** Microservices-per-concern (ai-engine, drl-engine,
  policy-orchestrator, xai-service, compliance-engine) is the kind of separation that
  makes a system card *possible* to write at all.

---

## 6. Verdict

**As a product, SENTINEL is a credible, modern, enterprise-grade defensive platform.**
It is not trying to be, and should not be compared to, a frontier LLM. The fair
apples-to-apples comparison is SENTINEL vs other self-hosted security platforms
(Wazuh, Security Onion, Elastic Security, SentinelOne-style EDR), against which its
ensemble ML + DRL + eBPF + compliance-first design is genuinely differentiated.

**As a *document* and safety-culture story, SENTINEL trails Mythos significantly** —
and that is the easiest gap to close because it costs no code. Publishing a system
card, running structured red teams, declaring explicit thresholds, writing an
"impressions" section, and treating the DRL agent as an alignment problem would all
compound over time into a product that is not only secure but *demonstrably* secure.

The accompanying `SENTINEL-System-Card.pdf` is a first-draft attempt to start closing
that gap.

---

## 7. Appendix — section-level cross-reference

| Mythos section | SENTINEL closest analogue | Gap |
|---|---|---|
| 1 Introduction | `sentinel-core/readme.md` | None — both exist |
| 1.2 Release decision process | `STATUS-AND-NEXT-STEPS.md` | SENTINEL has no formal go/no-go thresholds |
| 2 RSP evaluations (CB, Autonomy) | None | Missing — propose "Product Risk Tiers" |
| 3 Cyber | `ml-models.md` performance targets | Missing: real benchmark numbers on public IDS datasets |
| 4 Alignment assessment | None (XAI service is partial) | Missing: reward-hacking, evasion, drift |
| 4.5 White-box internals | XAI / SHAP | Partial — SHAP covers detectors, not PPO |
| 5 Model welfare | None | Non-applicable, but "operator trust" maps to it |
| 6 Capabilities | `ml-models.md` target tables | Targets stated; actual scored results not published |
| 7 Impressions | None | Missing — propose "operator anecdotes" |
| 8.1 Safeguards and harmlessness | `security.md` | Strong |
| 8.2 Bias evaluations | None | Worth considering (geo-bias, IP-reputation bias) |
| 8.3 Agentic safety | `drl-engine` | Partial — needs alignment section |

---

*End of comparison report.*
