# DROPPER

Open-source DNS shield for Windows. Runs as a local resolver on `127.0.0.1`, blocks connections to known-malicious domains using community-curated threat feeds, and serves a calm, evidence-led block-page when something is caught. All processing is local; nothing is sent to a server.

> **Platforms:** **Windows-only in v0.1.** Linux support is planned for v0.4, macOS for v2.0+. If you're not on Windows, you can still build and run the resolver from source today (see [Build from source](#build-from-source)) — the installer, tray, and service integration will be Windows-specific.

> **Status:** v0.1 in progress. The Rust resolver, the URLhaus threat-feed updater, the Tranco allowlist, and the `127.0.0.1` block-page are built and merged; the tray icon and Windows installer are in progress. No signed binary is published yet — the first signed release lands at the v0.1 launch. Until then, [build from source](#build-from-source).

> **Naming note (2026-05-07):** this DNS Shield project is named **Dropper**. The older Sentinel server/endpoint security platform has moved back to [`MuzeenMir/sentinel`](https://github.com/MuzeenMir/sentinel).

## Install

**Windows-only in v0.1** (see [Platforms](#dropper) above).

### Download (at v0.1 launch)

Signed Windows installers are published on the [Releases page](https://github.com/MuzeenMir/Dropper/releases/latest). Releases are signed with [cosign](https://github.com/sigstore/cosign) — verify the download before running it:

```sh
# Verify the installer against the published signature + certificate
cosign verify-blob \
  --certificate-identity-regexp 'https://github.com/MuzeenMir/Dropper/.+' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  --signature dropper-setup.exe.sig \
  --certificate dropper-setup.exe.pem \
  dropper-setup.exe
```

A successful verification prints `Verified OK`. (Binaries and their `.sig`/`.pem` are attached starting with the v0.1 release; until then the Releases page carries notes only.)

### Build from source

Works today on any platform Rust supports — useful for contributors and for non-Windows users who want to run the resolver:

```sh
git clone https://github.com/MuzeenMir/Dropper.git
cd Dropper
cargo build --release          # binary at target/release/dropper
cargo run -- --help            # see available commands
```

## Direction

A single-machine, single-purpose security tool aimed at users underserved by enterprise EDR/XDR products: home labs, indie devs, small studios, anyone who wants malicious-domain blocking without the SaaS console, the data exfil, or the per-seat bill.

- **DNS-layer blocking** against URLhaus + Tranco-anchored allowlist.
- **Quad9 / Cloudflare upstream** failover with status surfaced in the tray.
- **Block-page** on `127.0.0.1` explaining *what*, *why*, and *what to do* — with `Allow once` / `Allow forever` controls.
- **Tray icon** as the constant trust surface (green / amber / red).
- **VPN-aware** — auto-pause when another adapter takes over DNS (v0.2).
- **Open source, MIT, no telemetry, signed releases via cosign.**

Reference design: `DESIGN.md` (visual identity + component patterns). Roadmap: `TODOS.md` (T1–T3 + DX expansion).

## Trust & verification

Dropper is built to be inspected, not trusted on faith:

- **No telemetry.** Nothing leaves your machine — no accounts, no servers, no analytics. Verify it yourself in `src/`.
- **MIT-licensed**, fully open source. Read the resolver, the feed updater, and the block-page logic directly.
- **Signed releases.** Binaries are cosign-signed; verify before running (see [Install](#download-at-v01-launch)).
- **Questions?** See the **[FAQ](FAQ.md)** — DNS cache, DoH bypass, VPN behavior, false positives, and platform timelines.

## Repository layout

```
.
├── DESIGN.md         # visual identity, components, copy library
├── TODOS.md          # operational backlog (T1 archive, T2 Tranco refresh, T3 DX)
├── FAQ.md            # common questions (trust, DoH, VPN, false positives)
├── CLAUDE.md         # Claude Code session context
├── README.md         # this file
├── src/              # Rust resolver, feed updater (URLhaus + Tranco), block-page
├── templates/        # block-page HTML (DESIGN.md spec)
├── assets/           # tray icon SVGs + block-page artwork
└── .github/workflows # CI (rust, lint, security, a11y)
```

## Contributing

- Conventional Commits required (`commitlint.config.js`).
- Squash-merge only; signed commits required on `main`.
- `CODEOWNERS` gates review (solo-dev phase: everything routes to `@MuzeenMir`).
- Local dev loop: `cargo build --release` / `cargo run -- --help` (see [Build from source](#build-from-source)).

## License

MIT — see `LICENSE`.
