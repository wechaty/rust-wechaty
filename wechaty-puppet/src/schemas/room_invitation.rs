#[derive(Debug, Clone)]
pub struct RoomInvitationPayload {
    pub id: String,
    pub inviter_id: String,
    pub topic: String,
    pub avatar: String,
    pub invitation: String,
    pub member_count: u32,
    pub member_id_list: Vec<String>,
    pub timestamp: u64,
    pub receiver_id: String,
}
