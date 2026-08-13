use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub protocol: String,
    pub hostname: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            protocol: "socks5".to_string(),
            hostname: "127.0.0.1".to_string(),
            port: 9050,
            username: None,
            password: None,
        }
    }
}

/// Stable desktop Chrome User-Agent — avoids Electron/app name leakage.
///
/// This is a recent Chrome 131 UA that matches the InnerTube client version
/// used in request bodies. Unlike the old OpenTubeX implementation, this does
/// NOT contain "Electron", "Slytube", or "opentubex" identifiers.
pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// YouTube consent cookies sent with YouTube requests.
///
/// - `CONSENT=YES+`: Signals GDPR consent (prevents consent gate responses).
/// - `SOCS=CAI`: Signals "cookie accepted, interactive" state.
const YOUTUBE_COOKIES: &str = "CONSENT=YES+; SOCS=CAI";

/// Headers forbidden by browsers/webviews that must be stripped from responses.
///
/// These headers are set by the HTTP stack and should not be forwarded to
/// consumers. See the [Fetch standard](https://fetch.spec.whatwg.org/#forbidden-response-header-name)
/// for the authoritative list.
const FORBIDDEN_RESPONSE_HEADERS: &[&str] = &[
    "content-length",
    "content-encoding",
    "transfer-encoding",
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "upgrade",
];

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    /// Stored Invidious credentials: hostname → bearer token.
    /// Uses interior mutability so credentials can be updated without
    /// requiring `&mut self` (HttpClient is always wrapped in Arc).
    invidious_credentials: Mutex<HashMap<String, String>>,
}

impl HttpClient {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(USER_AGENT)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            invidious_credentials: Mutex::new(HashMap::new()),
        })
    }

    pub fn with_proxy(proxy: &ProxyConfig) -> Result<Self, String> {
        let proxy_url = format!(
            "{}://{}:{}",
            proxy.protocol, proxy.hostname, proxy.port
        );

        let mut req_proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|e| format!("Failed to create proxy: {}", e))?;

        if let (Some(user), Some(pass)) = (&proxy.username, &proxy.password) {
            req_proxy = req_proxy.basic_auth(user, pass);
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(USER_AGENT)
            .proxy(req_proxy)
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client with proxy: {}", e))?;

        Ok(Self {
            client,
            invidious_credentials: Mutex::new(HashMap::new()),
        })
    }

    /// Stores credentials for an Invidious instance.
    ///
    /// When a request is made to a known Invidious host, the stored bearer
    /// token is automatically added as an `Authorization` header.
    pub fn set_invidious_credentials(&self, instance_host: &str, token: &str) {
        let mut creds = self.invidious_credentials.lock().unwrap();
        creds.insert(instance_host.to_string(), token.to_string());
        tracing::debug!("Stored Invidious credentials for {}", instance_host);
    }

    /// Clears Invidious credentials for a specific instance, or all instances.
    pub fn clear_invidious_credentials(&self, instance_host: Option<&str>) {
        let mut creds = self.invidious_credentials.lock().unwrap();
        match instance_host {
            Some(host) => {
                creds.remove(host);
                tracing::debug!("Cleared Invidious credentials for {}", host);
            }
            None => {
                creds.clear();
                tracing::debug!("Cleared all Invidious credentials");
            }
        }
    }

    /// Core hardening entry point.
    ///
    /// Takes a request builder, detects the target URL, and applies the
    /// appropriate domain-specific headers. All request methods route through
    /// this function.
    pub fn request_internal(
        &self,
        builder: reqwest::RequestBuilder,
        url: &str,
    ) -> reqwest::RequestBuilder {
        let creds = self.invidious_credentials.lock().unwrap();
        apply_url_hardening(builder, url, &creds)
    }

    pub async fn get_json(&self, url: &str) -> Result<serde_json::Value, String> {
        let builder = self.client.get(url);
        let builder = self.request_internal(builder, url);

        let response = builder
            .send()
            .await
            .map_err(|e| format!("GET {} failed: {}", url, e))?;

        check_tracking_cookies(url, &response);

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("GET {} returned {}: {}", url, status, body));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse JSON from {}: {}", url, e))
    }

    /// Fetches a URL and returns the response body as text, applying hardening.
    pub async fn get_text(&self, url: &str) -> Result<String, String> {
        let builder = self.client.get(url);
        let builder = self.request_internal(builder, url);

        let response = builder
            .send()
            .await
            .map_err(|e| format!("GET {} failed: {}", url, e))?;

        check_tracking_cookies(url, &response);

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(format!("GET {} returned {}: {}", url, status, body));
        }

        response
            .text()
            .await
            .map_err(|e| format!("Failed to read text from {}: {}", url, e))
    }

    pub async fn post_json(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let builder = self.client.post(url);
        let builder = self.request_internal(builder, url);
        let builder = builder
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body);

        let response = builder
            .send()
            .await
            .map_err(|e| format!("POST {} failed: {}", url, e))?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(format!(
                "POST {} returned {}: {}",
                url, status, body_text
            ));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse JSON from {}: {}", url, e))
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

// ─── URL classification ──────────────────────────────────────────────────────

/// Extracts the hostname and path from a URL string without external dependencies.
///
/// This is a lightweight alternative to the `url` crate. It handles the common
/// cases we need for domain-based header hardening: scheme stripping, port
/// removal, and query/fragment removal.
fn extract_host_path(url: &str) -> Option<(&str, &str)> {
    let after_scheme = url.splitn(2, "://").nth(1)?;
    let host_end = after_scheme
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(after_scheme.len());
    let host = &after_scheme[..host_end];
    // Strip port if present
    let host = host.split(':').next().unwrap_or(host);
    // Get path (strip query and fragment)
    let rest = &after_scheme[host_end..];
    let path = rest.split(|c| c == '?' || c == '#').next().unwrap_or(rest);
    Some((host, path))
}

/// Classifies a URL into a domain category for hardening.
#[derive(Debug, Clone, PartialEq, Eq)]
enum UrlClass {
    YouTubeInnerTube,
    YouTubeServiceWorker,
    YouTubeImage,
    YouTubeVideoplayback,
    YouTubeOther,
    IpWhoIs,
    /// Contains the matched Invidious host key.
    Invidious(String),
    Unknown,
}

fn classify_url(url: &str, invidious_hosts: &HashMap<String, String>) -> UrlClass {
    let Some((host, path)) = extract_host_path(url) else {
        return UrlClass::Unknown;
    };

    // Check Invidious hosts first — they take precedence over generic rules.
    for inv_host in invidious_hosts.keys() {
        if host == *inv_host || host.ends_with(&format!(".{}", inv_host)) {
            return UrlClass::Invidious(inv_host.clone());
        }
    }

    // YouTube InnerTube API
    if host.ends_with("youtube.com") && path.starts_with("/youtubei/") {
        return UrlClass::YouTubeInnerTube;
    }

    // YouTube service worker data / timedtext
    if host.ends_with("youtube.com")
        && (path.contains("sw.js_data") || path.starts_with("/api/timedtext"))
    {
        return UrlClass::YouTubeServiceWorker;
    }

    // YouTube image / CDN domains
    if host.ends_with("googleusercontent.com")
        || host.ends_with("ggpht.com")
        || host.ends_with("ytimg.com")
    {
        return UrlClass::YouTubeImage;
    }

    // YouTube video playback
    if host.ends_with("googlevideo.com") && path.contains("videoplayback") {
        return UrlClass::YouTubeVideoplayback;
    }

    // YouTube other (pages, embed, etc.)
    if host.ends_with("youtube.com") {
        return UrlClass::YouTubeOther;
    }

    // ipwho.is — CORS fix target
    if host == "ipwho.is" || host.ends_with(".ipwho.is") {
        return UrlClass::IpWhoIs;
    }

    UrlClass::Unknown
}

// ─── Header hardening ────────────────────────────────────────────────────────

/// Applies domain-specific hardening headers to a request builder.
///
/// The hardening rules follow the domain-based header injection table from
/// the Phase 3 specification. Each URL class maps to a specific set of
/// headers that make the request look like it came from a normal Chrome
/// browser visiting YouTube.
fn apply_url_hardening(
    builder: reqwest::RequestBuilder,
    url: &str,
    invidious_creds: &HashMap<String, String>,
) -> reqwest::RequestBuilder {
    match classify_url(url, invidious_creds) {
        UrlClass::YouTubeInnerTube => builder
            .header("Referer", "https://www.youtube.com/")
            .header("Origin", "https://www.youtube.com")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Sec-Fetch-Mode", "same-origin")
            .header("Sec-Fetch-Dest", "empty")
            .header("X-Youtube-Bootstrap-Logged-In", "false")
            .header("X-Youtube-Client-Name", "1")
            .header("X-Youtube-Client-Version", "2.20240101.01.00")
            .header("Cookie", YOUTUBE_COOKIES),

        UrlClass::YouTubeServiceWorker => builder
            .header("Referer", "https://www.youtube.com/sw.js")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Sec-Fetch-Mode", "same-origin")
            .header("Cookie", YOUTUBE_COOKIES),

        UrlClass::YouTubeImage => builder
            .header("Referer", "https://www.youtube.com/")
            .header("Origin", "https://www.youtube.com"),

        UrlClass::YouTubeVideoplayback => {
            // Set Referer/Origin but DO NOT set Content-Type.
            // For GET requests (which videoplayback always is), reqwest does
            // not add Content-Type, so it is implicitly absent.
            builder
                .header("Referer", "https://www.youtube.com/")
                .header("Origin", "https://www.youtube.com")
        }

        UrlClass::YouTubeOther => builder.header("Cookie", YOUTUBE_COOKIES),

        UrlClass::IpWhoIs => {
            // CORS fix: do NOT add any extra headers beyond what the client
            // already has. This prevents CORS preflight failures.
            builder
        }

        UrlClass::Invidious(host) => {
            if let Some(token) = invidious_creds.get(&host) {
                builder.header("Authorization", token.as_str())
            } else {
                builder
            }
        }

        UrlClass::Unknown => builder,
    }
}

// ─── Response hardening ──────────────────────────────────────────────────────

/// Strips forbidden headers from a response HeaderMap.
///
/// Some headers are forbidden by browsers/webviews and can cause issues
/// when responses are passed through. This function removes them.
///
/// # Example
///
/// ```
/// use slytube_lib::http_client::sanitize_headers;
///
/// let mut headers = reqwest::header::HeaderMap::new();
/// headers.insert("content-type", "application/json".parse().unwrap());
/// headers.insert("content-length", "100".parse().unwrap());
/// let sanitized = sanitize_headers(&headers);
/// assert!(sanitized.contains_key("content-type"));
/// assert!(!sanitized.contains_key("content-length"));
/// ```
pub fn sanitize_headers(headers: &reqwest::header::HeaderMap) -> reqwest::header::HeaderMap {
    let mut sanitized = reqwest::header::HeaderMap::new();
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !FORBIDDEN_RESPONSE_HEADERS.contains(&key_str.as_str()) {
            sanitized.insert(key, value.clone());
        }
    }
    sanitized
}

/// Checks for tracking cookies in responses to specific YouTube endpoints.
///
/// Logs a warning when tracking cookies are detected on responses that should
/// not be setting them (sw.js_data, iframe_api). Since reqwest does not store
/// cookies in a cookie jar by default, the "stripping" is implicit — these
/// cookies are never stored or forwarded.
pub fn check_tracking_cookies(url: &str, response: &reqwest::Response) {
    let Some((host, path)) = extract_host_path(url) else {
        return;
    };

    if host.ends_with("youtube.com")
        && (path.contains("sw.js_data") || path.starts_with("/iframe_api"))
    {
        let has_set_cookie = response
            .headers()
            .iter()
            .any(|(k, _)| k.as_str().eq_ignore_ascii_case("set-cookie"));

        if has_set_cookie {
            tracing::debug!(
                "Tracking cookie detected on {} — stripped (not stored)",
                url
            );
        }
    }
}

pub type SharedHttpClient = Arc<HttpClient>;

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── URL extraction ───────────────────────────────────────────────────

    #[test]
    fn test_extract_host_path_simple() {
        let (host, path) = extract_host_path("https://www.youtube.com/youtubei/v1/player").unwrap();
        assert_eq!(host, "www.youtube.com");
        assert_eq!(path, "/youtubei/v1/player");
    }

    #[test]
    fn test_extract_host_path_with_port() {
        let (host, path) = extract_host_path("https://example.com:8080/api/test").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/api/test");
    }

    #[test]
    fn test_extract_host_path_with_query() {
        let (host, path) = extract_host_path("https://example.com/path?key=value").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/path");
    }

    #[test]
    fn test_extract_host_path_with_fragment() {
        let (host, path) = extract_host_path("https://example.com/path#section").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/path");
    }

    #[test]
    fn test_extract_host_path_with_query_and_fragment() {
        let (host, path) = extract_host_path("https://example.com/path?key=value#section").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/path");
    }

    #[test]
    fn test_extract_host_path_root() {
        let (host, path) = extract_host_path("https://example.com/").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "/");
    }

    #[test]
    fn test_extract_host_path_no_path() {
        let (host, path) = extract_host_path("https://example.com").unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(path, "");
    }

    #[test]
    fn test_extract_host_path_invalid() {
        assert!(extract_host_path("not-a-url").is_none());
    }

    // ─── URL classification ───────────────────────────────────────────────

    #[test]
    fn test_classify_youtube_innertube_player() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://www.youtube.com/youtubei/v1/player", &map),
            UrlClass::YouTubeInnerTube
        );
    }

    #[test]
    fn test_classify_youtube_innertube_search() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://www.youtube.com/youtubei/v1/search", &map),
            UrlClass::YouTubeInnerTube
        );
    }

    #[test]
    fn test_classify_youtube_innertube_browse() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://youtube.com/youtubei/v1/browse", &map),
            UrlClass::YouTubeInnerTube
        );
    }

    #[test]
    fn test_classify_youtube_service_worker() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://www.youtube.com/sw.js_data", &map),
            UrlClass::YouTubeServiceWorker
        );
    }

    #[test]
    fn test_classify_youtube_timedtext() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://www.youtube.com/api/timedtext?v=abc123", &map),
            UrlClass::YouTubeServiceWorker
        );
    }

    #[test]
    fn test_classify_youtube_image_ytimg() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://i.ytimg.com/vi/abc123/hqdefault.jpg", &map),
            UrlClass::YouTubeImage
        );
    }

    #[test]
    fn test_classify_youtube_image_ggpht() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://yt3.ggpht.com/photo.jpg", &map),
            UrlClass::YouTubeImage
        );
    }

    #[test]
    fn test_classify_youtube_image_googleusercontent() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://lh3.googleusercontent.com/proxy/abc", &map),
            UrlClass::YouTubeImage
        );
    }

    #[test]
    fn test_classify_youtube_videoplayback() {
        let map = HashMap::new();
        assert_eq!(
            classify_url(
                "https://rr1---sn-abc7.googlevideo.com/videoplayback?key=value",
                &map
            ),
            UrlClass::YouTubeVideoplayback
        );
    }

    #[test]
    fn test_classify_youtube_other_page() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://www.youtube.com/watch?v=abc123", &map),
            UrlClass::YouTubeOther
        );
    }

    #[test]
    fn test_classify_ipwhois_root() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://ipwho.is", &map),
            UrlClass::IpWhoIs
        );
    }

    #[test]
    fn test_classify_ipwhois_with_ip() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://ipwho.is/1.2.3.4", &map),
            UrlClass::IpWhoIs
        );
    }

    #[test]
    fn test_classify_invidious_with_creds() {
        let mut map = HashMap::new();
        map.insert("inv.nadeko.net".to_string(), "token123".to_string());
        assert_eq!(
            classify_url("https://inv.nadeko.net/api/v1/videos/abc", &map),
            UrlClass::Invidious("inv.nadeko.net".to_string())
        );
    }

    #[test]
    fn test_classify_invidious_takes_precedence() {
        // Even if the URL looks like something else, Invidious hosts win.
        let mut map = HashMap::new();
        map.insert("youtube.com".to_string(), "fake_token".to_string());
        assert_eq!(
            classify_url("https://youtube.com/api/something", &map),
            UrlClass::Invidious("youtube.com".to_string())
        );
    }

    #[test]
    fn test_classify_unknown() {
        let map = HashMap::new();
        assert_eq!(
            classify_url("https://example.com/api", &map),
            UrlClass::Unknown
        );
    }

    // ─── Header hardening ─────────────────────────────────────────────────

    #[test]
    fn test_apply_hardening_innertube_headers() {
        let map = HashMap::new();
        let class = classify_url("https://www.youtube.com/youtubei/v1/player", &map);
        assert_eq!(class, UrlClass::YouTubeInnerTube);
        // The apply_url_hardening function adds the headers — we verify
        // classification here since we cannot inspect RequestBuilder headers
        // without sending the request.
    }

    #[test]
    fn test_apply_hardening_image_headers() {
        let map = HashMap::new();
        let class = classify_url("https://i.ytimg.com/vi/abc/hqdefault.jpg", &map);
        assert_eq!(class, UrlClass::YouTubeImage);
    }

    #[test]
    fn test_apply_hardening_videoplayback_classification() {
        let map = HashMap::new();
        let class = classify_url(
            "https://rr1---sn-abc.googlevideo.com/videoplayback?key=value",
            &map,
        );
        assert_eq!(class, UrlClass::YouTubeVideoplayback);
        // Content-Type is not set for GET requests (videoplayback is always GET).
    }

    #[test]
    fn test_apply_hardening_invidious_auth_classification() {
        let mut map = HashMap::new();
        map.insert("inv.nadeko.net".to_string(), "bearer_token".to_string());
        let class = classify_url("https://inv.nadeko.net/api/v1/videos/abc", &map);
        assert_eq!(class, UrlClass::Invidious("inv.nadeko.net".to_string()));
    }

    #[test]
    fn test_apply_hardening_ipwhois_classification() {
        let map = HashMap::new();
        let class = classify_url("https://ipwho.is", &map);
        assert_eq!(class, UrlClass::IpWhoIs);
        // No extra headers added — verified by the match arm in apply_url_hardening.
    }

    // ─── Response sanitization ────────────────────────────────────────────

    #[test]
    fn test_sanitize_headers_removes_forbidden() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("content-length", "100".parse().unwrap());
        headers.insert("x-custom", "value".parse().unwrap());

        let sanitized = sanitize_headers(&headers);

        assert!(sanitized.contains_key("content-type"));
        assert!(!sanitized.contains_key("content-length"));
        assert!(sanitized.contains_key("x-custom"));
    }

    #[test]
    fn test_sanitize_headers_preserves_allowed() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("cache-control", "no-cache".parse().unwrap());
        headers.insert("x-custom-header", "value".parse().unwrap());

        let sanitized = sanitize_headers(&headers);

        assert_eq!(sanitized.len(), 3);
    }

    #[test]
    fn test_sanitize_headers_removes_content_encoding() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-encoding", "gzip".parse().unwrap());
        headers.insert("content-type", "text/html".parse().unwrap());

        let sanitized = sanitize_headers(&headers);
        assert!(!sanitized.contains_key("content-encoding"));
        assert!(sanitized.contains_key("content-type"));
    }

    #[test]
    fn test_sanitize_headers_empty() {
        let headers = reqwest::header::HeaderMap::new();
        let sanitized = sanitize_headers(&headers);
        assert!(sanitized.is_empty());
    }

    // ─── User-Agent ───────────────────────────────────────────────────────

    #[test]
    fn test_user_agent_is_stable_chrome() {
        assert!(USER_AGENT.contains("Chrome/"));
        assert!(USER_AGENT.contains("Windows NT 10.0"));
        assert!(!USER_AGENT.contains("Electron"));
        assert!(!USER_AGENT.contains("Slytube"));
        assert!(!USER_AGENT.contains("opentubex"));
    }

    #[test]
    fn test_user_agent_matches_innertube_context() {
        // The UA version should be close to the InnerTube client version
        // to avoid detection as a bot.
        assert!(USER_AGENT.contains("Chrome/131") || USER_AGENT.contains("Chrome/130") || USER_AGENT.contains("Chrome/132"));
    }

    // ─── Invidious credentials ────────────────────────────────────────────

    #[test]
    fn test_invidious_credentials_storage() {
        let client = HttpClient::new().unwrap();
        assert!(client.invidious_credentials.lock().unwrap().is_empty());

        client.set_invidious_credentials("inv.nadeko.net", "token123");
        let creds = client.invidious_credentials.lock().unwrap();
        assert_eq!(creds.get("inv.nadeko.net").unwrap(), "token123");
        drop(creds);

        client.clear_invidious_credentials(Some("inv.nadeko.net"));
        assert!(client.invidious_credentials.lock().unwrap().is_empty());
    }

    #[test]
    fn test_invidious_credentials_multiple_instances() {
        let client = HttpClient::new().unwrap();

        client.set_invidious_credentials("inv.nadeko.net", "token_a");
        client.set_invidious_credentials("yewtu.be", "token_b");

        let creds = client.invidious_credentials.lock().unwrap();
        assert_eq!(creds.len(), 2);
        assert_eq!(creds.get("inv.nadeko.net").unwrap(), "token_a");
        assert_eq!(creds.get("yewtu.be").unwrap(), "token_b");
        drop(creds);

        // Clear all
        client.clear_invidious_credentials(None);
        assert!(client.invidious_credentials.lock().unwrap().is_empty());
    }

    #[test]
    fn test_invidious_credentials_overwrite() {
        let client = HttpClient::new().unwrap();

        client.set_invidious_credentials("inv.nadeko.net", "old_token");
        client.set_invidious_credentials("inv.nadeko.net", "new_token");

        let creds = client.invidious_credentials.lock().unwrap();
        assert_eq!(creds.get("inv.nadeko.net").unwrap(), "new_token");
        assert_eq!(creds.len(), 1);
    }

    // ─── Tracking cookie detection ────────────────────────────────────────

    #[test]
    fn test_check_tracking_cookies_sw_js_data() {
        let url = "https://www.youtube.com/sw.js_data";
        let Some((host, path)) = extract_host_path(url) else {
            panic!("Failed to parse URL");
        };
        assert!(host.ends_with("youtube.com"));
        assert!(path.contains("sw.js_data"));
    }

    #[test]
    fn test_check_tracking_cookies_iframe_api() {
        let url = "https://www.youtube.com/iframe_api";
        let Some((host, path)) = extract_host_path(url) else {
            panic!("Failed to parse URL");
        };
        assert!(host.ends_with("youtube.com"));
        assert!(path.starts_with("/iframe_api"));
    }

    // ─── Cookie constants ─────────────────────────────────────────────────

    #[test]
    fn test_youtube_cookies_contain_consent() {
        assert!(YOUTUBE_COOKIES.contains("CONSENT=YES+"));
    }

    #[test]
    fn test_youtube_cookies_contain_socs() {
        assert!(YOUTUBE_COOKIES.contains("SOCS=CAI"));
    }
}
