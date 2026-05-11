//! Block-page HTTP server bound to `127.0.0.1:80` (fallback `:8053`).
//!
//! Serves the static template at `templates/blockpage.html` (terminal-frame
//! chrome, key/value props, three-tier action button hierarchy per
//! `DESIGN.md`) with per-request token substitution. Three POST endpoints
//! capture the user's decision (`keep_blocked` / `allow_once` /
//! `allow_forever`) on the action buttons. The decision handler is a
//! placeholder until the resolver lands and there is an allow-list to
//! mutate; for now it logs the form payload and returns `204 No Content`.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use axum::{
    extract::{rejection::FormRejection, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::feed::allowlist::{allow_forever, allow_once, forget, new_allowlist, AllowList};

/// The compiled-in block-page template. The Rust binary is fully self-
/// contained; the HTML never has to be read off disk at runtime.
const TEMPLATE: &str = include_str!("../../templates/blockpage.html");

/// All the per-request fields the resolver fills in before rendering.
///
/// `domain` is the only field that originates from untrusted DNS query
/// data; [`render`] HTML-escapes every field on the way in to keep a
/// crafted query like `<script>alert(1)</script>.example` from breaking
/// out of the page.
#[derive(Clone, Debug)]
pub struct BlockReason {
    pub domain: String,
    pub feed: String,
    /// `YYYY-MM-DD` per `DESIGN.md` block-page copy.
    pub listed_date: String,
    /// `(N days ago)` companion phrase, including the parens.
    pub listed_relative: String,
    pub threat_type: String,
    /// 8-char hex block id (the forensic-trust footer field).
    pub block_id: String,
    /// RFC 3339 timestamp of the block decision.
    pub ts_iso: String,
}

impl BlockReason {
    /// A self-evident placeholder used when the server is hit before a
    /// real DNS query has populated the current-block slot (e.g. opening
    /// `http://127.0.0.1` in a browser by hand).
    pub fn placeholder() -> Self {
        Self {
            domain: "no-recent-block.local".to_string(),
            feed: "URLhaus".to_string(),
            listed_date: "—".to_string(),
            listed_relative: String::new(),
            threat_type: "no recent block".to_string(),
            block_id: "00000000".to_string(),
            ts_iso: "—".to_string(),
        }
    }
}

/// Render the block-page HTML for a given [`BlockReason`].
///
/// All fields are HTML-escaped before substitution. `{{version}}` is
/// pulled from `CARGO_PKG_VERSION` at compile time.
pub fn render(reason: &BlockReason) -> String {
    let version = env!("CARGO_PKG_VERSION");
    TEMPLATE
        .replace("{{domain}}", &html_escape(&reason.domain))
        .replace("{{feed}}", &html_escape(&reason.feed))
        .replace("{{listed_date}}", &html_escape(&reason.listed_date))
        .replace("{{listed_relative}}", &html_escape(&reason.listed_relative))
        .replace("{{threat_type}}", &html_escape(&reason.threat_type))
        .replace("{{block_id}}", &html_escape(&reason.block_id))
        .replace("{{ts_iso}}", &html_escape(&reason.ts_iso))
        .replace("{{version}}", &html_escape(version))
}

/// Minimal HTML-attribute-safe escape. Replaces the five characters that
/// can break out of either an element body or a `value="..."` attribute.
pub(crate) fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Shared state passed to handlers. Holds the most recent block decision
/// the resolver has produced; readers (the GET / handler) clone the
/// `Arc`, the resolver writer swaps the inner `Option` on each block.
#[derive(Clone)]
pub struct AppState {
    pub current: Arc<RwLock<Option<BlockReason>>>,
    pub allowlist: AllowList,
}

impl AppState {
    pub fn new() -> Self {
        Self::with_allowlist(new_allowlist(tmp_allowlist_path()))
    }

    pub fn with_allowlist(allowlist: AllowList) -> Self {
        Self {
            current: Arc::new(RwLock::new(None)),
            allowlist,
        }
    }

    pub async fn set_current(&self, reason: BlockReason) {
        *self.current.write().await = Some(reason);
    }
}

fn tmp_allowlist_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("dropper-appstate-allowlist-{nanos}.toml"))
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the axum router. `GET /` renders the block page, `POST
/// /decision` records the user's keep/allow choice.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_block))
        .route("/decision", post(handle_decision))
        .with_state(state)
}

async fn serve_block(State(state): State<AppState>) -> Html<String> {
    let reason = match state.current.read().await.clone() {
        Some(r) => r,
        None => BlockReason::placeholder(),
    };
    Html(render(&reason))
}

#[derive(Deserialize, Debug)]
pub struct DecisionForm {
    pub domain: String,
    pub block_id: String,
    pub action: DecisionAction,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    KeepBlocked,
    AllowOnce,
    AllowForever,
    Forget,
}

async fn handle_decision(
    State(state): State<AppState>,
    form: std::result::Result<Form<DecisionForm>, FormRejection>,
) -> impl IntoResponse {
    let Form(form) = match form {
        Ok(form) => form,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    let DecisionForm {
        domain,
        block_id: _,
        action,
    } = form;
    match action {
        DecisionAction::KeepBlocked => {}
        DecisionAction::AllowOnce => allow_once(&state.allowlist, &domain).await,
        DecisionAction::AllowForever => {
            if let Err(e) = allow_forever(&state.allowlist, &domain).await {
                eprintln!("allowlist: allow_forever failed: {e:#}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
        DecisionAction::Forget => {
            if let Err(e) = forget(&state.allowlist, &domain).await {
                eprintln!("allowlist: forget failed: {e:#}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }
    StatusCode::NO_CONTENT
}

/// Bind the block-page server to `127.0.0.1:80`, falling back to
/// `127.0.0.1:8053` if `:80` is already in use (e.g. IIS, nginx,
/// Docker Desktop on the user's box).
///
/// Blocks the calling task forever; returns only on serve error.
pub async fn serve(state: AppState) -> Result<()> {
    let primary = SocketAddr::from((Ipv4Addr::LOCALHOST, 80));
    let fallback = SocketAddr::from((Ipv4Addr::LOCALHOST, 8053));
    let listener = match tokio::net::TcpListener::bind(primary).await {
        Ok(l) => l,
        Err(_) => tokio::net::TcpListener::bind(fallback)
            .await
            .with_context(|| format!("bind to {primary} and fallback {fallback} both failed"))?,
    };
    axum::serve(listener, router(state))
        .await
        .context("block-page server exited")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::allowlist::{
        allow_forever as persist_allow_forever, is_allowed, load, new_allowlist,
    };

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn sample_reason() -> BlockReason {
        BlockReason {
            domain: "phishing-microsoft-login.example.com".to_string(),
            feed: "URLhaus".to_string(),
            listed_date: "2026-04-22".to_string(),
            listed_relative: "(6 days ago)".to_string(),
            threat_type: "malware host / credential harvest".to_string(),
            block_id: "7f3a2b91".to_string(),
            ts_iso: "2026-04-28T04:48:00Z".to_string(),
        }
    }

    fn test_allowlist_path() -> PathBuf {
        tmp_allowlist_path()
    }

    #[tokio::test]
    async fn appstate_default_constructs_empty_allowlist() {
        let state = AppState::new();
        assert!(!is_allowed(&state.allowlist, "anything.example").await);
    }

    #[test]
    fn render_substitutes_every_token() {
        let html = render(&sample_reason());
        assert!(html.contains("phishing-microsoft-login.example.com"));
        assert!(html.contains("URLhaus"));
        assert!(html.contains("2026-04-22"));
        assert!(html.contains("(6 days ago)"));
        assert!(html.contains("malware host / credential harvest"));
        assert!(html.contains("7f3a2b91"));
        assert!(html.contains("2026-04-28T04:48:00Z"));
        // Cargo pkg version (lib crate version) makes it through too.
        assert!(html.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn render_leaves_no_unsubstituted_tokens() {
        let html = render(&sample_reason());
        assert!(
            !html.contains("{{"),
            "found unsubstituted opening brace in rendered html"
        );
        assert!(
            !html.contains("}}"),
            "found unsubstituted closing brace in rendered html"
        );
    }

    #[test]
    fn render_html_escapes_crafted_domain() {
        let mut reason = sample_reason();
        reason.domain = r#"<script>alert("xss")</script>.example"#.to_string();
        let html = render(&reason);
        assert!(
            !html.contains("<script>"),
            "raw <script> tag leaked into rendered html"
        );
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot;xss&quot;"));
    }

    #[test]
    fn render_escapes_inside_form_value_attribute() {
        // The hidden inputs put the domain into value="..." — a domain
        // that contains a double quote must not break out of the attr.
        let mut reason = sample_reason();
        reason.domain = r#"a"b'c&d.example"#.to_string();
        let html = render(&reason);
        assert!(html.contains("&quot;"));
        assert!(html.contains("&#x27;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn placeholder_renders_without_panicking() {
        let html = render(&BlockReason::placeholder());
        assert!(html.contains("no-recent-block.local"));
        assert!(!html.contains("{{"));
    }

    #[test]
    fn html_escape_handles_ascii_and_unicode() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("<x>"), "&lt;x&gt;");
        assert_eq!(html_escape("x'y\"z"), "x&#x27;y&quot;z");
        // Non-ASCII passes through untouched.
        assert_eq!(html_escape("café 中"), "café 中");
    }

    #[tokio::test]
    async fn router_get_root_returns_html_with_placeholder() {
        let app = router(AppState::new());
        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("no-recent-block.local"));
        assert!(body_str.contains("<!doctype html>"));
    }

    #[tokio::test]
    async fn router_get_root_renders_current_block_when_set() {
        let state = AppState::new();
        state.set_current(sample_reason()).await;
        let app = router(state);
        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("phishing-microsoft-login.example.com"));
        assert!(body_str.contains("7f3a2b91"));
    }

    #[tokio::test]
    async fn router_post_decision_returns_204() {
        let app = router(AppState::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/decision")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "domain=phishing-microsoft-login.example.com&block_id=7f3a2b91&action=keep_blocked",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn router_post_decision_allow_once_mutates_allowlist() {
        let state = AppState::new();
        let app = router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/decision")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "domain=phish.example&block_id=7f3a2b91&action=allow_once",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(is_allowed(&state.allowlist, "phish.example").await);
    }

    #[tokio::test]
    async fn router_post_decision_allow_forever_persists_to_disk() {
        let path = test_allowlist_path();
        let allowlist = new_allowlist(path.clone());
        let state = AppState::with_allowlist(allowlist);
        let app = router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/decision")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "domain=phish.example&block_id=7f3a2b91&action=allow_forever",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(path.exists());

        let reloaded = new_allowlist(path.clone());
        assert_eq!(load(&reloaded).await.unwrap(), 1);
        assert!(is_allowed(&reloaded, "phish.example").await);

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn router_post_decision_forget_removes_from_allowlist() {
        let path = test_allowlist_path();
        let allowlist = new_allowlist(path.clone());
        persist_allow_forever(&allowlist, "phish.example")
            .await
            .unwrap();
        let state = AppState::with_allowlist(allowlist);
        let app = router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/decision")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "domain=phish.example&block_id=7f3a2b91&action=forget",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(!is_allowed(&state.allowlist, "phish.example").await);

        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn router_post_decision_rejects_unknown_action() {
        let app = router(AppState::new());
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/decision")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(
                        "domain=phish.example&block_id=7f3a2b91&action=nuke_from_orbit",
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
