use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniProgramPayload {
    appid: Option<String>,
    description: Option<String>,
    page_path: Option<String>,
    icon_url: Option<String>,
    share_id: Option<String>,
    thumb_url: Option<String>,
    title: Option<String>,
    username: Option<String>,
    thumb_key: Option<String>,
}
