use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, PartialEq, FromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum PayloadType {
    Unknown,
    Message,
    Contact,
    Room,
    RoomMember,
    Friendship,
}
