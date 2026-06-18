# Contributing to Dropper

Thanks for your interest. Dropper is a small, single-purpose Rust project — the dev loop is fast and the bar is "CI is green and the change is honest about what it does."

## Prerequisites

- **Rust (stable).** `rust-toolchain.toml` pins the channel + components (`rustfmt`, `clippy`), so `rustup` selects the right toolchain automatically.
- **Git.**
- *Optional:* **Node 20** — only needed to reproduce the block-page accessibility check locally.

Dropper builds on any platform Rust supports. The installer, tray, and Windows service are Windows-specific; everything else (resolver, feed updater, block-page) builds and runs cross-platform from source.

## Dev loop

```sh
git clone https://github.com/MuzeenMir/Dropper.git
cd Dropper
cargo build
cargo run -- --help            # CLI surface
```

Run the whole stack locally **without installing anything** — this is the contributor dev loop today:

```sh
cargo run -- service           # boots resolver + block-page + URLhaus refresher
```

`service` starts the DNS resolver, the `127.0.0.1` block-page server, and the URLhaus feed refresher in one process. Binding the DNS port may require elevated privileges depending on your OS. (A dedicated `service --foreground --no-install` flag for cleaner dev ergonomics is on the T3 backlog; until it lands, plain `cargo run -- service` is the path.)

## Tests, format, lint

These mirror CI — run them before opening a PR:

```sh
cargo test                                              # CI runs: cargo test --verbose
cargo fmt --all                                         # CI checks: cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Iterating on tests? `scripts/watchtest` re-runs the suite on file changes.

### Block-page / template changes

The block-page has an **axe-core WCAG 2.1 AA gate** that runs in CI whenever `src/blockpage/**` or `templates/**` change. Reproduce it locally:

```sh
# render the template with fixture values (same substitution as src/blockpage render())
node .github/scripts/render-blockpage-fixture.mjs templates/blockpage.html /tmp/blockpage.html
# run axe-core against the rendered file
npx --yes @axe-core/cli@4.10.2 --tags wcag2a,wcag2aa,wcag21aa "file:///tmp/blockpage.html"
```

## Commits & PRs

- **Conventional Commits** — enforced by `commitlint` (`commitlint.config.js`).
- **One item, or one small related group, per PR.** Branch off `main`, open a PR, get CI green.
- **Squash-merge only.** `main` requires signed commits, so always land via the GitHub merge button (don't push to `main`).
- **`CODEOWNERS`** routes review to `@MuzeenMir` (solo-dev phase).

### CI gates (all must be green)

| Check | What it runs |
| ----- | ------------ |
| `rust` | `cargo fmt --check`, `clippy -D warnings`, build, `cargo test` on Ubuntu + Windows |
| `lint` | `commitlint` on the commit messages |
| `security` | `gitleaks` + `trivy` |
| `a11y` | axe-core on the rendered block-page (only on `src/blockpage/**` / `templates/**` changes) |

## Code map

```
src/resolver/   # DNS resolver (loopback, upstream failover)
src/feed/       # URLhaus blocklist + Tranco allowlist + local allowlist persistence
src/blockpage/  # 127.0.0.1 block-page server + /decision endpoint
templates/      # block-page HTML (DESIGN.md spec)
assets/         # tray icon SVGs + block-page artwork
```

## Reporting a security issue

Please **don't** open a public issue for a vulnerability — see [`SECURITY.md`](SECURITY.md) for the private reporting channel.
