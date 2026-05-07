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

/// User clicked "Allow once" on the block-page. Domain is allowed for
/// [`ALLOW_ONCE_TTL`] (30 minutes) and then auto-re-blocks.
pub async fn allow_once(allowlist: &AllowList, domain: &str) {
    allow_once_with_ttl(allowlist, domain, ALLOW_ONCE_TTL).await
}

/// Test helper: same as [`allow_once`] but with caller-controlled TTL so
/// tests can verify expiry behavior without sleeping for 30 minutes.
/// Crate-private — production callers always use [`allow_once`].
pub(crate) async fn allow_once_with_ttl(allowlist: &AllowList, domain: &str, ttl: Duration) {
    let domain = domain.to_lowercase();
    let expires_at = OffsetDateTime::now_utc() + time::Duration::seconds(ttl.as_secs() as i64);
    let mut guard = allowlist.write().await;
    guard.once.insert(domain, expires_at);
}

/// User clicked "Allow forever" on the block-page. Domain is added to
/// the persistent allowlist and the on-disk file is rewritten atomically.
pub async fn allow_forever(allowlist: &AllowList, domain: &str) -> Result<()> {
    let domain = domain.to_lowercase();
    let mut guard = allowlist.write().await;
    guard.forever.insert(domain);
    persist(&guard).await
}

/// Hydrate the `forever` layer from `persist_path`. Returns the count
/// of loaded entries.
///
/// Missing file is **not** an error — first run starts with an empty
/// list. Malformed TOML **is** an error; the caller decides whether to
/// fall back or hard-fail.
pub async fn load(allowlist: &AllowList) -> Result<usize> {
    let path = {
        let guard = allowlist.read().await;
        guard.persist_path.clone()
    };

    if !path.exists() {
        return Ok(0);
    }

    let bytes = tokio::fs::read(&path)
        .await
        .with_context(|| format!("reading allowlist {}", path.display()))?;
    let text = String::from_utf8(bytes).context("allowlist is not valid utf-8")?;
    let parsed: PersistedAllowList = toml::from_str(&text)
        .with_context(|| format!("parsing allowlist {}", path.display()))?;

    let mut guard = allowlist.write().await;
    guard.forever = parsed
        .forever
        .into_iter()
        .map(|d| d.to_lowercase())
        .collect();
    Ok(guard.forever.len())
}

/// Atomic write: serialize → write to `<path>.new` → rename over
/// `<path>`. POSIX rename is atomic on the same filesystem; on NTFS,
/// `MoveFileEx` with replace semantics gives the same guarantee.
///
/// Held under the caller's write guard, so two concurrent
/// `allow_forever` calls serialize through the lock and the on-disk
/// state always matches the in-memory state at the moment of the
/// successful rename.
async fn persist(state: &AllowState) -> Result<()> {
    let mut sorted: Vec<String> = state.forever.iter().cloned().collect();
    sorted.sort();
    let payload = PersistedAllowList { forever: sorted };
    let serialized = toml::to_string(&payload).context("serializing allowlist")?;

    let path = &state.persist_path;
    let tmp = path.with_extension("toml.new");

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("creating parent dir {}", parent.display()))?;
        }
    }

    tokio::fs::write(&tmp, serialized.as_bytes())
        .await
        .with_context(|| format!("writing temp {}", tmp.display()))?;
    tokio::fs::rename(&tmp, path)
        .await
        .with_context(|| format!("renaming {} to {}", tmp.display(), path.display()))?;

    Ok(())
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

    #[tokio::test]
    async fn allow_once_is_allowed_within_ttl() {
        let al = new_allowlist(tmp_path());
        allow_once_with_ttl(&al, "phish.example", Duration::from_secs(60)).await;
        assert!(is_allowed(&al, "phish.example").await);
        assert!(is_allowed(&al, "PHISH.EXAMPLE").await);
    }

    #[tokio::test]
    async fn allow_once_denied_after_ttl_and_prunes_entry() {
        let al = new_allowlist(tmp_path());
        // 1ms TTL — guaranteed expired by the time tokio yields.
        allow_once_with_ttl(&al, "phish.example", Duration::from_millis(1)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(!is_allowed(&al, "phish.example").await);

        // Verify the read-path prune actually removed the expired entry.
        let guard = al.read().await;
        assert!(!guard.once.contains_key("phish.example"));
    }

    #[tokio::test]
    async fn allow_forever_roundtrips_through_disk() {
        let path = tmp_path();

        // First instance writes.
        let al1 = new_allowlist(path.clone());
        allow_forever(&al1, "Phish.Example").await.unwrap();
        allow_forever(&al1, "another.example").await.unwrap();

        // Drop and re-construct against the same file.
        drop(al1);
        let al2 = new_allowlist(path.clone());
        let n = load(&al2).await.unwrap();
        assert_eq!(n, 2);
        assert!(is_allowed(&al2, "phish.example").await);
        assert!(is_allowed(&al2, "another.example").await);
        assert!(!is_allowed(&al2, "benign.example").await);

        // Cleanup.
        let _ = std::fs::remove_file(&path);
    }
}
