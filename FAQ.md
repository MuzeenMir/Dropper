# Dropper FAQ

Common questions about what Dropper does, what it can't do, and how to verify it. For the visual/UX design, see `DESIGN.md`; for the roadmap, `TODOS.md`.

> **Status:** v0.1 is Windows-only and pre-signed-binary. Some answers below reference features tagged to a later version (v0.2 / v0.4 / v2.0+) — those are roadmap, not shipped. The version tag is called out where it applies.

---

## Trust & privacy

### Does Dropper send anything off my machine?

No. There is no telemetry, no account, no server, and no analytics. The resolver runs entirely on `127.0.0.1`, blocking decisions are made locally against a feed it downloads, and the block-page is served from your own machine. The only outbound network traffic is:

1. **Threat-feed updates** — Dropper downloads the URLhaus blocklist to refresh its local block list.
2. **Upstream DNS** — queries for domains *not* on the blocklist are forwarded to Quad9 / Cloudflare (your choice).

Everything else stays local. The code is MIT-licensed — read `src/` and confirm it yourself.

### How do I verify the download is genuine?

Releases are signed with [cosign](https://github.com/sigstore/cosign). Verify the installer before running it — the exact command is in the [README install section](README.md#download-at-v01-launch). A successful check prints `Verified OK`. (Signed binaries are attached starting with the v0.1 release.)

---

## How blocking works

### What does Dropper actually block?

Domains on the **URLhaus** malware/phishing feed, minus anything on a **Tranco-anchored allowlist** of popular legitimate domains (to keep false positives low). It is a DNS-layer block: when a process tries to resolve a blocked domain, Dropper returns the loopback address so the connection lands on the local block-page instead of the malicious host.

### Why did Dropper block a site I trust? (False-positive procedure)

Feeds are imperfect. When you hit a block you believe is wrong:

1. On the block-page, use **Allow once** (this session) or **Allow forever** (adds the domain to your local allowlist).
2. If you think the feed itself is wrong, report it via the **[False-positive report](https://github.com/MuzeenMir/Dropper/issues/new?template=fp-report.yml)** issue template so the allowlist can be improved for everyone.

Your `Allow forever` choices are stored locally in your allowlist and are never uploaded.

### My change didn't take effect / a blocked site still loads

Windows caches DNS answers in the **DNS Client service ("Dnscache")**, and browsers keep their own in-memory cache. A previously-resolved domain can stay cached after Dropper starts blocking it (or vice-versa). Flush both:

```sh
ipconfig /flushdns          # Windows DNS Client cache
```

…and restart the browser (or clear its internal DNS cache). Cached TTLs expire on their own within minutes if you'd rather wait.

---

## Limitations & edge cases

### Does Dropper block DNS-over-HTTPS (DoH)?

Not in v0.1. Apps that do their own **DoH** — notably Firefox (on by default) and Chrome — send DNS queries over HTTPS directly to their own resolver, bypassing the OS resolver Dropper sits on. Dropper never sees those queries and can't block them. To stay protected today, **disable DoH in your browser** so DNS goes through the system resolver. Tightening this is on the roadmap.

### What happens when I connect to a VPN?

Many VPNs push their own DNS server onto their network adapter, so queries route around `127.0.0.1` and Dropper stops seeing them — protection silently lapses until you disconnect. In v0.1 you should be aware of this. **v0.2** adds VPN-awareness: Dropper detects when another adapter takes over DNS, auto-pauses, and raises an `E004 VPN_DETECTED` notice (per-adapter handling) so the lapse is never silent.

### Does it slow down my browsing?

Blocking is a local lookup against an in-memory set, so the overhead per query is negligible. Non-blocked queries are forwarded to your chosen upstream (Quad9 / Cloudflare), so steady-state latency is essentially that of your upstream resolver.

---

## Platforms & roadmap

### When will there be a macOS / Linux version?

- **Windows** — v0.1 (now).
- **Linux** — planned for **v0.4**. The core resolver already builds and runs from source on Linux today (`cargo build --release`); what's missing is the installer, service integration, and tray.
- **macOS** — planned for **v2.0+**.

v0.1 is deliberately Windows-only and narrow — see the README for why.

### How do I uninstall, and does it restore my DNS?

The Windows uninstaller (shipping with v0.1) restores your previous DNS configuration. Because laptops change network adapters (Wi-Fi ↔ Ethernet ↔ VPN), edge cases around partial restore are tracked as `E003 DNS_RESTORE_PARTIAL`; if your DNS looks wrong after uninstall, set your adapter's DNS back to automatic (DHCP) in Windows network settings.

---

*Question not answered here? Open a [discussion](https://github.com/MuzeenMir/Dropper/discussions) or file an [issue](https://github.com/MuzeenMir/Dropper/issues/new/choose).*
