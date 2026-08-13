use serde::{Deserialize, Serialize};

/// An extraction request dispatched to the hidden youtubei.js webview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// Result returned from the extractor webview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionResult {
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Supported extraction methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionMethod {
    GetVideoInfo,
    Search,
    GetChannel,
    GetChannelVideos,
    GetChannelShorts,
    GetChannelLive,
    GetChannelCommunity,
    GetComments,
    GetCommentReplies,
    GetTrending,
    GetPlaylist,
    GetHashtag,
    GetCommunityPost,
    GetSearchSuggestions,
    GeneratePoToken,
}

impl ExtractionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GetVideoInfo => "getVideoInfo",
            Self::Search => "search",
            Self::GetChannel => "getChannel",
            Self::GetChannelVideos => "getChannelVideos",
            Self::GetChannelShorts => "getChannelShorts",
            Self::GetChannelLive => "getChannelLive",
            Self::GetChannelCommunity => "getChannelCommunity",
            Self::GetComments => "getComments",
            Self::GetCommentReplies => "getCommentReplies",
            Self::GetTrending => "getTrending",
            Self::GetPlaylist => "getPlaylist",
            Self::GetHashtag => "getHashtag",
            Self::GetCommunityPost => "getCommunityPost",
            Self::GetSearchSuggestions => "getSearchSuggestions",
            Self::GeneratePoToken => "generatePoToken",
        }
    }
}
