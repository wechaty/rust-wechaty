use regex::Regex;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, PartialEq, FromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum MessageType {
    Unknown,
    Attachment,
    Audio,
    Contact,
    ChatHistory,
    Emoticon,
    Image,
    Text,
    Location,
    MiniProgram,
    GroupNote,
    Transfer,
    RedEnvelope,
    Recalled,
    Url,
    Video,
}

#[derive(Debug, Clone, PartialEq, FromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum WechatAppMessageType {
    Text = 1,
    Img = 2,
    Audio = 3,
    Video = 4,
    Url = 5,
    Attach = 6,
    Open = 7,
    Emoji = 8,
    VoiceRemind = 9,
    ScanGood = 10,
    Good = 13,
    Emotion = 15,
    CardTicket = 16,
    RealtimeShareLocation = 17,
    ChatHistory = 19,
    MiniProgram = 33,
    Transfers = 2000,
    RedEnvelopes = 2001,
    ReaderType = 100001,
}

#[derive(Debug, Clone, PartialEq, FromPrimitive, Deserialize_repr, Serialize_repr)]
#[repr(i32)]
pub enum WechatMessageType {
    Text = 1,
    Image = 3,
    Voice = 34,
    VerifyMsg = 37,
    PossibleFriendMsg = 40,
    ShareCard = 42,
    Video = 43,
    Emoticon = 47,
    Location = 48,
    App = 49,
    VoipMsg = 50,
    StatusNotify = 51,
    VoipNotify = 52,
    VoipInvite = 53,
    MicroVideo = 62,
    Transfer = 2000,
    RedEnvelope = 2001,
    MiniProgram = 2002,
    GroupInvite = 2003,
    File = 2004,
    SysNotice = 9999,
    Sys = 10000,
    Recalled = 10002,
}

#[derive(Debug, Clone)]
pub struct MessagePayload {
    pub id: String,
    pub filename: String,
    pub text: String,
    pub timestamp: u64,
    pub message_type: MessageType,
    pub from_id: String,
    pub mention_id_list: Vec<String>,
    pub room_id: String,
    pub to_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct MessageQueryFilter {
    pub from_id: Option<String>,
    pub id: Option<String>,
    pub room_id: Option<String>,
    pub text: Option<String>,
    pub text_regex: Option<Regex>,
    pub to_id: Option<String>,
    pub message_type: Option<MessageType>,
}

// FIXME: trait aliases are experimental, see issue #41517 <https://github.com/rust-lang/rust/issues/41517>
// pub trait MessagePayloadFilterFunction = Fn(MessagePayload) -> bool;
//
// pub trait MessagePayloadFilterFactory = Fn(MessageQueryFilter) ->
// MessagePayloadFilterFunction;
