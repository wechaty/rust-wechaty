use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::schemas::payload::PayloadType;

#[derive(Debug, Clone, PartialEq, FromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum ScanStatus {
    Unknown,
    Cancel,
    Waiting,
    Scanned,
    Confirmed,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct EventFriendshipPayload {
    pub friendship_id: String,
}

#[derive(Debug, Clone)]
pub struct EventLoginPayload {
    pub contact_id: String,
}

#[derive(Debug, Clone)]
pub struct EventLogoutPayload {
    pub contact_id: String,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct EventMessagePayload {
    pub message_id: String,
}

#[derive(Debug, Clone)]
pub struct EventRoomInvitePayload {
    pub room_invitation_id: String,
}

#[derive(Debug, Clone)]
pub struct EventRoomJoinPayload {
    pub invitee_id_list: Vec<String>,
    pub inviter_id: String,
    pub room_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EventRoomLeavePayload {
    pub removee_id_list: Vec<String>,
    pub remover_id: String,
    pub room_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EventRoomTopicPayload {
    pub changer_id: String,
    pub new_topic: String,
    pub old_topic: String,
    pub room_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EventScanPayload {
    pub status: ScanStatus,
    pub qrcode: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EventDongPayload {
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct EventErrorPayload {
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct EventReadyPayload {
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct EventResetPayload {
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct EventHeartbeatPayload {
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct EventDirtyPayload {
    pub payload_type: PayloadType,
    pub payload_id: String,
}
