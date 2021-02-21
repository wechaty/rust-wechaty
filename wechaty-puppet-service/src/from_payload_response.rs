use std::time::{SystemTime, UNIX_EPOCH};

use num_traits::FromPrimitive;
use wechaty_grpc::puppet::{
    ContactPayloadResponse, FriendshipPayloadResponse, MessagePayloadResponse, RoomInvitationPayloadResponse,
    RoomMemberPayloadResponse, RoomPayloadResponse,
};
use wechaty_puppet::schemas::contact::ContactPayload;
use wechaty_puppet::schemas::friendship::FriendshipPayload;
use wechaty_puppet::schemas::message::MessagePayload;
use wechaty_puppet::schemas::room::{RoomMemberPayload, RoomPayload};
use wechaty_puppet::schemas::room_invitation::RoomInvitationPayload;

pub trait FromPayloadResponse<T> {
    fn from_payload_response(payload_response: T) -> Self;
}

impl FromPayloadResponse<ContactPayloadResponse> for ContactPayload {
    fn from_payload_response(response: ContactPayloadResponse) -> Self {
        Self {
            id: response.id,
            gender: FromPrimitive::from_i32(response.gender).unwrap(),
            contact_type: FromPrimitive::from_i32(response.r#type).unwrap(),
            name: response.name,
            avatar: response.avatar,
            address: response.address,
            alias: response.alias,
            city: response.city,
            friend: response.friend,
            province: response.province,
            signature: response.signature,
            star: response.star,
            weixin: response.weixin,
            corporation: response.corporation,
            title: response.title,
            description: response.description,
            coworker: response.coworker,
            phone: response.phone,
        }
    }
}

impl FromPayloadResponse<FriendshipPayloadResponse> for FriendshipPayload {
    fn from_payload_response(response: FriendshipPayloadResponse) -> Self {
        Self {
            id: response.id,
            contact_id: response.contact_id,
            hello: response.hello,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            scene: FromPrimitive::from_i32(response.scene).unwrap(),
            stranger: response.stranger,
            ticket: response.ticket,
            friendship_type: FromPrimitive::from_i32(response.r#type).unwrap(),
        }
    }
}

impl FromPayloadResponse<MessagePayloadResponse> for MessagePayload {
    fn from_payload_response(response: MessagePayloadResponse) -> Self {
        Self {
            id: response.id,
            from_id: response.from_id,
            to_id: response.to_id,
            room_id: response.room_id,
            filename: response.filename,
            text: response.text,
            timestamp: response.timestamp,
            message_type: FromPrimitive::from_i32(response.r#type).unwrap(),
            mention_id_list: response.mention_ids,
        }
    }
}

impl FromPayloadResponse<RoomPayloadResponse> for RoomPayload {
    fn from_payload_response(response: RoomPayloadResponse) -> Self {
        Self {
            id: response.id,
            topic: response.topic,
            avatar: response.avatar,
            member_id_list: response.member_ids,
            owner_id: response.owner_id,
            admin_id_list: response.admin_ids,
        }
    }
}

impl FromPayloadResponse<RoomMemberPayloadResponse> for RoomMemberPayload {
    fn from_payload_response(response: RoomMemberPayloadResponse) -> Self {
        Self {
            id: response.id,
            room_alias: response.room_alias,
            avatar: response.avatar,
            inviter_id: response.inviter_id,
            name: response.name,
        }
    }
}

impl FromPayloadResponse<RoomInvitationPayloadResponse> for RoomInvitationPayload {
    fn from_payload_response(response: RoomInvitationPayloadResponse) -> Self {
        Self {
            id: response.id,
            inviter_id: response.inviter_id,
            topic: response.topic,
            avatar: response.avatar,
            invitation: response.invitation,
            member_count: response.member_count,
            member_id_list: response.member_ids,
            timestamp: response.timestamp,
            receiver_id: response.receiver_id,
        }
    }
}
