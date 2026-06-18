# Security Policy

Dropper is a security tool, so its own security matters. Thanks for helping keep it trustworthy.

## Supported versions

Dropper is pre-1.0 and ships from a single line of development. Only the **latest release** receives security fixes; please reproduce issues against `main` or the newest tag before reporting.

| Version | Supported |
| ------- | --------- |
| latest (v0.1.x) | ✅ |
| older tags | ❌ |

## Reporting a vulnerability

**Do not open a public issue for a security vulnerability.**

Report privately through GitHub's confidential channel:

→ **[Report a vulnerability](https://github.com/MuzeenMir/Dropper/security/advisories/new)** (Security → Advisories → "Report a vulnerability")

This opens a private advisory visible only to you and the maintainer — no plaintext email, no public exposure before a fix is ready. If you can't use GitHub advisories, open a normal issue titled *"security contact request"* with **no technical details** and you'll be given a private channel.

When reporting, please include:

- affected component (resolver, URLhaus feed updater, allowlist handling, block-page / `/decision` endpoint, or installer),
- version / commit,
- reproduction steps or a proof-of-concept,
- impact (what an attacker gains).

## What's in scope

Dropper runs locally and binds loopback, so the threat model centers on **local and cross-origin attacks**, for example:

- bypassing or disabling blocking (e.g. forcing the resolver to allow a malicious domain),
- cross-origin / CSRF abuse of the loopback block-page or the `/decision` endpoint,
- the threat-feed update path (tampering, downgrade, or stale-feed handling),
- allowlist poisoning or unsafe persistence,
- privilege or DNS-configuration handling in the (forthcoming) installer/uninstaller.

Out of scope: vulnerabilities in upstream resolvers (Quad9 / Cloudflare), the URLhaus feed contents themselves, and issues requiring an already-fully-compromised machine.

## Disclosure

This is a solo-maintained project; expect a best-effort first response within a few days. Fixes are developed in a private advisory and disclosed coordinately once a patch is available. Reporters are credited unless they prefer to remain anonymous.

## Verifying releases

Released binaries are signed with [cosign](https://github.com/sigstore/cosign); verify before running (see [README → Install](README.md#download-at-v01-launch)).
