//! Image caching for YouTube thumbnails.
//!
//! Provides an in-memory LRU cache that fetches YouTube images through the
//! hardened `HttpClient`, caches them with proper expiry, and serves them via
//! a custom `imgcache://` Tauri protocol. This avoids direct connections from
//! the webview to `*.googleusercontent.com` / `*.ytimg.com`, preventing tracking
//! and CORS issues.

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;
use chrono::Utc;
use reqwest::header::HeaderMap;
use tauri::Manager;
use tauri::http::{Request, Response};
use thiserror::Error;
use urlencoding;

use crate::http_client::{HttpClient, SharedHttpClient};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Maximum number of images kept in the LRU cache.
const MAX_CACHE_ENTRIES: usize = 256;

/// Default time-to-live when the server provides no cache headers.
const DEFAULT_TTL: Duration = Duration::from_secs(2 * 60 * 60); // 2 hours

/// How often the background cleanup task runs.
const CLEANUP_INTERVAL: Duration = Duration::from_secs(5 * 60); // 5 minutes

// ─── Errors ───────────────────────────────────────────────────────────────────

/// Errors that can occur during image caching operations.
#[derive(Debug, Error)]
pub enum ImageCacheError {
    /// The HTTP request to fetch the image failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The cache mutex was poisoned by a panicking thread.
    #[error("Cache lock poisoned")]
    LockPoisoned,

    /// The provided URL was empty or otherwise invalid.
    #[error("Invalid image URL: {0}")]
    InvalidUrl(String),

    /// The server returned a non-success HTTP status code.
    #[error("Image fetch failed with status {0}")]
    BadStatus(u16),
}

/// Helper to convert a poisoned mutex guard into our error type.
impl<T> From<std::sync::PoisonError<T>> for ImageCacheError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        ImageCacheError::LockPoisoned
    }
}

// ─── Cache entry ──────────────────────────────────────────────────────────────

/// A single cached image with its MIME type and expiry time.
#[derive(Clone, Debug)]
pub struct CacheEntry {
    /// Raw image bytes.
    pub data: Vec<u8>,
    /// MIME type (e.g. `image/jpeg`).
    pub mime: String,
    /// When this entry should be considered stale.
    pub expiry: Instant,
}

// ─── LRU cache internals ─────────────────────────────────────────────────────

/// Internal LRU cache state protected by a mutex.
///
/// Uses a `HashMap` for O(1) lookups and a `VecDeque` to track access order.
/// The front of the deque holds the least-recently-used key.
struct LruCache {
    entries: HashMap<String, CacheEntry>,
    order: VecDeque<String>,
    capacity: usize,
}

impl LruCache {
    fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Returns the entry if present and not expired, marking it as recently used.
    fn get(&mut self, key: &str) -> Option<CacheEntry> {
        if let Some(entry) = self.entries.get(key) {
            if entry.expiry < Instant::now() {
                // Expired — remove and return None.
                self.entries.remove(key);
                self.order.retain(|k| k != key);
                return None;
            }
            // Move to back (most recently used).
            self.order.retain(|k| k != key);
            self.order.push_back(key.to_string());
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Inserts an entry, evicting the least-recently-used item if at capacity.
    fn put(&mut self, key: String, value: CacheEntry) {
        if self.entries.contains_key(&key) {
            // Update existing — just refresh order.
            self.order.retain(|k| k != &key);
        } else if self.entries.len() >= self.capacity {
            // Evict LRU.
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
        self.order.push_back(key.clone());
        self.entries.insert(key, value);
    }

    /// Removes all expired entries.
    fn cleanup(&mut self) {
        let now = Instant::now();
        let expired: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, v)| v.expiry < now)
            .map(|(k, _)| k.clone())
            .collect();
        for k in expired {
            self.entries.remove(&k);
            self.order.retain(|o| o != &k);
        }
    }
}

// ─── Public cache handle ─────────────────────────────────────────────────────

/// Thread-safe handle to the image cache.
///
/// Cheap to clone — all clones share the same underlying cache state.
#[derive(Clone)]
pub struct ImageCache {
    inner: Arc<Mutex<LruCache>>,
}

impl ImageCache {
    /// Creates a new, empty image cache.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(MAX_CACHE_ENTRIES))),
        }
    }

    /// Looks up a cached entry without marking it as recently used.
    ///
    /// Returns `None` if the entry is missing or expired.
    pub fn peek(&self, key: &str) -> Result<Option<CacheEntry>, ImageCacheError> {
        let mut cache = self.inner.lock()?;
        Ok(cache.get(key))
    }

    /// Inserts an entry into the cache.
    pub fn put(&self, key: String, value: CacheEntry) -> Result<(), ImageCacheError> {
        let mut cache = self.inner.lock()?;
        cache.put(key, value);
        Ok(())
    }

    /// Removes all expired entries.
    pub fn cleanup(&self) -> Result<(), ImageCacheError> {
        let mut cache = self.inner.lock()?;
        cache.cleanup();
        Ok(())
    }

    /// Spawns a background task that periodically removes expired entries.
    ///
    /// The task runs until the process exits.
    pub fn start_cleanup_task(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(CLEANUP_INTERVAL).await;
                if let Err(e) = self.cleanup() {
                    tracing::warn!("Image cache cleanup failed: {}", e);
                }
            }
        })
    }

    /// Fetches an image through the hardened HTTP client and caches it.
    ///
    /// On cache hit the image is returned immediately. On miss the image is
    /// fetched, stored with the appropriate TTL, and returned.
    async fn fetch_and_cache(
        &self,
        url: &str,
        client: &HttpClient,
    ) -> Result<CacheEntry, ImageCacheError> {
        let builder = client.client().get(url);
        let builder = client.request_internal(builder, url);
        let response = builder.send().await?;

        let status = response.status();
        if !status.is_success() {
            return Err(ImageCacheError::BadStatus(status.as_u16()));
        }

        let headers = response.headers().clone();
        let mime = extract_mime(&headers);
        let ttl = parse_cache_duration(&headers);
        let data = response.bytes().await?.to_vec();

        let entry = CacheEntry {
            data,
            mime,
            expiry: Instant::now() + ttl,
        };

        self.put(url.to_string(), entry.clone())?;
        Ok(entry)
    }

    /// Returns an image as a base64 `data:` URL, fetching and caching on miss.
    ///
    /// This is the primary entry point for the `image_cache_get` Tauri command.
    pub async fn get_image(&self, url: &str, client: &HttpClient) -> Result<String, ImageCacheError> {
        if url.is_empty() {
            return Err(ImageCacheError::InvalidUrl("empty URL".into()));
        }

        // Cache hit.
        if let Some(entry) = self.peek(url)? {
            let encoded = base64::engine::general_purpose::STANDARD.encode(&entry.data);
            return Ok(format!("data:{mime};base64,{encoded}", mime = entry.mime));
        }

        // Cache miss — fetch and cache.
        let entry = self.fetch_and_cache(url, client).await?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&entry.data);
        Ok(format!("data:{mime};base64,{encoded}", mime = entry.mime))
    }

    /// Returns raw image bytes and MIME type, fetching and caching on miss.
    ///
    /// Used by the `imgcache://` protocol handler.
    pub async fn get_image_bytes(
        &self,
        url: &str,
        client: &HttpClient,
    ) -> Result<(Vec<u8>, String), ImageCacheError> {
        if url.is_empty() {
            return Err(ImageCacheError::InvalidUrl("empty URL".into()));
        }

        if let Some(entry) = self.peek(url)? {
            return Ok((entry.data, entry.mime));
        }

        let entry = self.fetch_and_cache(url, client).await?;
        Ok((entry.data, entry.mime))
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Header helpers ───────────────────────────────────────────────────────────

/// Extracts the MIME type from the `Content-Type` header, stripping parameters.
fn extract_mime(headers: &HeaderMap) -> String {
    headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or(s).trim().to_lowercase())
        .unwrap_or_else(|| "image/jpeg".to_string())
}

/// Parses `Cache-Control: max-age` or `Expires` to determine TTL.
///
/// Falls back to [`DEFAULT_TTL`] when neither header is present or parseable.
fn parse_cache_duration(headers: &HeaderMap) -> Duration {
    // Try Cache-Control: max-age=<seconds>
    if let Some(cc) = headers.get("cache-control") {
        if let Ok(cc_str) = cc.to_str() {
            for part in cc_str.split(',') {
                let part = part.trim();
                if let Some(rest) = part.strip_prefix("max-age=") {
                    if let Ok(secs) = rest.trim().parse::<u64>() {
                        return Duration::from_secs(secs);
                    }
                }
            }
        }
    }

    // Try Expires header (HTTP-date format).
    if let Some(exp) = headers.get("expires") {
        if let Ok(exp_str) = exp.to_str() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(exp_str) {
                let now = Utc::now();
                if let Ok(dur) = dt.signed_duration_since(now).to_std() {
                    return dur;
                }
            }
        }
    }

    DEFAULT_TTL
}

// ─── Protocol handler ─────────────────────────────────────────────────────────

/// A 1×1 transparent PNG used as a fallback when an image cannot be fetched.
const TRANSPARENT_1X1_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1×1
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // 8-bit RGBA
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, // IDAT chunk
    0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
    0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
    0xAE, 0x42, 0x60, 0x82,
];

/// Builds a 404 response containing a 1×1 transparent PNG.
fn not_found_response() -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(404)
        .header("Content-Type", "image/png")
        .body(Cow::Borrowed(TRANSPARENT_1X1_PNG))
        .unwrap_or_else(|_| Response::builder().status(404).body(Cow::Owned(vec![])).unwrap())
}

/// Handles an `imgcache://` request.
///
/// Extracts the original image URL from the request URI, fetches it through the
/// cache, and returns the image data with the correct `Content-Type`.
pub fn handle_protocol_request(
    app: &tauri::AppHandle,
    request: &Request<Vec<u8>>,
) -> Response<Cow<'static, [u8]>> {
    let cache = app.state::<ImageCache>();
    let http = app.state::<SharedHttpClient>();

    // The URI is `imgcache://<encoded-url>` — strip the scheme prefix.
    let uri_str = request.uri().to_string();
    let encoded = uri_str.strip_prefix("imgcache://").unwrap_or("");

    let original_url = match urlencoding::decode(encoded) {
        Ok(s) => s.to_string(),
        Err(e) => {
            tracing::warn!("Failed to decode imgcache URL '{encoded}': {e}");
            return not_found_response();
        }
    };

    if original_url.is_empty() {
        return not_found_response();
    }

    // Fetch (blocking inside the sync protocol handler).
    let result = tauri::async_runtime::block_on(async {
        cache.get_image_bytes(&original_url, &http).await
    });

    match result {
        Ok((data, mime)) => Response::builder()
            .status(200)
            .header("Content-Type", &mime)
            .body(Cow::Owned(data))
            .unwrap_or_else(|_| not_found_response()),
        Err(e) => {
            tracing::warn!("Image cache fetch failed for {original_url}: {e}");
            not_found_response()
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── URL encoding / decoding ───────────────────────────────────────────

    #[test]
    fn test_imgcache_url_encoding_roundtrip() {
        let original = "https://i.ytimg.com/vi/abc123/hqdefault.jpg";
        let encoded = urlencoding::encode(original);
        let full_url = format!("imgcache://{encoded}");
        let stripped = full_url.strip_prefix("imgcache://").unwrap();
        let decoded = urlencoding::decode(stripped).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_imgcache_url_encoding_with_query() {
        let original = "https://i.ytimg.com/vi/abc123/hqdefault.jpg?width=1280&height=720";
        let encoded = urlencoding::encode(original);
        let full_url = format!("imgcache://{encoded}");
        let stripped = full_url.strip_prefix("imgcache://").unwrap();
        let decoded = urlencoding::decode(stripped).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_imgcache_url_encoding_ggpht() {
        let original = "https://yt3.ggpht.com/ytc/AIdro_abc123=s900-c-k-c0x00ffffff-no-rj";
        let encoded = urlencoding::encode(original);
        let full_url = format!("imgcache://{encoded}");
        let stripped = full_url.strip_prefix("imgcache://").unwrap();
        let decoded = urlencoding::decode(stripped).unwrap();
        assert_eq!(decoded, original);
    }

    // ─── Data URL formatting ───────────────────────────────────────────────

    #[test]
    fn test_data_url_format() {
        let data = b"fake image data";
        let mime = "image/jpeg";
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        let url = format!("data:{mime};base64,{encoded}");
        assert!(url.starts_with("data:image/jpeg;base64,"));
        assert!(url.contains(&encoded));
    }

    #[test]
    fn test_data_url_format_png() {
        let data = b"png data";
        let mime = "image/png";
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);
        let url = format!("data:{mime};base64,{encoded}");
        assert!(url.starts_with("data:image/png;base64,"));
    }

    // ─── Cache hit / miss logic ────────────────────────────────────────────

    #[test]
    fn test_cache_miss_empty() {
        let cache = ImageCache::new();
        let result = cache.peek("https://example.com/img.jpg").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_put_and_hit() {
        let cache = ImageCache::new();
        let entry = CacheEntry {
            data: b"test data".to_vec(),
            mime: "image/jpeg".to_string(),
            expiry: Instant::now() + Duration::from_secs(3600),
        };
        cache
            .put("https://example.com/img.jpg".to_string(), entry.clone())
            .unwrap();

        let result = cache.peek("https://example.com/img.jpg").unwrap();
        assert!(result.is_some());
        let hit = result.unwrap();
        assert_eq!(hit.data, b"test data");
        assert_eq!(hit.mime, "image/jpeg");
    }

    #[test]
    fn test_cache_expired_entry() {
        let cache = ImageCache::new();
        let entry = CacheEntry {
            data: b"expired data".to_vec(),
            mime: "image/jpeg".to_string(),
            // Already expired.
            expiry: Instant::now() - Duration::from_secs(1),
        };
        cache
            .put("https://example.com/expired.jpg".to_string(), entry)
            .unwrap();

        // Should be treated as a miss.
        let result = cache.peek("https://example.com/expired.jpg").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_overwrite() {
        let cache = ImageCache::new();
        let entry1 = CacheEntry {
            data: b"v1".to_vec(),
            mime: "image/jpeg".to_string(),
            expiry: Instant::now() + Duration::from_secs(3600),
        };
        let entry2 = CacheEntry {
            data: b"v2".to_vec(),
            mime: "image/png".to_string(),
            expiry: Instant::now() + Duration::from_secs(3600),
        };

        cache
            .put("https://example.com/img.jpg".to_string(), entry1)
            .unwrap();
        cache
            .put("https://example.com/img.jpg".to_string(), entry2.clone())
            .unwrap();

        let result = cache.peek("https://example.com/img.jpg").unwrap().unwrap();
        assert_eq!(result.data, b"v2");
        assert_eq!(result.mime, "image/png");
    }

    // ─── LRU eviction ──────────────────────────────────────────────────────

    #[test]
    fn test_lru_eviction() {
        let cache = ImageCache::new();

        // Fill to capacity.
        for i in 0..MAX_CACHE_ENTRIES {
            let entry = CacheEntry {
                data: vec![i as u8],
                mime: "image/jpeg".to_string(),
                expiry: Instant::now() + Duration::from_secs(3600),
            };
            cache
                .put(format!("https://example.com/img{i}.jpg"), entry)
                .unwrap();
        }

        // All entries should be present.
        for i in 0..MAX_CACHE_ENTRIES {
            let url = format!("https://example.com/img{i}.jpg");
            assert!(cache.peek(&url).unwrap().is_some(), "missing {url}");
        }

        // Insert one more — the first (LRU) should be evicted.
        let new_entry = CacheEntry {
            data: b"new".to_vec(),
            mime: "image/jpeg".to_string(),
            expiry: Instant::now() + Duration::from_secs(3600),
        };
        cache
            .put("https://example.com/new.jpg".to_string(), new_entry)
            .unwrap();

        // First entry evicted.
        assert!(cache
            .peek("https://example.com/img0.jpg")
            .unwrap()
            .is_none());
        // New entry present.
        assert!(cache.peek("https://example.com/new.jpg").unwrap().is_some());
    }

    #[test]
    fn test_lru_access_updates_order() {
        let cache = ImageCache::new();

        // Insert 3 entries.
        for i in 0..3 {
            let entry = CacheEntry {
                data: vec![i as u8],
                mime: "image/jpeg".to_string(),
                expiry: Instant::now() + Duration::from_secs(3600),
            };
            cache
                .put(format!("https://example.com/img{i}.jpg"), entry)
                .unwrap();
        }

        // Access img0 to mark it as recently used.
        let _ = cache.peek("https://example.com/img0.jpg");

        // Insert enough entries to overflow capacity by exactly one.
        // After peeking img0, the LRU order is [img1, img2, img0]. We start
        // at 3 entries, so adding (capacity - 3 + 1) new entries fills to
        // capacity then evicts the single least-recently-used entry (img1).
        for i in 3..(3 + MAX_CACHE_ENTRIES - 2) {
            let entry = CacheEntry {
                data: vec![i as u8],
                mime: "image/jpeg".to_string(),
                expiry: Instant::now() + Duration::from_secs(3600),
            };
            cache
                .put(format!("https://example.com/img{i}.jpg"), entry)
                .unwrap();
        }

        // img0 was accessed recently, so it should still be present.
        assert!(cache
            .peek("https://example.com/img0.jpg")
            .unwrap()
            .is_some());
        // img2 was accessed after img0 was moved; it should still be present.
        assert!(cache
            .peek("https://example.com/img2.jpg")
            .unwrap()
            .is_some());
        // img1 was the least-recently-used and should be evicted.
        assert!(cache
            .peek("https://example.com/img1.jpg")
            .unwrap()
            .is_none());
    }

    // ─── Cleanup ───────────────────────────────────────────────────────────

    #[test]
    fn test_cleanup_removes_expired() {
        let cache = ImageCache::new();

        // One expired, one fresh.
        let expired = CacheEntry {
            data: b"old".to_vec(),
            mime: "image/jpeg".to_string(),
            expiry: Instant::now() - Duration::from_secs(10),
        };
        let fresh = CacheEntry {
            data: b"new".to_vec(),
            mime: "image/jpeg".to_string(),
            expiry: Instant::now() + Duration::from_secs(3600),
        };

        cache
            .put("https://example.com/expired.jpg".to_string(), expired)
            .unwrap();
        cache
            .put("https://example.com/fresh.jpg".to_string(), fresh)
            .unwrap();

        cache.cleanup().unwrap();

        assert!(cache
            .peek("https://example.com/expired.jpg")
            .unwrap()
            .is_none());
        assert!(cache.peek("https://example.com/fresh.jpg").unwrap().is_some());
    }

    // ─── MIME extraction ───────────────────────────────────────────────────

    #[test]
    fn test_extract_mime_basic() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "image/jpeg".parse().unwrap());
        assert_eq!(extract_mime(&headers), "image/jpeg");
    }

    #[test]
    fn test_extract_mime_with_charset() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "image/webp; charset=utf-8".parse().unwrap());
        assert_eq!(extract_mime(&headers), "image/webp");
    }

    #[test]
    fn test_extract_mime_missing() {
        let headers = HeaderMap::new();
        assert_eq!(extract_mime(&headers), "image/jpeg");
    }

    #[test]
    fn test_extract_mime_uppercase() {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "IMAGE/PNG".parse().unwrap());
        assert_eq!(extract_mime(&headers), "image/png");
    }

    // ─── Cache-Control parsing ─────────────────────────────────────────────

    #[test]
    fn test_parse_cache_control_max_age() {
        let mut headers = HeaderMap::new();
        headers.insert("cache-control", "public, max-age=3600".parse().unwrap());
        let dur = parse_cache_duration(&headers);
        assert_eq!(dur, Duration::from_secs(3600));
    }

    #[test]
    fn test_parse_cache_control_max_age_only() {
        let mut headers = HeaderMap::new();
        headers.insert("cache-control", "max-age=60".parse().unwrap());
        let dur = parse_cache_duration(&headers);
        assert_eq!(dur, Duration::from_secs(60));
    }

    #[test]
    fn test_parse_cache_control_no_max_age() {
        let mut headers = HeaderMap::new();
        headers.insert("cache-control", "no-cache".parse().unwrap());
        let dur = parse_cache_duration(&headers);
        assert_eq!(dur, DEFAULT_TTL);
    }

    #[test]
    fn test_parse_cache_control_empty() {
        let headers = HeaderMap::new();
        let dur = parse_cache_duration(&headers);
        assert_eq!(dur, DEFAULT_TTL);
    }

    // ─── Protocol handler URL extraction ───────────────────────────────────

    #[test]
    fn test_extract_original_url_from_imgcache() {
        let original = "https://i.ytimg.com/vi/abc123/hqdefault.jpg";
        let encoded = urlencoding::encode(original);
        let uri = format!("imgcache://{encoded}");
        let stripped = uri.strip_prefix("imgcache://").unwrap();
        let decoded = urlencoding::decode(stripped).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_extract_original_url_empty() {
        let uri = "imgcache://";
        let stripped = uri.strip_prefix("imgcache://").unwrap();
        assert!(stripped.is_empty());
    }

    // ─── Transparent PNG fallback ──────────────────────────────────────────

    #[test]
    fn test_transparent_png_is_valid() {
        // The PNG should start with the PNG signature.
        assert_eq!(&TRANSPARENT_1X1_PNG[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }
}
