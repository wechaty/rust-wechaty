use regex::Regex;

#[derive(Debug, Clone)]
pub struct RoomMemberQueryFilter {
    pub name: Option<String>,
    pub room_alias: Option<String>,
    pub name_regex: Option<Regex>,
    pub room_alias_regex: Option<Regex>,
}

#[derive(Debug, Clone)]
pub struct RoomQueryFilter {
    pub id: Option<String>,
    pub topic: Option<String>,
    pub topic_regex: Option<Regex>,
}

#[derive(Debug, Clone)]
pub struct RoomPayload {
    pub id: String,
    pub topic: String,
    pub avatar: String,
    pub member_id_list: Vec<String>,
    pub owner_id: String,
    pub admin_id_list: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RoomMemberPayload {
    pub id: String,
    pub room_alias: String,
    pub inviter_id: String,
    pub avatar: String,
    pub name: String,
}

// FIXME: trait aliases are experimental, see issue #41517 <https://github.com/rust-lang/rust/issues/41517>
// pub trait RoomPayloadFilterFunction = Fn(RoomPayload) -> bool;
//
// pub trait RoomPayloadFilterFactory = Fn(RoomQueryFilter) ->
// RoomPayloadFilterFunction;
