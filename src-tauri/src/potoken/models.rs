use serde::{Deserialize, Serialize};

/// PoToken generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoTokenRequest {
    pub video_id: String,
    pub context: String,
}

/// PoToken generation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoTokenResult {
    pub token: String,
    pub generated_at: String,
}
