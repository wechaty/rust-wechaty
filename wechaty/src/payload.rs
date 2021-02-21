use wechaty_puppet::{
    EventDongPayload, EventErrorPayload, EventHeartbeatPayload, EventReadyPayload, EventResetPayload, EventScanPayload,
    PuppetImpl,
};

use crate::user::contact_self::ContactSelf;
use crate::{Contact, Friendship, Message, Room, RoomInvitation};

pub type DongPayload = EventDongPayload;

pub type ErrorPayload = EventErrorPayload;

#[derive(Clone, Debug)]
pub struct FriendshipPayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub friendship: Friendship<T>,
}

pub type HeartbeatPayload = EventHeartbeatPayload;

#[derive(Clone, Debug)]
pub struct LoginPayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub contact: ContactSelf<T>,
}

#[derive(Clone, Debug)]
pub struct LogoutPayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub contact: ContactSelf<T>,
    pub data: String,
}

#[derive(Clone, Debug)]
pub struct MessagePayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub message: Message<T>,
}

pub type ScanPayload = EventScanPayload;

pub type ReadyPayload = EventReadyPayload;

pub type ResetPayload = EventResetPayload;

#[derive(Clone, Debug)]
pub struct RoomInvitePayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub room_invitation: RoomInvitation<T>,
}

#[derive(Clone, Debug)]
pub struct RoomJoinPayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub room: Room<T>,
    pub invitee_list: Vec<Contact<T>>,
    pub inviter: Contact<T>,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct RoomLeavePayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub room: Room<T>,
    pub removee_list: Vec<Contact<T>>,
    pub remover: Contact<T>,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct RoomTopicPayload<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub room: Room<T>,
    pub old_topic: String,
    pub new_topic: String,
    pub changer: Contact<T>,
    pub timestamp: u64,
}
