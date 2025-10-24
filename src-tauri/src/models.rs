use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidePreview {
    pub index: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlideIndexItem {
    pub id: String,
    pub path: String,
    pub name: String,
    pub kind: SlideKind,
    pub slide_count: Option<u32>,
    pub snippet: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub updated_at: u64,
    #[serde(default)]
    pub slides: Vec<SlidePreview>,
    #[serde(default)]
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    #[serde(default)]
    pub directories: Vec<String>,
    #[serde(default)]
    pub items: Vec<SlideIndexItem>,
    pub last_indexed_at: Option<u64>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgressPayload {
    pub path: Option<String>,
    pub status: Option<String>, // "cached" or "scanning"
    pub debug_info: Option<String>, // debug messages for UI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub indexed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scanned: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<usize>,
    pub errors: Vec<String>,
    pub last_indexed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub items: Vec<SlideIndexItem>,
    pub total: usize,
    pub last_indexed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlideKind {
    Pptx,
    Pdf,
    Ppt,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            directories: Vec::new(),
            items: Vec::new(),
            last_indexed_at: None,
            warnings: Vec::new(),
        }
    }
}
