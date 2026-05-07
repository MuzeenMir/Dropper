//! Tranco top-N allowlist anchor.
//!
//! Tranco publishes a daily-rebuilt ranking of the most popular domains.
//! We pull the top tier as an allowlist anchor: even if a popular domain
//! appears on URLhaus (compromise, hijack, false-positive on a CDN) the
//! resolver does not blackhole it. The list shifts over time as sites
//! rise and fall in popularity, so a long-installed Dropper refreshes
//! quarterly (T2 in `TODOS.md`).

use std::collections::HashSet;
use std::io::{Cursor, Read};

use anyhow::{Context, Result};

/// Tranco "always latest" stable URL. Returns a zip containing a single
/// `top-1m.csv` with `rank,domain` rows pre-sorted by ascending rank.
const TRANCO_URL: &str = "https://tranco-list.eu/top-1m.csv.zip";

/// Cap on the allowlist size. The full top-1M CSV is ~25 MB extracted;
/// the top tier is plenty as a hijack/false-positive guard, and stopping
/// at 10k keeps the resolver-side `HashSet` lookup cost negligible.
pub const TRANCO_TOP_N: usize = 10_000;

/// Fetch the Tranco zip, extract the embedded CSV, and parse the top
/// `TRANCO_TOP_N` rows into a deduped, lowercased domain set.
pub async fn fetch_top_n() -> Result<HashSet<String>> {
    let bytes = reqwest::get(TRANCO_URL)
        .await
        .context("Tranco fetch failed")?
        .error_for_status()
        .context("Tranco returned non-2xx")?
        .bytes()
        .await
        .context("Tranco body read failed")?;
    // Zip extraction is sync I/O on an in-memory buffer; offload to
    // spawn_blocking so a slow archive does not stall the runtime.
    let csv = tokio::task::spawn_blocking(move || -> Result<String> {
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).context("Tranco zip parse failed")?;
        // Documented to contain a single CSV; read by index so an
        // upstream filename change does not break the fetch.
        let mut entry = archive.by_index(0).context("Tranco zip is empty")?;
        let mut body = String::with_capacity(entry.size() as usize);
        entry
            .read_to_string(&mut body)
            .context("Tranco csv read failed")?;
        Ok(body)
    })
    .await
    .context("Tranco zip task panicked")??;
    Ok(parse_csv(&csv, TRANCO_TOP_N))
}

/// Parse a Tranco `rank,domain` CSV into a deduped, lowercased domain
/// set, stopping after `limit` rows. Rows that are blank, comments
/// (`#`), missing the comma, or have an empty domain field are skipped.
///
/// Tranco rows are pre-sorted by ascending rank, so the first `limit`
/// well-formed rows are the top `limit` domains.
pub fn parse_csv(body: &str, limit: usize) -> HashSet<String> {
    let mut domains = HashSet::with_capacity(limit);
    for raw in body.lines() {
        if domains.len() >= limit {
            break;
        }
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((_rank, domain)) = line.split_once(',') else {
            continue;
        };
        let domain = domain.trim().to_ascii_lowercase();
        if domain.is_empty() {
            continue;
        }
        domains.insert(domain);
    }
    domains
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_lowercases_and_dedupes() {
        let body = "1,EXAMPLE.com\n2,Google.com\n3,example.com\n";
        let domains = parse_csv(body, 10);
        assert_eq!(domains.len(), 2);
        assert!(domains.contains("example.com"));
        assert!(domains.contains("google.com"));
    }

    #[test]
    fn parse_csv_honors_limit_in_rank_order() {
        let body = "1,a.example\n2,b.example\n3,c.example\n4,d.example\n";
        let domains = parse_csv(body, 2);
        assert_eq!(domains.len(), 2);
        assert!(domains.contains("a.example"));
        assert!(domains.contains("b.example"));
        assert!(!domains.contains("c.example"));
    }

    #[test]
    fn parse_csv_skips_malformed_rows_and_comments() {
        let body = "\
# Tranco list, generated 2026-04-29
1,ok.example
no-comma-row
2,
3,trailing.example
";
        let domains = parse_csv(body, 100);
        assert_eq!(domains.len(), 2);
        assert!(domains.contains("ok.example"));
        assert!(domains.contains("trailing.example"));
    }

    #[test]
    fn parse_csv_handles_zero_limit() {
        let body = "1,a.example\n2,b.example\n";
        let domains = parse_csv(body, 0);
        assert!(domains.is_empty());
    }
}
