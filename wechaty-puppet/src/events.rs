use actix::Message;

use crate::schemas::event::*;
// use crate::types::AsyncFnPtr;

// pub type PuppetDirtyListener = AsyncFnPtr<EventDirtyPayload, ()>;
// pub type PuppetDongListener = AsyncFnPtr<EventDongPayload, ()>;
// pub type PuppetErrorListener = AsyncFnPtr<EventErrorPayload, ()>;
// pub type PuppetFriendshipListener = AsyncFnPtr<EventFriendshipPayload, ()>;
// pub type PuppetHeartbeatListener = AsyncFnPtr<EventHeartbeatPayload, ()>;
// pub type PuppetLoginListener = AsyncFnPtr<EventLoginPayload, ()>;
// pub type PuppetLogoutListener = AsyncFnPtr<EventLogoutPayload, ()>;
// pub type PuppetMessageListener = AsyncFnPtr<EventMessagePayload, ()>;
// pub type PuppetReadyListener = AsyncFnPtr<EventReadyPayload, ()>;
// pub type PuppetResetListener = AsyncFnPtr<EventResetPayload, ()>;
// pub type PuppetRoomInviteListener = AsyncFnPtr<EventRoomInvitePayload, ()>;
// pub type PuppetRoomJoinListener = AsyncFnPtr<EventRoomJoinPayload, ()>;
// pub type PuppetRoomLeaveListener = AsyncFnPtr<EventRoomLeavePayload, ()>;
// pub type PuppetRoomTopicListener = AsyncFnPtr<EventRoomTopicPayload, ()>;
// pub type PuppetScanListener = AsyncFnPtr<EventScanPayload, ()>;

#[derive(Debug, Clone, Message)]
#[rtype("()")]
pub enum PuppetEvent {
    Dirty(EventDirtyPayload),
    Dong(EventDongPayload),
    Error(EventErrorPayload),
    Friendship(EventFriendshipPayload),
    Heartbeat(EventHeartbeatPayload),
    Login(EventLoginPayload),
    Logout(EventLogoutPayload),
    Message(EventMessagePayload),
    Ready(EventReadyPayload),
    Reset(EventResetPayload),
    RoomInvite(EventRoomInvitePayload),
    RoomJoin(EventRoomJoinPayload),
    RoomLeave(EventRoomLeavePayload),
    RoomTopic(EventRoomTopicPayload),
    Scan(EventScanPayload),
}
