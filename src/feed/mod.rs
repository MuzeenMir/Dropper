//! Threat-feed integration.
//!
//! Wires up URLhaus (live blocklist) and a Tranco-anchored allowlist.
//! T2 in `TODOS.md` adds quarterly Tranco baseline auto-refresh.

pub mod tranco;
pub mod urlhaus;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Steady-state URLhaus refresh cadence. URLhaus is updated continuously
/// upstream; six hours keeps the local list fresh without hammering the
/// feed.
const URLHAUS_REFRESH_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// Retry cadence after a failed fetch. Far shorter than the steady-state
/// interval so a transient first-boot network hiccup doesn't leave the
/// resolver running with an empty blocklist for hours.
const URLHAUS_RETRY_AFTER_FAILURE: Duration = Duration::from_secs(5 * 60);

/// Tranco shifts slowly — the top 10k popularity ranking moves on the
/// scale of weeks-to-months, not hours. Quarterly is conservative and
/// keeps the allowlist honest over a multi-year installed lifetime.
const TRANCO_REFRESH_INTERVAL: Duration = Duration::from_secs(90 * 24 * 60 * 60);

/// Retry cadence after a failed Tranco fetch. One hour, deliberately
/// longer than the URLhaus retry: a stale allowlist is far less harmful
/// than a stale blocklist, so a fast retry would just hammer the feed
/// without buying meaningful freshness.
const TRANCO_RETRY_AFTER_FAILURE: Duration = Duration::from_secs(60 * 60);

/// Per-domain block metadata surfaced on the block-page (listing date,
/// threat classification). Populated from URLhaus's CSV feed when
/// available; left empty when we fall back to the hostfile feed.
///
/// `listed_date` is `YYYY-MM-DD` per `DESIGN.md` block-page copy.
/// `threat_type` is URLhaus's snake_case classification
/// (e.g. `malware_download`, `phishing`); the resolver humanizes it
/// for display.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockMetadata {
    pub listed_date: String,
    pub threat_type: String,
}

/// Shared, reader-friendly blocklist. Cheap to clone — wraps an `Arc`.
///
/// Many readers (resolver hot path, block-page server) hit it concurrently;
/// the writer (feed refresher) swaps the inner map wholesale on each cycle.
///
/// Domain key is always lowercase. The metadata value is empty for entries
/// sourced from the hostfile fallback (no listing date upstream).
pub type BlockList = Arc<RwLock<HashMap<String, BlockMetadata>>>;

/// Construct an empty `BlockList`.
pub fn new_blocklist() -> BlockList {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Shared, reader-friendly Tranco allowlist of the top-N most popular
/// domains. Domains in this set are never blackholed by the resolver,
/// regardless of URLhaus content — a hijack/false-positive guard for
/// popular sites. Cheap to clone — wraps an `Arc`. Keys are lowercase.
pub type Allowlist = Arc<RwLock<HashSet<String>>>;

/// Construct an empty `Allowlist`.
pub fn new_allowlist() -> Allowlist {
    Arc::new(RwLock::new(HashSet::new()))
}

/// Refresh URLhaus.
///
/// Tries the CSV feed first (carries `dateadded` + `threat`), falls back
/// to the hostfile feed (domain-only, no metadata) if CSV is unreachable
/// or returns an empty payload. Wholesale-replaces the blocklist contents
/// and returns the new entry count.
pub async fn refresh_urlhaus(blocklist: &BlockList) -> anyhow::Result<usize> {
    let new_map = match urlhaus::fetch_csv_online().await {
        Ok(m) if !m.is_empty() => m,
        Ok(_) => {
            eprintln!("urlhaus: csv_online returned empty payload, falling back to hostfile");
            hostfile_to_metadata_map(urlhaus::fetch_domains().await?)
        }
        Err(e) => {
            eprintln!("urlhaus: csv_online fetch failed ({e:#}), falling back to hostfile");
            hostfile_to_metadata_map(urlhaus::fetch_domains().await?)
        }
    };
    let count = new_map.len();
    let mut guard = blocklist.write().await;
    *guard = new_map;
    Ok(count)
}

/// Long-running URLhaus refresher loop. Refreshes on entry, then sleeps
/// `URLHAUS_REFRESH_INTERVAL` between successful cycles and
/// `URLHAUS_RETRY_AFTER_FAILURE` between failed ones.
///
/// Fail-open: if URLhaus is unreachable the blocklist stays at its last
/// known contents (empty on first boot) and every domain forwards to
/// upstream. We log the failure but never propagate it — "nothing
/// blocked yet" beats "DNS service refused to start because of an ISP
/// hiccup at install time."
///
/// `Ok(())` is unreachable in practice — the loop runs for the lifetime
/// of the process. The `Result<()>` shape matches the resolver /
/// blockpage tasks so the supervising `tokio::select!` can treat all
/// three uniformly.
pub async fn run_urlhaus_refresher(blocklist: BlockList) -> anyhow::Result<()> {
    loop {
        match refresh_urlhaus(&blocklist).await {
            Ok(count) => {
                eprintln!("urlhaus: loaded {count} domains");
                sleep(URLHAUS_REFRESH_INTERVAL).await;
            }
            Err(e) => {
                eprintln!(
                    "urlhaus: refresh failed: {e:#}; retrying in {:?}",
                    URLHAUS_RETRY_AFTER_FAILURE
                );
                sleep(URLHAUS_RETRY_AFTER_FAILURE).await;
            }
        }
    }
}

/// Look `domain` up in `blocklist`. Returns the per-domain metadata
/// (listing date, threat type) when blocked, or `None` when allowed.
///
/// Domains are stored lowercase at insertion time, so the lookup
/// lower-cases too.
pub async fn lookup(blocklist: &BlockList, domain: &str) -> Option<BlockMetadata> {
    blocklist.read().await.get(&domain.to_lowercase()).cloned()
}

/// Convenience wrapper: presence-only check. Used by call sites that
/// just need a yes/no decision (and tests).
pub async fn is_blocked(blocklist: &BlockList, domain: &str) -> bool {
    lookup(blocklist, domain).await.is_some()
}

/// Refresh the Tranco allowlist: download the zip, extract the top-N
/// domains, wholesale-replace the contents, return the new entry count.
pub async fn refresh_tranco(allowlist: &Allowlist) -> anyhow::Result<usize> {
    let domains = tranco::fetch_top_n().await?;
    let count = domains.len();
    let mut guard = allowlist.write().await;
    *guard = domains;
    Ok(count)
}

/// Long-running Tranco refresher loop. Quarterly steady-state cadence,
/// one-hour retry on failure.
///
/// Fail-open: if the fetch fails the existing allowlist (empty on first
/// boot, or last-known set from the previous cycle) stays in place.
/// On-disk persistence is a follow-up — for v0.1 the allowlist lives
/// in-memory and the resolver does not yet consult it, so the
/// empty-on-first-boot window has no observable effect.
///
/// `Ok(())` is unreachable in practice — the loop runs for the lifetime
/// of the process. The `Result<()>` shape matches the URLhaus and
/// resolver tasks so the supervising `tokio::select!` can treat all of
/// them uniformly.
pub async fn run_tranco_refresher(allowlist: Allowlist) -> anyhow::Result<()> {
    loop {
        match refresh_tranco(&allowlist).await {
            Ok(count) => {
                eprintln!("tranco: loaded {count} domains");
                sleep(TRANCO_REFRESH_INTERVAL).await;
            }
            Err(e) => {
                eprintln!(
                    "tranco: refresh failed: {e:#}; retrying in {:?}",
                    TRANCO_RETRY_AFTER_FAILURE
                );
                sleep(TRANCO_RETRY_AFTER_FAILURE).await;
            }
        }
    }
}

/// Look `domain` up in `allowlist`. Domains are stored lowercase, so the
/// lookup lower-cases too.
pub async fn is_allowed(allowlist: &Allowlist, domain: &str) -> bool {
    allowlist.read().await.contains(&domain.to_lowercase())
}

/// Convert a hostfile domain set into a metadata map with empty
/// metadata fields. Used as fallback when the CSV feed is unreachable;
/// the resolver substitutes a `—` placeholder for any empty
/// `listed_date` so the page still renders cleanly.
fn hostfile_to_metadata_map(domains: HashSet<String>) -> HashMap<String, BlockMetadata> {
    domains
        .into_iter()
        .map(|d| (d, BlockMetadata::default()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_blocklist_blocks_nothing() {
        let bl = new_blocklist();
        assert!(!is_blocked(&bl, "example.com").await);
        assert!(lookup(&bl, "example.com").await.is_none());
    }

    #[tokio::test]
    async fn manual_insert_then_lookup_is_case_insensitive() {
        let bl = new_blocklist();
        bl.write().await.insert(
            "malicious.example".to_string(),
            BlockMetadata {
                listed_date: "2026-04-22".to_string(),
                threat_type: "malware_download".to_string(),
            },
        );
        assert!(is_blocked(&bl, "malicious.example").await);
        assert!(is_blocked(&bl, "MALICIOUS.EXAMPLE").await);
        assert!(!is_blocked(&bl, "benign.example").await);

        let meta = lookup(&bl, "Malicious.Example").await.unwrap();
        assert_eq!(meta.listed_date, "2026-04-22");
        assert_eq!(meta.threat_type, "malware_download");
    }

    #[test]
    fn hostfile_fallback_yields_empty_metadata() {
        let mut set = HashSet::new();
        set.insert("a.example".to_string());
        set.insert("b.example".to_string());
        let map = hostfile_to_metadata_map(set);
        assert_eq!(map.len(), 2);
        for (_, meta) in map {
            assert!(meta.listed_date.is_empty());
            assert!(meta.threat_type.is_empty());
        }
    }

    #[tokio::test]
    async fn empty_allowlist_allows_nothing() {
        let al = new_allowlist();
        assert!(!is_allowed(&al, "example.com").await);
    }

    #[tokio::test]
    async fn allowlist_lookup_is_case_insensitive() {
        let al = new_allowlist();
        // Allowlist contract: callers insert lowercase keys (matching
        // what `tranco::parse_csv` emits). Lookups normalise too.
        al.write().await.insert("example.com".to_string());
        assert!(is_allowed(&al, "example.com").await);
        assert!(is_allowed(&al, "EXAMPLE.COM").await);
        assert!(is_allowed(&al, "Example.Com").await);
        assert!(!is_allowed(&al, "other.com").await);
    }
}
