use std::sync::Arc;
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

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client })
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
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .proxy(req_proxy)
            .build()
            .map_err(|e| format!("Failed to create HTTP client with proxy: {}", e))?;

        Ok(Self { client })
    }

    pub async fn get_json(&self, url: &str) -> Result<serde_json::Value, String> {
        let response = self.client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("GET {} failed: {}", url, e))?;

        if !response.status().is_success() {
            return Err(format!("GET {} returned status {}", url, response.status()));
        }

        response.json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse JSON from {}: {}", url, e))
    }

    pub async fn post_json(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let response = self.client
            .post(url)
            .header("Referer", "https://www.youtube.com/")
            .header("Origin", "https://www.youtube.com")
            .header("Content-Type", "application/json")
            .header("X-Youtube-Bootstrap-Logged-In", "false")
            .header("X-Youtube-Client-Name", "1")
            .header("X-Youtube-Client-Version", "2.20240101.01.00")
            .json(body)
            .send()
            .await
            .map_err(|e| format!("POST {} failed: {}", url, e))?;

        if !response.status().is_success() {
            return Err(format!("POST {} returned status {}", url, response.status()));
        }

        response.json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Failed to parse JSON from {}: {}", url, e))
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

pub type SharedHttpClient = Arc<HttpClient>;
