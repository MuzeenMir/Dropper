//! User-controlled allow-list with two layers:
//!
//! - `forever` — persisted to TOML at `persist_path`, survives restart
//! - `once`    — memory-only, 30-minute lazy-expiring TTL
//!
//! Both layers are normalized to lowercase on insert and lookup. The
//! resolver (wired in PR-2) checks `is_allowed` before consulting the
//! URLhaus blocklist.
//!
//! Atomic persistence: writes to `<path>.new` then renames over `<path>`,
//! so a crash mid-write never leaves a half-written file in place.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::RwLock;

/// Hardcoded for v0.1. Configurable via the operability-cluster TOML
/// config in a later phase.
const ALLOW_ONCE_TTL: Duration = Duration::from_secs(30 * 60);

/// Cheap-to-clone shared handle. Cloning bumps the `Arc` refcount only;
/// readers (resolver hot path, future block-page handler) and writers
/// (decision handler) share the same inner `AllowState`.
pub type AllowList = Arc<RwLock<AllowState>>;

/// Two-layer allow store + the path the `forever` layer persists to.
pub struct AllowState {
    forever: HashSet<String>,
    once: HashMap<String, OffsetDateTime>,
    persist_path: PathBuf,
}

/// On-disk shape. Sorted on serialize so the file is diff-friendly.
#[derive(Serialize, Deserialize, Default)]
struct PersistedAllowList {
    forever: Vec<String>,
}

/// Construct an empty allowlist that will persist `forever` entries to
/// `path`. The file at `path` is **not** read here; call [`load`] to
/// hydrate from disk.
pub fn new_allowlist(path: PathBuf) -> AllowList {
    Arc::new(RwLock::new(AllowState {
        forever: HashSet::new(),
        once: HashMap::new(),
        persist_path: path,
    }))
}

/// Is `domain` currently allowed?
///
/// Checks `forever` first (cheap `HashSet` hit), then `once` with an
/// expiry check. Expired `once` entries are pruned lazily on the read
/// path — no background sweeper task. Memory bound is the count of
/// allow-once clicks within any single 30-minute window.
pub async fn is_allowed(allowlist: &AllowList, domain: &str) -> bool {
    let domain = domain.to_lowercase();

    // Fast path under read lock. Returns early in three of four cases.
    let needs_prune = {
        let guard = allowlist.read().await;
        if guard.forever.contains(&domain) {
            return true;
        }
        match guard.once.get(&domain) {
            None => return false,
            Some(&expires_at) if OffsetDateTime::now_utc() < expires_at => return true,
            Some(_) => true,
        }
    };

    if needs_prune {
        let mut guard = allowlist.write().await;
        guard.once.remove(&domain);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!("sentinel-allowlist-test-{nanos}.toml"));
        p
    }

    #[tokio::test]
    async fn empty_allowlist_allows_nothing() {
        let al = new_allowlist(tmp_path());
        assert!(!is_allowed(&al, "example.com").await);
        assert!(!is_allowed(&al, "EXAMPLE.COM").await);
    }
}
