use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UrlLinkPayload {
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub title: String,
    pub url: String,
}
