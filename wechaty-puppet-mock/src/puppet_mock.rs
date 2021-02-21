use async_trait::async_trait;
use wechaty_puppet::*;

#[derive(Debug)]
pub struct PuppetMock {}

#[allow(dead_code)]
#[async_trait]
impl PuppetImpl for PuppetMock {
    async fn contact_self_name_set(&self, name: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn contact_self_qr_code(&self) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn contact_self_signature_set(&self, signature: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn tag_contact_add(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn tag_contact_remove(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn tag_contact_delete(&self, tag_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn tag_contact_list(&self, contact_id: String) -> Result<Vec<String>, PuppetError> {
        unimplemented!()
    }

    async fn tag_list(&self) -> Result<Vec<String>, PuppetError> {
        unimplemented!()
    }

    async fn contact_alias(&self, contact_id: String) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn contact_alias_set(&self, contact_id: String, alias: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn contact_avatar(&self, contact_id: String) -> Result<FileBox, PuppetError> {
        unimplemented!()
    }

    async fn contact_avatar_set(&self, contact_id: String, file: FileBox) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn contact_phone_set(&self, contact_id: String, phone_list: Vec<String>) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn contact_corporation_remark_set(
        &self,
        contact_id: String,
        corporation_remark: Option<String>,
    ) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn contact_description_set(
        &self,
        contact_id: String,
        description: Option<String>,
    ) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn contact_list(&self) -> Result<Vec<String>, PuppetError> {
        unimplemented!()
    }

    async fn contact_raw_payload(&self, contact_id: String) -> Result<ContactPayload, PuppetError> {
        unimplemented!()
    }

    async fn message_contact(&self, message_id: String) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn message_file(&self, message_id: String) -> Result<FileBox, PuppetError> {
        unimplemented!()
    }

    async fn message_image(&self, message_id: String, image_type: ImageType) -> Result<FileBox, PuppetError> {
        unimplemented!()
    }

    async fn message_mini_program(&self, message_id: String) -> Result<MiniProgramPayload, PuppetError> {
        unimplemented!()
    }

    async fn message_url(&self, message_id: String) -> Result<UrlLinkPayload, PuppetError> {
        unimplemented!()
    }

    async fn message_send_contact(
        &self,
        conversation_id: String,
        contact_id: String,
    ) -> Result<Option<String>, PuppetError> {
        unimplemented!()
    }

    async fn message_send_file(&self, conversation_id: String, file: FileBox) -> Result<Option<String>, PuppetError> {
        unimplemented!()
    }

    async fn message_send_mini_program(
        &self,
        conversation_id: String,
        mini_program_payload: MiniProgramPayload,
    ) -> Result<Option<String>, PuppetError> {
        unimplemented!()
    }

    async fn message_send_text(
        &self,
        conversation_id: String,
        text: String,
        mention_id_list: Vec<String>,
    ) -> Result<Option<String>, PuppetError> {
        unimplemented!()
    }

    async fn message_send_url(
        &self,
        conversation_id: String,
        url_link_payload: UrlLinkPayload,
    ) -> Result<Option<String>, PuppetError> {
        unimplemented!()
    }

    async fn message_raw_payload(&self, message_id: String) -> Result<MessagePayload, PuppetError> {
        unimplemented!()
    }

    async fn friendship_accept(&self, friendship_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn friendship_add(&self, contact_id: String, hello: Option<String>) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn friendship_search_phone(&self, phone: String) -> Result<Option<String>, PuppetError> {
        unimplemented!()
    }

    async fn friendship_search_weixin(&self, weixin: String) -> Result<Option<String>, PuppetError> {
        unimplemented!()
    }

    async fn friendship_raw_payload(&self, friendship_id: String) -> Result<FriendshipPayload, PuppetError> {
        unimplemented!()
    }

    async fn room_invitation_accept(&self, room_invitation_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn room_invitation_raw_payload(
        &self,
        room_invitation_id: String,
    ) -> Result<RoomInvitationPayload, PuppetError> {
        unimplemented!()
    }

    async fn room_add(&self, room_id: String, contact_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn room_avatar(&self, room_id: String) -> Result<FileBox, PuppetError> {
        unimplemented!()
    }

    async fn room_create(&self, contact_id_list: Vec<String>, topic: Option<String>) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn room_del(&self, room_id: String, contact_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn room_qr_code(&self, room_id: String) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn room_quit(&self, room_id: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn room_topic(&self, room_id: String) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn room_topic_set(&self, room_id: String, topic: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn room_list(&self) -> Result<Vec<String>, PuppetError> {
        unimplemented!()
    }

    async fn room_raw_payload(&self, room_id: String) -> Result<RoomPayload, PuppetError> {
        unimplemented!()
    }

    async fn room_announce(&self, room_id: String) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn room_announce_set(&self, room_id: String, text: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn room_member_list(&self, room_id: String) -> Result<Vec<String>, PuppetError> {
        unimplemented!()
    }

    async fn room_member_raw_payload(
        &self,
        room_id: String,
        contact_id: String,
    ) -> Result<RoomMemberPayload, PuppetError> {
        unimplemented!()
    }

    async fn start(&self) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn stop(&self) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn ding(&self, data: String) -> Result<(), PuppetError> {
        unimplemented!()
    }

    async fn version(&self) -> Result<String, PuppetError> {
        unimplemented!()
    }

    async fn logout(&self) -> Result<(), PuppetError> {
        unimplemented!()
    }
}
