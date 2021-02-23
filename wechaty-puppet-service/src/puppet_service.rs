use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Recipient, StreamHandler};
use async_trait::async_trait;
use log::{debug, error, info};
use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use tonic::{transport::Channel, Status, Streaming};
use wechaty_grpc::puppet::*;
use wechaty_grpc::puppet_client::PuppetClient;
use wechaty_puppet::*;
use wechaty_puppet::{ImageType, PayloadType};

use crate::from_payload_response::FromPayloadResponse;
use crate::service_endpoint::discover;

#[derive(Clone)]
pub struct PuppetService {
    client_: PuppetClient<Channel>,
    addr: Addr<PuppetServiceInner>,
}

impl PuppetService {
    /// Create puppet instance from puppet options.
    ///
    /// First use endpoint, if endpoint is not given, try token instead.
    pub async fn new(options: PuppetOptions) -> Result<Puppet<Self>, PuppetError> {
        let endpoint = if let Some(endpoint) = options.endpoint {
            endpoint
        } else if let Some(token) = options.token {
            match discover(token).await {
                Ok(endpoint) => endpoint,
                Err(e) => return Err(e),
            }
        } else {
            return Err(PuppetError::InvalidToken);
        };

        match PuppetClient::connect(endpoint.clone()).await {
            Ok(mut client) => {
                info!("Connected to endpoint {}", endpoint);
                let response = client.event(EventRequest {}).await;
                match response {
                    Ok(response) => {
                        info!("Subscribed to event stream");
                        let addr = PuppetServiceInner::new().start();
                        let puppet_service = Self {
                            client_: client,
                            addr: addr.clone(),
                        };
                        let puppet = Puppet::new(puppet_service);
                        let callback_addr = puppet.self_addr();
                        addr.do_send(PuppetServiceInternalMessage::SetupCallback(callback_addr));
                        addr.do_send(PuppetServiceInternalMessage::SetupStream(response.into_inner()));
                        Ok(puppet)
                    }
                    Err(e) => Err(PuppetError::Network(format!(
                        "Failed to establish event stream, reason: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(PuppetError::Network(format!(
                "Failed to establish RPC connection, reason: {}",
                e
            ))),
        }
    }

    fn client(&self) -> PuppetClient<Channel> {
        self.client_.clone()
    }
}

#[derive(Message)]
#[rtype("()")]
enum PuppetServiceInternalMessage {
    SetupCallback(Recipient<PuppetEvent>),
    SetupStream(Streaming<EventResponse>),
}

#[derive(Clone, Debug)]
struct PuppetServiceInner {
    callback_addr: Option<Recipient<PuppetEvent>>,
}

impl PuppetServiceInner {
    fn new() -> Self {
        Self { callback_addr: None }
    }

    fn emit(&self, msg: PuppetEvent) {
        if let Err(e) = self.callback_addr.as_ref().unwrap().do_send(msg) {
            error!("Internal error: {}", e)
        }
    }
}

impl Actor for PuppetServiceInner {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Puppet service started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Puppet service stopped");
    }
}

impl Handler<PuppetServiceInternalMessage> for PuppetServiceInner {
    type Result = ();

    fn handle(&mut self, msg: PuppetServiceInternalMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PuppetServiceInternalMessage::SetupCallback(callback_addr) => {
                self.callback_addr = Some(callback_addr);
            }
            PuppetServiceInternalMessage::SetupStream(stream) => {
                ctx.add_stream(stream);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct EventPayload {
    pub data: Option<String>,
    pub contact_id: Option<String>,
    pub message_id: Option<String>,
    pub room_invitation_id: Option<String>,
    pub friendship_id: Option<String>,
    pub qrcode: Option<String>,
    pub status: Option<ScanStatus>,
    pub timestamp: Option<u64>,
    pub changer_id: Option<String>,
    pub new_topic: Option<String>,
    pub old_topic: Option<String>,
    pub room_id: Option<String>,
    pub removee_id_list: Option<Vec<String>>,
    pub remover_id: Option<String>,
    pub invitee_id_list: Option<Vec<String>>,
    pub inviter_id: Option<String>,
    pub payload_type: Option<PayloadType>,
    pub payload_id: Option<String>,
}

impl StreamHandler<Result<EventResponse, Status>> for PuppetServiceInner {
    fn handle(&mut self, item: Result<EventResponse, Status>, _ctx: &mut Self::Context) {
        match item {
            Ok(response) => {
                let payload: EventPayload = from_str(&response.payload).unwrap();
                info!("Receive event response, {:?}", response);

                match response.r#type {
                    0 => {
                        // Unspecified
                    }
                    1 => {
                        // Heartbeat
                        if payload.data == None {
                            error!("Heartbeat payload should have data");
                        } else {
                            self.emit(PuppetEvent::Heartbeat(EventHeartbeatPayload {
                                data: payload.data.unwrap(),
                            }));
                        }
                    }
                    2 => {
                        // Message
                        if payload.message_id == None {
                            error!("Message payload should have message id");
                        } else {
                            self.emit(PuppetEvent::Message(EventMessagePayload {
                                message_id: payload.message_id.unwrap(),
                            }));
                        }
                    }
                    3 => {
                        // Dong
                        if payload.data == None {
                            error!("Dong payload should have data");
                        } else {
                            self.emit(PuppetEvent::Dong(EventDongPayload {
                                data: payload.data.unwrap(),
                            }));
                        }
                    }
                    16 => {
                        // Error
                        if payload.data == None {
                            error!("Error payload should have data");
                        } else {
                            self.emit(PuppetEvent::Error(EventErrorPayload {
                                data: payload.data.unwrap(),
                            }));
                        }
                    }
                    17 => {
                        // Friendship
                        if payload.friendship_id == None {
                            error!("Friendship payload should have friendship id");
                        } else {
                            self.emit(PuppetEvent::Friendship(EventFriendshipPayload {
                                friendship_id: payload.friendship_id.unwrap(),
                            }));
                        }
                    }
                    18 => {
                        // Room invite
                        if payload.room_invitation_id == None {
                            error!("Room invite payload should have room invitation id");
                        } else {
                            self.emit(PuppetEvent::RoomInvite(EventRoomInvitePayload {
                                room_invitation_id: payload.room_invitation_id.unwrap(),
                            }));
                        }
                    }
                    19 => {
                        // Room join
                        if payload.room_id == None
                            || payload.invitee_id_list == None
                            || payload.inviter_id == None
                            || payload.timestamp == None
                        {
                            error!("Room join payload should have room id, inviter id, invitee id list and timestamp");
                        } else {
                            self.emit(PuppetEvent::RoomJoin(EventRoomJoinPayload {
                                room_id: payload.room_id.unwrap(),
                                inviter_id: payload.inviter_id.unwrap(),
                                invitee_id_list: payload.invitee_id_list.unwrap(),
                                timestamp: payload.timestamp.unwrap(),
                            }));
                        }
                    }
                    20 => {
                        // Room leave
                        if payload.room_id == None
                            || payload.removee_id_list == None
                            || payload.remover_id == None
                            || payload.timestamp == None
                        {
                            error!("Room leave payload should have room id, remover id, removee id list and timestamp");
                        } else {
                            self.emit(PuppetEvent::RoomLeave(EventRoomLeavePayload {
                                room_id: payload.room_id.unwrap(),
                                remover_id: payload.remover_id.unwrap(),
                                removee_id_list: payload.removee_id_list.unwrap(),
                                timestamp: payload.timestamp.unwrap(),
                            }));
                        }
                    }
                    21 => {
                        // Room topic
                        if payload.room_id == None
                            || payload.changer_id == None
                            || payload.old_topic == None
                            || payload.new_topic == None
                            || payload.timestamp == None
                        {
                            error!("Room topic payload should have room id, changer id, old topic, new topic and timestamp");
                        } else {
                            self.emit(PuppetEvent::RoomTopic(EventRoomTopicPayload {
                                room_id: payload.room_id.unwrap(),
                                changer_id: payload.changer_id.unwrap(),
                                old_topic: payload.old_topic.unwrap(),
                                new_topic: payload.new_topic.unwrap(),
                                timestamp: payload.timestamp.unwrap(),
                            }));
                        }
                    }
                    22 => {
                        // Scan
                        if payload.status == None {
                            error!("Scan payload should have scan status");
                        } else {
                            self.emit(PuppetEvent::Scan(EventScanPayload {
                                status: payload.status.unwrap(),
                                qrcode: payload.qrcode,
                                data: payload.data,
                            }));
                        }
                    }
                    23 => {
                        // Ready
                        if payload.data == None {
                            error!("Ready payload should have data");
                        } else {
                            self.emit(PuppetEvent::Ready(EventReadyPayload {
                                data: payload.data.unwrap(),
                            }));
                        }
                    }
                    24 => {
                        // Reset
                        if payload.data == None {
                            error!("Reset payload should have data");
                        } else {
                            self.emit(PuppetEvent::Reset(EventResetPayload {
                                data: payload.data.unwrap(),
                            }));
                        }
                    }
                    25 => {
                        // Log in
                        if payload.contact_id == None {
                            error!("Login payload should have contact id");
                        } else {
                            self.emit(PuppetEvent::Login(EventLoginPayload {
                                contact_id: payload.contact_id.unwrap(),
                            }));
                        }
                    }
                    26 => {
                        // Log out
                        if payload.contact_id == None || payload.data == None {
                            error!("Logout payload should have contact id and data");
                        } else {
                            self.emit(PuppetEvent::Logout(EventLogoutPayload {
                                contact_id: payload.contact_id.unwrap(),
                                data: payload.data.unwrap(),
                            }));
                        }
                    }
                    27 => {
                        // Dirty
                        if payload.payload_type == None || payload.payload_id == None {
                            error!("Dirty payload should have payload type and payload id");
                        } else {
                            self.emit(PuppetEvent::Dirty(EventDirtyPayload {
                                payload_type: payload.payload_type.unwrap(),
                                payload_id: payload.payload_id.unwrap(),
                            }));
                        }
                    }
                    _ => {
                        error!("Invalid event type: {}", response.r#type);
                    }
                }
            }
            Err(e) => {
                error!("Network error: {}", e);
            }
        }
    }

    fn finished(&mut self, _ctx: &mut Self::Context) {
        info!("Stream finished");
    }
}

#[async_trait]
impl PuppetImpl for PuppetService {
    async fn contact_self_name_set(&self, name: String) -> Result<(), PuppetError> {
        debug!("contact_self_name_set(name = {})", name);
        match self.client().contact_self_name(ContactSelfNameRequest { name }).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network("Failed to set contact self name".to_owned())),
        }
    }

    async fn contact_self_qr_code(&self) -> Result<String, PuppetError> {
        debug!("contact_self_qr_code()");
        match self.client().contact_self_qr_code(ContactSelfQrCodeRequest {}).await {
            Ok(response) => Ok(response.into_inner().qrcode),
            Err(_) => Err(PuppetError::Network("Failed to get contact self qrcode".to_owned())),
        }
    }

    async fn contact_self_signature_set(&self, signature: String) -> Result<(), PuppetError> {
        debug!("contact_self_signature_set(signature = {})", signature);
        match self
            .client()
            .contact_self_signature(ContactSelfSignatureRequest { signature })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network("Failed to set contact self signature".to_owned())),
        }
    }

    async fn tag_contact_add(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError> {
        debug!("tag_contact_add(tag_id = {}, contact_id = {})", tag_id, contact_id);
        match self
            .client()
            .tag_contact_add(TagContactAddRequest {
                id: tag_id.clone(),
                contact_id: contact_id.clone(),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to add tag {} for contact {}",
                tag_id, contact_id
            ))),
        }
    }

    async fn tag_contact_remove(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError> {
        debug!("tag_contact_remove(tag_id = {}, contact_id = {})", tag_id, contact_id);
        match self
            .client()
            .tag_contact_remove(TagContactRemoveRequest {
                id: tag_id.clone(),
                contact_id: contact_id.clone(),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to remove tag {} for contact {}",
                tag_id, contact_id
            ))),
        }
    }

    async fn tag_contact_delete(&self, tag_id: String) -> Result<(), PuppetError> {
        debug!("tag_contact_delete(tag_id = {})", tag_id);
        match self
            .client()
            .tag_contact_delete(TagContactDeleteRequest { id: tag_id.clone() })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!("Failed to remove tag {}", tag_id))),
        }
    }

    async fn tag_contact_list(&self, contact_id: String) -> Result<Vec<String>, PuppetError> {
        debug!("tag_contact_list(contact_id = {})", contact_id);
        match self
            .client()
            .tag_contact_list(TagContactListRequest {
                contact_id: Some(contact_id.clone()),
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().ids),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get tags for contact {}",
                contact_id
            ))),
        }
    }

    async fn tag_list(&self) -> Result<Vec<String>, PuppetError> {
        debug!("tag_list()");
        match self
            .client()
            .tag_contact_list(TagContactListRequest { contact_id: None })
            .await
        {
            Ok(response) => Ok(response.into_inner().ids),
            Err(_) => Err(PuppetError::Network("Failed to get tags".to_owned())),
        }
    }

    async fn contact_alias(&self, contact_id: String) -> Result<String, PuppetError> {
        debug!("contact_alias(contact_id = {})", contact_id);
        match self
            .client()
            .contact_alias(ContactAliasRequest {
                id: contact_id.clone(),
                alias: None,
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().alias.unwrap()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get alias of contact {}",
                contact_id
            ))),
        }
    }

    async fn contact_alias_set(&self, contact_id: String, alias: String) -> Result<(), PuppetError> {
        debug!("contact_alias_set(contact_id = {}, alias = {})", contact_id, alias);
        match self
            .client()
            .contact_alias(ContactAliasRequest {
                id: contact_id.clone(),
                alias: Some(alias.clone()),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to set alias for contact {}",
                contact_id
            ))),
        }
    }

    async fn contact_avatar(&self, contact_id: String) -> Result<FileBox, PuppetError> {
        debug!("contact_avatar(contact_id = {})", contact_id);
        match self
            .client()
            .contact_avatar(ContactAvatarRequest {
                id: contact_id.clone(),
                filebox: None,
            })
            .await
        {
            Ok(response) => Ok(FileBox::from(response.into_inner().filebox.unwrap())),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get avatar of contact {}",
                contact_id
            ))),
        }
    }

    async fn contact_avatar_set(&self, contact_id: String, file: FileBox) -> Result<(), PuppetError> {
        debug!("contact_avatar_set(contact_id = {}, file = {})", contact_id, file);
        match self
            .client()
            .contact_avatar(ContactAvatarRequest {
                id: contact_id.clone(),
                filebox: Some(file.to_string()),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to set avatar for contact {}",
                contact_id
            ))),
        }
    }

    async fn contact_phone_set(&self, contact_id: String, phone_list: Vec<String>) -> Result<(), PuppetError> {
        debug!(
            "contact_phone_set(contact_id = {}, phone_list = {:?})",
            contact_id, phone_list
        );
        match self
            .client()
            .contact_phone(ContactPhoneRequest {
                contact_id: contact_id.clone(),
                phone_list,
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to set phone for contact {}",
                contact_id
            ))),
        }
    }

    async fn contact_corporation_remark_set(
        &self,
        contact_id: String,
        corporation_remark: Option<String>,
    ) -> Result<(), PuppetError> {
        debug!(
            "contact_corporation_remark_set(contact_id = {}, corporation_remark = {:?})",
            contact_id, corporation_remark
        );
        match self
            .client()
            .contact_corporation_remark(ContactCorporationRemarkRequest {
                contact_id: contact_id.clone(),
                corporation_remark,
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to set corporation remark for contact {}",
                contact_id
            ))),
        }
    }

    async fn contact_description_set(
        &self,
        contact_id: String,
        description: Option<String>,
    ) -> Result<(), PuppetError> {
        debug!(
            "contact_description_set(contact_id = {}, description = {:?})",
            contact_id, description
        );
        match self
            .client()
            .contact_description(ContactDescriptionRequest {
                contact_id: contact_id.clone(),
                description,
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to set description for contact {}",
                contact_id
            ))),
        }
    }

    async fn contact_list(&self) -> Result<Vec<String>, PuppetError> {
        debug!("contact_list()");
        match self.client().contact_list(ContactListRequest {}).await {
            Ok(response) => Ok(response.into_inner().ids),
            Err(_) => Err(PuppetError::Network("Failed to get contacts".to_owned())),
        }
    }

    async fn contact_raw_payload(&self, contact_id: String) -> Result<ContactPayload, PuppetError> {
        debug!("contact_raw_payload(contact_id = {})", contact_id);
        match self
            .client()
            .contact_payload(ContactPayloadRequest { id: contact_id.clone() })
            .await
        {
            Ok(response) => Ok(ContactPayload::from_payload_response(response.into_inner())),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get raw payload for contact {}",
                contact_id
            ))),
        }
    }

    async fn message_contact(&self, message_id: String) -> Result<String, PuppetError> {
        debug!("message_contact(message_id = {})", message_id);
        match self
            .client()
            .message_contact(MessageContactRequest { id: message_id.clone() })
            .await
        {
            Ok(response) => Ok(response.into_inner().id),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get contact of message {}",
                message_id
            ))),
        }
    }

    async fn message_file(&self, message_id: String) -> Result<FileBox, PuppetError> {
        debug!("message_file(message_id = {})", message_id);
        match self
            .client()
            .message_file(MessageFileRequest { id: message_id.clone() })
            .await
        {
            Ok(response) => Ok(FileBox::from(response.into_inner().filebox)),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get file of message {}",
                message_id
            ))),
        }
    }

    async fn message_image(&self, message_id: String, image_type: ImageType) -> Result<FileBox, PuppetError> {
        debug!("message_image(message_id = {})", message_id);
        match self
            .client()
            .message_image(MessageImageRequest {
                id: message_id.clone(),
                r#type: image_type.to_i32().unwrap(),
            })
            .await
        {
            Ok(response) => Ok(FileBox::from(response.into_inner().filebox)),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get image of message {}",
                message_id
            ))),
        }
    }

    async fn message_mini_program(&self, message_id: String) -> Result<MiniProgramPayload, PuppetError> {
        debug!("message_mini_program(message_id = {})", message_id);
        match self
            .client()
            .message_mini_program(MessageMiniProgramRequest { id: message_id.clone() })
            .await
        {
            Ok(response) => Ok(from_str(&response.into_inner().mini_program).unwrap()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get mini_program of message {}",
                message_id
            ))),
        }
    }

    async fn message_url(&self, message_id: String) -> Result<UrlLinkPayload, PuppetError> {
        debug!("message_url(message_id = {})", message_id);
        match self
            .client()
            .message_url(MessageUrlRequest { id: message_id.clone() })
            .await
        {
            Ok(response) => Ok(from_str(&response.into_inner().url_link).unwrap()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get url link of message {}",
                message_id
            ))),
        }
    }

    async fn message_send_contact(
        &self,
        conversation_id: String,
        contact_id: String,
    ) -> Result<Option<String>, PuppetError> {
        debug!(
            "message_send_contact(conversation_id = {}, contact_id = {})",
            conversation_id, contact_id
        );
        match self
            .client()
            .message_send_contact(MessageSendContactRequest {
                conversation_id: conversation_id.clone(),
                contact_id: contact_id.clone(),
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().id),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to send contact {} in conversation {}",
                contact_id, conversation_id
            ))),
        }
    }

    async fn message_send_file(&self, conversation_id: String, file: FileBox) -> Result<Option<String>, PuppetError> {
        debug!(
            "message_send_file(conversation_id = {}, file = {})",
            conversation_id, file
        );
        match self
            .client()
            .message_send_file(MessageSendFileRequest {
                conversation_id: conversation_id.clone(),
                filebox: file.to_string(),
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().id),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to send file in conversation {}",
                conversation_id
            ))),
        }
    }

    async fn message_send_mini_program(
        &self,
        conversation_id: String,
        mini_program_payload: MiniProgramPayload,
    ) -> Result<Option<String>, PuppetError> {
        debug!(
            "message_send_file(conversation_id = {}, mini_program_payload = {:?})",
            conversation_id, mini_program_payload
        );
        match self
            .client()
            .message_send_mini_program(MessageSendMiniProgramRequest {
                conversation_id: conversation_id.clone(),
                mini_program: to_string::<MiniProgramPayload>(&mini_program_payload).unwrap(),
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().id),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to send mini program in conversation {}",
                conversation_id
            ))),
        }
    }

    async fn message_send_text(
        &self,
        conversation_id: String,
        text: String,
        mention_id_list: Vec<String>,
    ) -> Result<Option<String>, PuppetError> {
        debug!(
            "message_send_text(conversation_id = {}, text = {}, mention_id_list = {:?})",
            conversation_id, text, mention_id_list
        );
        match self
            .client()
            .message_send_text(MessageSendTextRequest {
                conversation_id: conversation_id.clone(),
                text,
                mentonal_ids: mention_id_list,
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().id),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to send text in conversation {}",
                conversation_id
            ))),
        }
    }

    async fn message_send_url(
        &self,
        conversation_id: String,
        url_link_payload: UrlLinkPayload,
    ) -> Result<Option<String>, PuppetError> {
        debug!(
            "message_send_url(conversation_id = {}, url_link_payload = {:?})",
            conversation_id, url_link_payload
        );
        match self
            .client()
            .message_send_url(MessageSendUrlRequest {
                conversation_id: conversation_id.clone(),
                url_link: to_string::<UrlLinkPayload>(&url_link_payload).unwrap(),
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().id),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to send url link in conversation {}",
                conversation_id
            ))),
        }
    }

    async fn message_raw_payload(&self, message_id: String) -> Result<MessagePayload, PuppetError> {
        debug!("message_raw_payload(message_id = {})", message_id);
        match self
            .client()
            .message_payload(MessagePayloadRequest { id: message_id.clone() })
            .await
        {
            Ok(response) => Ok(MessagePayload::from_payload_response(response.into_inner())),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get raw payload for message {}",
                message_id
            ))),
        }
    }

    async fn friendship_accept(&self, friendship_id: String) -> Result<(), PuppetError> {
        debug!("friendship_accept(friendship_id = {})", friendship_id);
        match self
            .client()
            .friendship_accept(FriendshipAcceptRequest {
                id: friendship_id.clone(),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to accept friendship {}",
                friendship_id
            ))),
        }
    }

    async fn friendship_add(&self, contact_id: String, hello: Option<String>) -> Result<(), PuppetError> {
        debug!("friendship_add(contact_id = {}, hello = {:?})", contact_id, hello);
        match self
            .client()
            .friendship_add(FriendshipAddRequest {
                contact_id: contact_id.clone(),
                hello: if let Some(hello) = hello { hello } else { String::new() },
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!("Failed to add contact {}", contact_id))),
        }
    }

    async fn friendship_search_phone(&self, phone: String) -> Result<Option<String>, PuppetError> {
        debug!("friendship_search_phone(phone = {})", phone);
        match self
            .client()
            .friendship_search_phone(FriendshipSearchPhoneRequest { phone: phone.clone() })
            .await
        {
            Ok(response) => Ok(response.into_inner().contact_id),
            Err(_) => Err(PuppetError::Network(format!("Failed to search phone {}", phone))),
        }
    }

    async fn friendship_search_weixin(&self, weixin: String) -> Result<Option<String>, PuppetError> {
        debug!("friendship_search_weixin(weixin = {})", weixin);
        match self
            .client()
            .friendship_search_weixin(FriendshipSearchWeixinRequest { weixin: weixin.clone() })
            .await
        {
            Ok(response) => Ok(response.into_inner().contact_id),
            Err(_) => Err(PuppetError::Network(format!("Failed to search weixin {}", weixin))),
        }
    }

    async fn friendship_raw_payload(&self, friendship_id: String) -> Result<FriendshipPayload, PuppetError> {
        debug!("friendship_raw_payload(friendship_id = {})", friendship_id);
        match self
            .client()
            .friendship_payload(FriendshipPayloadRequest {
                id: friendship_id.clone(),
                payload: None,
            })
            .await
        {
            Ok(response) => Ok(FriendshipPayload::from_payload_response(response.into_inner())),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get raw payload for friendship {}",
                friendship_id
            ))),
        }
    }

    async fn room_invitation_accept(&self, room_invitation_id: String) -> Result<(), PuppetError> {
        debug!("room_invitation_accept(room_invitation_id = {})", room_invitation_id);
        match self
            .client()
            .room_invitation_accept(RoomInvitationAcceptRequest {
                id: room_invitation_id.clone(),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to accept room invitation {}",
                room_invitation_id
            ))),
        }
    }

    async fn room_invitation_raw_payload(
        &self,
        room_invitation_id: String,
    ) -> Result<RoomInvitationPayload, PuppetError> {
        debug!(
            "room_invitation_raw_payload(room_invitation_id = {})",
            room_invitation_id
        );
        match self
            .client()
            .room_invitation_payload(RoomInvitationPayloadRequest {
                id: room_invitation_id.clone(),
                payload: None,
            })
            .await
        {
            Ok(response) => Ok(RoomInvitationPayload::from_payload_response(response.into_inner())),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get raw payload for room invitation {}",
                room_invitation_id
            ))),
        }
    }

    async fn room_add(&self, room_id: String, contact_id: String) -> Result<(), PuppetError> {
        debug!("room_add(room_id = {}, contact_id = {})", room_id, contact_id);
        match self
            .client()
            .room_add(RoomAddRequest {
                id: room_id.clone(),
                contact_id: contact_id.clone(),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to add contact {} into room {}",
                contact_id, room_id
            ))),
        }
    }

    async fn room_avatar(&self, room_id: String) -> Result<FileBox, PuppetError> {
        debug!("room_avatar(room_id = {})", room_id);
        match self
            .client()
            .room_avatar(RoomAvatarRequest { id: room_id.clone() })
            .await
        {
            Ok(response) => Ok(FileBox::from(response.into_inner().filebox)),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get avatar of room {}",
                room_id
            ))),
        }
    }

    async fn room_create(&self, contact_id_list: Vec<String>, topic: Option<String>) -> Result<String, PuppetError> {
        debug!(
            "room_create(contact_id_list = {:?}, topic = {:?})",
            contact_id_list, topic
        );
        match self
            .client()
            .room_create(RoomCreateRequest {
                contact_ids: contact_id_list,
                topic: if let Some(topic) = topic { topic } else { String::new() },
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().id),
            Err(_) => Err(PuppetError::Network("Failed to create room".to_owned())),
        }
    }

    async fn room_del(&self, room_id: String, contact_id: String) -> Result<(), PuppetError> {
        debug!("room_del(room_id = {}, contact_id = {})", room_id, contact_id);
        match self
            .client()
            .room_del(RoomDelRequest {
                id: room_id.clone(),
                contact_id: contact_id.clone(),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to remove contact {} from room {}",
                contact_id, room_id
            ))),
        }
    }

    async fn room_qr_code(&self, room_id: String) -> Result<String, PuppetError> {
        debug!("room_qr_code(room_id = {})", room_id);
        match self
            .client()
            .room_qr_code(RoomQrCodeRequest { id: room_id.clone() })
            .await
        {
            Ok(response) => Ok(response.into_inner().qrcode),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get qrcode of room {}",
                room_id
            ))),
        }
    }

    async fn room_quit(&self, room_id: String) -> Result<(), PuppetError> {
        debug!("room_quit(room_id = {})", room_id);
        match self.client().room_quit(RoomQuitRequest { id: room_id.clone() }).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!("Failed to quit room {}", room_id))),
        }
    }

    async fn room_topic(&self, room_id: String) -> Result<String, PuppetError> {
        debug!("room_topic(room_id = {})", room_id);
        match self
            .client()
            .room_topic(RoomTopicRequest {
                id: room_id.clone(),
                topic: None,
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().topic.unwrap()),
            Err(_) => Err(PuppetError::Network(format!("Failed to get topic of room {}", room_id))),
        }
    }

    async fn room_topic_set(&self, room_id: String, topic: String) -> Result<(), PuppetError> {
        debug!("room_topic_set(room_id = {}, topic = {})", room_id, topic);
        match self
            .client()
            .room_topic(RoomTopicRequest {
                id: room_id.clone(),
                topic: Some(topic),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to set topic for room {}",
                room_id
            ))),
        }
    }

    async fn room_list(&self) -> Result<Vec<String>, PuppetError> {
        debug!("room_list()");
        match self.client().room_list(RoomListRequest {}).await {
            Ok(response) => Ok(response.into_inner().ids),
            Err(_) => Err(PuppetError::Network("Failed to get rooms".to_owned())),
        }
    }

    async fn room_raw_payload(&self, room_id: String) -> Result<RoomPayload, PuppetError> {
        debug!("room_raw_payload(room_id = {})", room_id);
        match self
            .client()
            .room_payload(RoomPayloadRequest { id: room_id.clone() })
            .await
        {
            Ok(response) => Ok(RoomPayload::from_payload_response(response.into_inner())),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get raw payload for room {}",
                room_id
            ))),
        }
    }

    async fn room_announce(&self, room_id: String) -> Result<String, PuppetError> {
        debug!("room_announce(room_id = {})", room_id);
        match self
            .client()
            .room_announce(RoomAnnounceRequest {
                id: room_id.clone(),
                text: None,
            })
            .await
        {
            Ok(response) => Ok(response.into_inner().text.unwrap()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get announce of room {}",
                room_id
            ))),
        }
    }

    async fn room_announce_set(&self, room_id: String, text: String) -> Result<(), PuppetError> {
        debug!("room_announce(room_id = {}, text = {})", room_id, text);
        match self
            .client()
            .room_announce(RoomAnnounceRequest {
                id: room_id.clone(),
                text: Some(text),
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to set announce for room {}",
                room_id
            ))),
        }
    }

    async fn room_member_list(&self, room_id: String) -> Result<Vec<String>, PuppetError> {
        debug!("room_member_list(room_id = {})", room_id);
        match self
            .client()
            .room_member_list(RoomMemberListRequest { id: room_id.clone() })
            .await
        {
            Ok(response) => Ok(response.into_inner().member_ids),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get members of room {}",
                room_id
            ))),
        }
    }

    async fn room_member_raw_payload(
        &self,
        room_id: String,
        contact_id: String,
    ) -> Result<RoomMemberPayload, PuppetError> {
        debug!(
            "room_member_raw_payload(room_id = {}, contact_id = {})",
            room_id, contact_id
        );
        match self
            .client()
            .room_member_payload(RoomMemberPayloadRequest {
                id: room_id.clone(),
                member_id: contact_id.clone(),
            })
            .await
        {
            Ok(response) => Ok(RoomMemberPayload::from_payload_response(response.into_inner())),
            Err(_) => Err(PuppetError::Network(format!(
                "Failed to get raw payload for member {} of room {}",
                contact_id, room_id
            ))),
        }
    }

    async fn start(&self) -> Result<(), PuppetError> {
        debug!("start()");
        match self.client().start(StartRequest {}).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network("Failed to start puppet".to_owned())),
        }
    }

    async fn stop(&self) -> Result<(), PuppetError> {
        debug!("stop()");
        match self.client().stop(StopRequest {}).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network("Failed to stop puppet".to_owned())),
        }
    }

    async fn ding(&self, data: String) -> Result<(), PuppetError> {
        debug!("ding(data = {})", data);
        match self.client().ding(DingRequest { data }).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network("Failed to ding".to_owned())),
        }
    }

    async fn version(&self) -> Result<String, PuppetError> {
        debug!("version()");
        match self.client().version(VersionRequest {}).await {
            Ok(response) => Ok(response.into_inner().version),
            Err(_) => Err(PuppetError::Network("Failed to get puppet version".to_owned())),
        }
    }

    async fn logout(&self) -> Result<(), PuppetError> {
        debug!("logout()");
        match self.client().logout(LogoutRequest {}).await {
            Ok(_) => Ok(()),
            Err(_) => Err(PuppetError::Network("Failed to logout".to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn cannot_create_puppet_service_with_invalid_token() {
        let invalid_token = uuid::Uuid::new_v4().to_string();

        match PuppetService::new(PuppetOptions {
            endpoint: None,
            timeout: None,
            token: Some(invalid_token),
        })
        .await
        {
            Err(e) => println!("Failed to create puppet service: {}", e),
            Ok(_) => println!("Create puppet service successfully"),
        }
    }
}
