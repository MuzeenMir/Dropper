# DROPPER

Open-source DNS shield for Windows. Runs as a local resolver on `127.0.0.1`, blocks connections to known-malicious domains using community-curated threat feeds, and serves a calm, evidence-led block-page when something is caught. All processing is local; nothing is sent to a server.

> **Status:** pre-v0.1. Active scaffolding. The Rust skeleton, threat-feed updater, tray icon, installer, and block-page are being built per `TODOS.md` and `DESIGN.md`. No release artifact yet.

> **Naming note (2026-05-07):** this DNS Shield project is now named **Dropper**. The older Sentinel server/endpoint security platform has moved back to [`MuzeenMir/sentinel`](https://github.com/MuzeenMir/sentinel).

## Direction

A single-machine, single-purpose security tool aimed at users underserved by enterprise EDR/XDR products: home labs, indie devs, small studios, anyone who wants malicious-domain blocking without the SaaS console, the data exfil, or the per-seat bill.

- **DNS-layer blocking** against URLhaus + Tranco-anchored allowlist.
- **Quad9 / Cloudflare upstream** failover with status surfaced in the tray.
- **Block-page** on `127.0.0.1` explaining *what*, *why*, and *what to do* — with `Allow once` / `Allow forever` controls.
- **Tray icon** as the constant trust surface (green / amber / red).
- **VPN-aware** — auto-pause when another adapter takes over DNS (v0.2).
- **Open source, MIT, no telemetry, signed releases via cosign.**

Reference design: `DESIGN.md` (visual identity + component patterns). Roadmap: `TODOS.md` (T1–T3 + DX expansion).

## Repository layout

```
.
├── DESIGN.md         # visual identity, components, copy library
├── TODOS.md          # operational backlog (T1 archive, T2 Tranco refresh, T3 DX)
├── CLAUDE.md         # Claude Code session context
├── README.md         # this file
└── .github/workflows # CI (lint commitlint, security gitleaks + trivy)
```

The Rust crate skeleton (`Cargo.toml`, `src/`, `crates/`) lands in PR-3 of the T1 slice.

## Contributing

- Conventional Commits required (`commitlint.config.js`).
- Squash-merge only; signed commits required on `main`.
- `CODEOWNERS` gates review (solo-dev phase: everything routes to `@MuzeenMir`).

## License

MIT — see `LICENSE`.
