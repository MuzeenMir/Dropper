//! Threat-feed integration.
//!
//! Wires up URLhaus (live blocklist) and a Tranco-anchored allowlist.
//! T2 in `TODOS.md` adds quarterly Tranco baseline auto-refresh.

pub mod tranco;
pub mod urlhaus;

use std::collections::HashSet;
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

/// Shared, reader-friendly blocklist. Cheap to clone — wraps an `Arc`.
///
/// Many readers (resolver hot path, block-page server) hit it concurrently;
/// the writer (feed refresher) swaps the inner set wholesale on each cycle.
pub type BlockList = Arc<RwLock<HashSet<String>>>;

/// Construct an empty `BlockList`.
pub fn new_blocklist() -> BlockList {
    Arc::new(RwLock::new(HashSet::new()))
}

/// Fetch URLhaus and replace `blocklist`'s contents wholesale.
///
/// Returns the number of domains in the new set.
pub async fn refresh_urlhaus(blocklist: &BlockList) -> anyhow::Result<usize> {
    let domains = urlhaus::fetch_domains().await?;
    let count = domains.len();
    let mut guard = blocklist.write().await;
    *guard = domains;
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

/// Look `domain` up in `blocklist`. Domains are normalized to lowercase
/// at insertion time, so the lookup lower-cases too.
pub async fn is_blocked(blocklist: &BlockList, domain: &str) -> bool {
    blocklist.read().await.contains(&domain.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_blocklist_blocks_nothing() {
        let bl = new_blocklist();
        assert!(!is_blocked(&bl, "example.com").await);
    }

    #[tokio::test]
    async fn manual_insert_then_lookup_is_case_insensitive() {
        let bl = new_blocklist();
        bl.write().await.insert("malicious.example".to_string());
        assert!(is_blocked(&bl, "malicious.example").await);
        assert!(is_blocked(&bl, "MALICIOUS.EXAMPLE").await);
        assert!(!is_blocked(&bl, "benign.example").await);
    }
}
