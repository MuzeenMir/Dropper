//! Operational error catalogue (E001–E004).
//!
//! Dropper's user-facing failures are a small, closed set. Each one carries
//! a stable `Exxx` code, a short machine-ish NAME, a plain-language cause, a
//! concrete fix, and the `/help/...` block-page route that explains it in
//! full. Surfaces that present these (installer dialog, tray tooltip, toast,
//! uninstaller dialog) land with their respective subsystems; this module is
//! the single source of truth they all read from so the code/cause/fix/link
//! never drift between a dialog and the help page.

use std::fmt;

/// A user-facing operational failure with a stable `Exxx` code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationalError {
    /// E001 — UDP/53 is already held by another process.
    Port53Occupied,
    /// E002 — the URLhaus blocklist is past its freshness window.
    FeedStale,
    /// E003 — uninstall could not revert DNS on every adapter.
    DnsRestorePartial,
    /// E004 — an active VPN / alternate DNS bypasses Dropper.
    VpnDetected,
}

impl OperationalError {
    /// Every operational error, in code order. Drives the catalogue tests
    /// and any "list all codes" surface (e.g. `dropper doctor`, the
    /// `/help/*` route table).
    pub const ALL: &'static [OperationalError] = &[
        OperationalError::Port53Occupied,
        OperationalError::FeedStale,
        OperationalError::DnsRestorePartial,
        OperationalError::VpnDetected,
    ];

    /// Stable `Exxx` identifier shown to users and quoted in bug reports.
    pub const fn code(self) -> &'static str {
        match self {
            OperationalError::Port53Occupied => "E001",
            OperationalError::FeedStale => "E002",
            OperationalError::DnsRestorePartial => "E003",
            OperationalError::VpnDetected => "E004",
        }
    }

    /// SCREAMING_SNAKE machine name (log keys, tray tooltip ids).
    pub const fn name(self) -> &'static str {
        match self {
            OperationalError::Port53Occupied => "PORT_53_OCCUPIED",
            OperationalError::FeedStale => "FEED_STALE",
            OperationalError::DnsRestorePartial => "DNS_RESTORE_PARTIAL",
            OperationalError::VpnDetected => "VPN_DETECTED",
        }
    }

    /// One-sentence plain-language cause.
    pub const fn cause(self) -> &'static str {
        match self {
            OperationalError::Port53Occupied => {
                "Another program is already using UDP port 53, the DNS port Dropper needs to answer queries."
            }
            OperationalError::FeedStale => {
                "The URLhaus blocklist has not refreshed within its freshness window, so newly listed threats may be missing."
            }
            OperationalError::DnsRestorePartial => {
                "During uninstall, Dropper could not restore the original DNS settings on one or more network adapters (an adapter was changed or removed since install)."
            }
            OperationalError::VpnDetected => {
                "A VPN or alternate DNS is active and routing queries around Dropper's local resolver, so blocking may be silently bypassed."
            }
        }
    }

    /// One concrete corrective action the user can take.
    pub const fn fix(self) -> &'static str {
        match self {
            OperationalError::Port53Occupied => {
                "Stop the conflicting DNS service (often Internet Connection Sharing, another DNS filter, or a local resolver) and restart Dropper, or let Dropper use its fallback port."
            }
            OperationalError::FeedStale => {
                "Check this machine's internet connection; Dropper keeps using the last good list and retries automatically."
            }
            OperationalError::DnsRestorePartial => {
                "Open Network adapter settings and set DNS back to Automatic (DHCP) on the affected adapters."
            }
            OperationalError::VpnDetected => {
                "Point the VPN at Dropper's resolver or enable split tunneling for DNS; otherwise expect reduced coverage while the VPN is on."
            }
        }
    }

    /// Block-page route (served on `127.0.0.1`) with the full explanation.
    pub const fn doc_path(self) -> &'static str {
        match self {
            OperationalError::Port53Occupied => "/help/port-conflict",
            OperationalError::FeedStale => "/help/feed",
            OperationalError::DnsRestorePartial => "/help/uninstall",
            OperationalError::VpnDetected => "/help/vpn",
        }
    }

    /// Look an error up by its `Exxx` code; `None` for anything unknown.
    pub fn from_code(code: &str) -> Option<OperationalError> {
        OperationalError::ALL
            .iter()
            .copied()
            .find(|e| e.code() == code)
    }
}

impl fmt::Display for OperationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {} Fix: {} See http://127.0.0.1{}",
            self.code(),
            self.name(),
            self.cause(),
            self.fix(),
            self.doc_path()
        )
    }
}

impl std::error::Error for OperationalError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_has_a_unique_sequential_code() {
        let codes: Vec<&str> = OperationalError::ALL.iter().map(|e| e.code()).collect();
        assert_eq!(codes, ["E001", "E002", "E003", "E004"]);
    }

    #[test]
    fn each_error_maps_to_its_spec_help_route() {
        assert_eq!(
            OperationalError::Port53Occupied.doc_path(),
            "/help/port-conflict"
        );
        assert_eq!(OperationalError::FeedStale.doc_path(), "/help/feed");
        assert_eq!(
            OperationalError::DnsRestorePartial.doc_path(),
            "/help/uninstall"
        );
        assert_eq!(OperationalError::VpnDetected.doc_path(), "/help/vpn");
    }

    #[test]
    fn doc_path_is_a_help_route_for_every_error() {
        for e in OperationalError::ALL {
            assert!(
                e.doc_path().starts_with("/help/"),
                "{} doc_path must be a /help/ route, got {}",
                e.code(),
                e.doc_path()
            );
        }
    }

    #[test]
    fn code_name_cause_and_fix_are_nonempty_for_every_error() {
        for e in OperationalError::ALL {
            assert!(e.code().starts_with('E'), "{} bad code", e.code());
            assert!(!e.name().is_empty(), "{} name empty", e.code());
            assert!(!e.cause().is_empty(), "{} cause empty", e.code());
            assert!(!e.fix().is_empty(), "{} fix empty", e.code());
        }
    }

    #[test]
    fn display_includes_code_name_cause_and_fix() {
        let e = OperationalError::Port53Occupied;
        let s = e.to_string();
        assert!(s.contains("E001"), "missing code in: {s}");
        assert!(s.contains("PORT_53_OCCUPIED"), "missing name in: {s}");
        assert!(s.contains(e.cause()), "missing cause in: {s}");
        assert!(s.contains(e.fix()), "missing fix in: {s}");
        assert!(s.contains(e.doc_path()), "missing help link in: {s}");
    }

    #[test]
    fn from_code_round_trips_known_codes_and_rejects_unknown() {
        for e in OperationalError::ALL {
            assert_eq!(OperationalError::from_code(e.code()), Some(*e));
        }
        assert_eq!(OperationalError::from_code("E999"), None);
        assert_eq!(OperationalError::from_code(""), None);
    }
}
