# Show HN: Dropper — A Windows DNS shield that tells you what it blocked and why

Status: DRAFT. Locked 2026-04-30 as part of office-hours session 5 Hard Gate close.
Re-review for wedge staleness on 2026-05-13 (T+2 weeks). Do not post until full T3
is shipped, 5 user-watch sessions are completed, and at least one stranger has
v0.1.x running 7 days post-install.

---

## Title

Show HN: Dropper — A Windows DNS shield that tells you what it blocked and why

## Body (3 sentences — wedge first, honest limits last)

Dropper is a local DNS resolver for Windows that blocks malicious domains from
URLhaus, then shows you a block-page explaining what was blocked and why — so
you don't keep retrying the link, and you don't think Dropper is broken when a
site fails to load. Cloudflare 1.1.1.2 already does the malware-blocking part
for free, but it silently NXDOMAINs everything; you get a "this site can't be
reached" error and no idea whether your DNS just saved you or your internet is
broken. Dropper runs as a single signed binary, no SaaS, no telemetry,
MIT-licensed — and it's narrow on purpose: v0.1 is Windows-only, blocks
malware-only (URLhaus + Tranco allowlist), and is honest that it's one layer
of defense, not "fully protected."

## Screenshot

Path: `docs/screenshots/blockpage-urlhaus.png`

**Status: file not yet captured.** Capture during T3 Week 2 (block-page polish
pass) on a real Windows VM showing a real URLhaus-listed domain being blocked.
Show the titlebar identity + verdict pill + blocked domain + props block + the
3-tier action buttons. Do NOT capture against a synthetic/test domain — HN
readers will smell it. Use a real domain from URLhaus that day.

## Repo + verification (paste into HN post under the body)

GitHub: https://github.com/MuzeenMir/Dropper

To verify the binary you downloaded:

```
sha256sum dropper-windows-x64.msi    # compare to value in release notes
cosign verify-blob \
  --certificate dropper-windows-x64.msi.cert \
  --signature   dropper-windows-x64.msi.sig  \
  dropper-windows-x64.msi
```

## Banned phrases (do not let these creep in on edit)

- "fully protected"
- "forget about it"
- "all-in-one"
- "complete security"
- anything implying DNS shield = total protection

## Wedge sentence (this is what every other surface should echo)

> When Dropper blocks a domain, it tells you what the threat was and why — so
> you don't retry it, and so you don't suspect Dropper is broken when sites
> fail.

Reuse this sentence verbatim in:
- README.md trust-signal section
- INSTALL.md first paragraph
- Tray right-click "About" tooltip (if space allows)

## OQ2 stranger-skim test (from design doc 2026-04-29)

Pass criterion: watchee #1, given this draft cold, restates the wedge in their
own words within 30 seconds without re-reading. Fail = P2 wedge revises before
launch.
