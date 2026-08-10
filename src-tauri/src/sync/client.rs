use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::sync::models::SyncSnapshot;

/// Errors that can occur during sync client operations.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Server returned error: {0}")]
    ServerError(String),
    #[error("Authentication failed: {0}")]
    AuthError(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Response wrapper for sync server API responses.
#[derive(Debug, Serialize, Deserialize)]
struct SyncResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

/// Response from the test connection endpoint.
#[derive(Debug, Serialize, Deserialize)]
struct TestConnectionResponse {
    status: String,
    version: String,
}

/// Uploads an encrypted snapshot to the sync server.
///
/// # Arguments
/// * `server_url` - Base URL of the sync server
/// * `token` - Authentication token
/// * `snapshot` - The sync snapshot to upload
pub async fn upload_snapshot(
    server_url: &str,
    token: &str,
    snapshot: &SyncSnapshot,
) -> Result<(), ClientError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let url = format!("{}/api/sync/upload", server_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(snapshot)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else if response.status().as_u16() == 401 {
        Err(ClientError::AuthError(
            "Invalid or expired token".to_string(),
        ))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(ClientError::ServerError(format!(
            "HTTP {}: {}",
            status, body
        )))
    }
}

/// Downloads the latest snapshot from the sync server.
///
/// # Arguments
/// * `server_url` - Base URL of the sync server
/// * `token` - Authentication token
pub async fn download_snapshot(
    server_url: &str,
    token: &str,
) -> Result<SyncSnapshot, ClientError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let url = format!("{}/api/sync/download", server_url.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status().is_success() {
        let snapshot = response
            .json::<SyncSnapshot>()
            .await
            .map_err(|e| ClientError::InvalidResponse(e.to_string()))?;
        Ok(snapshot)
    } else if response.status().as_u16() == 401 {
        Err(ClientError::AuthError(
            "Invalid or expired token".to_string(),
        ))
    } else if response.status().as_u16() == 404 {
        // No snapshot exists yet on the server
        Err(ClientError::InvalidResponse(
            "No snapshot found on server".to_string(),
        ))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(ClientError::ServerError(format!(
            "HTTP {}: {}",
            status, body
        )))
    }
}

/// Tests connectivity to the sync server.
///
/// Returns `true` if the server is reachable and the token is valid.
pub async fn test_connection(server_url: &str, token: &str) -> Result<bool, ClientError> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let url = format!(
        "{}/api/sync/test",
        server_url.trim_end_matches('/')
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status().is_success() {
        let test_response = response
            .json::<TestConnectionResponse>()
            .await
            .map_err(|e| ClientError::InvalidResponse(e.to_string()))?;

        Ok(test_response.status == "ok")
    } else if response.status().as_u16() == 401 {
        Err(ClientError::AuthError(
            "Invalid or expired token".to_string(),
        ))
    } else {
        Ok(false)
    }
}
