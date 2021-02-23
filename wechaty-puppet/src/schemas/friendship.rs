use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, PartialEq, FromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum FriendshipType {
    Unknown,
    Confirm,
    Receive,
    Verify,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, FromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum FriendshipSceneType {
    Unknown = 0,
    QQ = 1,
    Email = 2,
    Weixin = 3,
    QQtbd = 12,
    Room = 14,
    Phone = 15,
    Card = 17,
    Location = 18,
    Bottle = 25,
    Shaking = 29,
    QRCode = 30,
}

#[derive(Debug, Clone)]
pub struct FriendshipPayload {
    pub id: String,
    pub contact_id: String,
    pub hello: String,
    pub timestamp: u64,
    pub scene: FriendshipSceneType,
    pub stranger: String,
    pub ticket: String,
    pub friendship_type: FriendshipType,
}

#[derive(Debug, Clone)]
pub struct FriendshipSearchQueryFilter {
    pub phone: Option<String>,
    pub weixin: Option<String>,
}
