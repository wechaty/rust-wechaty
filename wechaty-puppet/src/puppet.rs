use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use actix::{Actor, Addr, Context, Handler, Message, Recipient};
use async_trait::async_trait;
use futures::StreamExt;
use log::{debug, error, info};
use lru::LruCache;

use crate::{
    ContactPayload, ContactQueryFilter, FileBox, FriendshipPayload, FriendshipSearchQueryFilter, ImageType, MessagePayload,
    MessageQueryFilter, MessageType, MiniProgramPayload, PayloadType, PuppetError, PuppetEvent, RoomInvitationPayload,
    RoomMemberPayload, RoomMemberQueryFilter, RoomPayload, RoomQueryFilter, UrlLinkPayload,
};

const DEFAULT_CONTACT_CACHE_CAP: usize = 3000;
const DEFAULT_FRIENDSHIP_CACHE_CAP: usize = 300;
const DEFAULT_MESSAGE_CACHE_CAP: usize = 500;
const DEFAULT_ROOM_CACHE_CAP: usize = 500;
const DEFAULT_ROOM_MEMBER_CACHE_CAP: usize = 30000;
const DEFAULT_ROOM_INVITATION_CACHE_CAP: usize = 100;

type LruCachePtr<T> = Arc<Mutex<LruCache<String, T>>>;

#[derive(Clone)]
pub struct Puppet<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    puppet_impl: T,
    addr: Addr<PuppetInner>,
    cache_contact_payload: LruCachePtr<ContactPayload>,
    cache_friendship_payload: LruCachePtr<FriendshipPayload>,
    cache_message_payload: LruCachePtr<MessagePayload>,
    cache_room_payload: LruCachePtr<RoomPayload>,
    cache_room_member_payload: LruCachePtr<RoomMemberPayload>,
    cache_room_invitation_payload: LruCachePtr<RoomInvitationPayload>,
    id: Option<String>,
}

type SubscribersPtr = Arc<Mutex<HashMap<String, Recipient<PuppetEvent>>>>;

#[derive(Message)]
#[rtype("()")]
pub struct Subscribe {
    pub addr: Recipient<PuppetEvent>,
    pub name: String,
    pub event_name: &'static str,
}

#[derive(Message)]
#[rtype("()")]
pub struct UnSubscribe {
    pub name: String,
    pub event_name: &'static str,
}

#[derive(Clone)]
struct PuppetInner {
    dong_subscribers: SubscribersPtr,
    error_subscribers: SubscribersPtr,
    friendship_subscribers: SubscribersPtr,
    heartbeat_subscribers: SubscribersPtr,
    login_subscribers: SubscribersPtr,
    logout_subscribers: SubscribersPtr,
    message_subscribers: SubscribersPtr,
    ready_subscribers: SubscribersPtr,
    reset_subscribers: SubscribersPtr,
    room_invite_subscribers: SubscribersPtr,
    room_join_subscribers: SubscribersPtr,
    room_leave_subscribers: SubscribersPtr,
    room_topic_subscribers: SubscribersPtr,
    scan_subscribers: SubscribersPtr,
}

impl PuppetInner {
    fn new() -> Self {
        Self {
            dong_subscribers: Arc::new(Mutex::new(HashMap::new())),
            error_subscribers: Arc::new(Mutex::new(HashMap::new())),
            friendship_subscribers: Arc::new(Mutex::new(HashMap::new())),
            heartbeat_subscribers: Arc::new(Mutex::new(HashMap::new())),
            login_subscribers: Arc::new(Mutex::new(HashMap::new())),
            logout_subscribers: Arc::new(Mutex::new(HashMap::new())),
            message_subscribers: Arc::new(Mutex::new(HashMap::new())),
            ready_subscribers: Arc::new(Mutex::new(HashMap::new())),
            reset_subscribers: Arc::new(Mutex::new(HashMap::new())),
            room_invite_subscribers: Arc::new(Mutex::new(HashMap::new())),
            room_join_subscribers: Arc::new(Mutex::new(HashMap::new())),
            room_leave_subscribers: Arc::new(Mutex::new(HashMap::new())),
            room_topic_subscribers: Arc::new(Mutex::new(HashMap::new())),
            scan_subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn notify(&self, msg: PuppetEvent, subscribers: SubscribersPtr) {
        for (name, subscriber) in subscribers.lock().unwrap().clone() {
            match subscriber.do_send(msg.clone()) {
                Err(e) => {
                    error!("Failed to notify {} : {}", name, e);
                }
                Ok(_) => {}
            }
        }
    }
}

impl Actor for PuppetInner {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Puppet started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Puppet stopped");
    }
}

impl Handler<Subscribe> for PuppetInner {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        info!("{} is trying to subscribe to {}", msg.name, msg.event_name);
        match msg.event_name {
            "dong" => {
                self.dong_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "error" => {
                self.error_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "friendship" => {
                self.friendship_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "heartbeat" => {
                self.heartbeat_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "login" => {
                self.login_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "logout" => {
                self.logout_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "message" => {
                self.message_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "ready" => {
                self.ready_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "reset" => {
                self.reset_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "room-invite" => {
                self.room_invite_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "room-join" => {
                self.room_join_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "room-leave" => {
                self.room_leave_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "room-topic" => {
                self.room_topic_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            "scan" => {
                self.scan_subscribers.lock().unwrap().insert(msg.name, msg.addr);
            }
            _ => {
                error!("Trying to subscribe to unknown event: {}", msg.name);
            }
        }
    }
}

impl Handler<UnSubscribe> for PuppetInner {
    type Result = ();

    fn handle(&mut self, msg: UnSubscribe, _ctx: &mut Self::Context) -> Self::Result {
        info!("{} is trying to unsubscribe from {}", msg.name, msg.event_name);
        match msg.event_name {
            "dong" => {
                self.dong_subscribers.lock().unwrap().remove(&msg.name);
            }
            "error" => {
                self.error_subscribers.lock().unwrap().remove(&msg.name);
            }
            "friendship" => {
                self.friendship_subscribers.lock().unwrap().remove(&msg.name);
            }
            "heartbeat" => {
                self.heartbeat_subscribers.lock().unwrap().remove(&msg.name);
            }
            "login" => {
                self.login_subscribers.lock().unwrap().remove(&msg.name);
            }
            "logout" => {
                self.logout_subscribers.lock().unwrap().remove(&msg.name);
            }
            "message" => {
                self.message_subscribers.lock().unwrap().remove(&msg.name);
            }
            "ready" => {
                self.ready_subscribers.lock().unwrap().remove(&msg.name);
            }
            "reset" => {
                self.reset_subscribers.lock().unwrap().remove(&msg.name);
            }
            "room-invite" => {
                self.room_invite_subscribers.lock().unwrap().remove(&msg.name);
            }
            "room-join" => {
                self.room_join_subscribers.lock().unwrap().remove(&msg.name);
            }
            "room-leave" => {
                self.room_leave_subscribers.lock().unwrap().remove(&msg.name);
            }
            "room-topic" => {
                self.room_topic_subscribers.lock().unwrap().remove(&msg.name);
            }
            "scan" => {
                self.scan_subscribers.lock().unwrap().remove(&msg.name);
            }
            _ => {
                error!("Trying to unsubscribe from unknown event: {}", msg.name);
            }
        }
    }
}

impl Handler<PuppetEvent> for PuppetInner {
    type Result = ();

    fn handle(&mut self, msg: PuppetEvent, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PuppetEvent::Dong(_) => self.notify(msg, self.dong_subscribers.clone()),
            PuppetEvent::Error(_) => self.notify(msg, self.error_subscribers.clone()),
            PuppetEvent::Friendship(_) => self.notify(msg, self.friendship_subscribers.clone()),
            PuppetEvent::Heartbeat(_) => self.notify(msg, self.heartbeat_subscribers.clone()),
            PuppetEvent::Login(_) => self.notify(msg, self.login_subscribers.clone()),
            PuppetEvent::Logout(_) => self.notify(msg, self.logout_subscribers.clone()),
            PuppetEvent::Message(_) => self.notify(msg, self.message_subscribers.clone()),
            PuppetEvent::Ready(_) => self.notify(msg, self.ready_subscribers.clone()),
            PuppetEvent::Reset(_) => self.notify(msg, self.reset_subscribers.clone()),
            PuppetEvent::RoomInvite(_) => self.notify(msg, self.room_invite_subscribers.clone()),
            PuppetEvent::RoomJoin(_) => self.notify(msg, self.room_join_subscribers.clone()),
            PuppetEvent::RoomLeave(_) => self.notify(msg, self.room_leave_subscribers.clone()),
            PuppetEvent::RoomTopic(_) => self.notify(msg, self.room_topic_subscribers.clone()),
            PuppetEvent::Scan(_) => self.notify(msg, self.scan_subscribers.clone()),
            _ => {}
        }
    }
}

impl<T> Puppet<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub fn new(puppet_impl: T) -> Self {
        let addr = PuppetInner::new().start();

        Self {
            puppet_impl,
            addr,
            cache_contact_payload: Arc::new(Mutex::new(LruCache::new(DEFAULT_CONTACT_CACHE_CAP))),
            cache_friendship_payload: Arc::new(Mutex::new(LruCache::new(DEFAULT_FRIENDSHIP_CACHE_CAP))),
            cache_message_payload: Arc::new(Mutex::new(LruCache::new(DEFAULT_MESSAGE_CACHE_CAP))),
            cache_room_payload: Arc::new(Mutex::new(LruCache::new(DEFAULT_ROOM_CACHE_CAP))),
            cache_room_member_payload: Arc::new(Mutex::new(LruCache::new(DEFAULT_ROOM_MEMBER_CACHE_CAP))),
            cache_room_invitation_payload: Arc::new(Mutex::new(LruCache::new(DEFAULT_ROOM_INVITATION_CACHE_CAP))),
            id: None,
        }
    }

    pub fn self_addr(&self) -> Recipient<PuppetEvent> {
        debug!("self_addr()");
        self.addr.clone().recipient()
    }

    pub fn get_subscribe_addr(&self) -> Recipient<Subscribe> {
        debug!("get_subscribe_addr()");
        self.addr.clone().recipient()
    }

    pub fn get_unsubscribe_addr(&self) -> Recipient<UnSubscribe> {
        debug!("get_unsubscribe_addr()");
        self.addr.clone().recipient()
    }

    pub fn self_id(self) -> Option<String> {
        debug!("self_id()");
        self.id.clone()
    }

    pub fn log_on_off(self) -> bool {
        debug!("log_on_off()");
        match self.id {
            Some(_) => true,
            None => false,
        }
    }

    /*
        Contact
    */

    /// Load a contact by id.
    pub async fn contact_payload(&self, contact_id: String) -> Result<ContactPayload, PuppetError> {
        debug!("contact_payload(contact_id = {})", contact_id);
        let cache = &*self.cache_contact_payload;
        if cache.lock().unwrap().contains(&contact_id) {
            Ok(cache.lock().unwrap().get(&contact_id).unwrap().clone())
        } else {
            match self.puppet_impl.contact_raw_payload(contact_id.clone()).await {
                Ok(payload) => {
                    cache.lock().unwrap().put(contact_id.clone(), payload.clone());
                    Ok(payload)
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Batch load contacts with a default batch size of 16.
    ///
    /// A key point here is that the method called in stream::iter(...).map() cannot hold &mut self.
    ///
    /// Reference: [Batch execution of futures in the tokio runtime](https://users.rust-lang.org/t/batch-execution-of-futures-in-the-tokio-runtime-or-max-number-of-active-futures-at-a-time/47659).
    ///
    /// Note the API change: `tokio::stream::iter` is now temporarily `tokio_stream::iter`, according to
    /// [tokio's tutorial](https://tokio.rs/tokio/tutorial/streams), it will be moved back to the `tokio`
    /// crate when the `Stream` trait is stable.
    async fn contact_payload_batch(&self, contact_id_list: Vec<String>) -> Vec<ContactPayload> {
        debug!("contact_payload_batch(contact_id_list = {:?})", contact_id_list);
        let mut contact_list = vec![];
        let mut stream = tokio_stream::iter(contact_id_list)
            .map(|contact_id| self.contact_payload(contact_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(contact) = result {
                contact_list.push(contact);
            }
        }
        contact_list
    }

    /// Search contacts by string.
    ///
    /// Return all contacts that has an alias or name that matches the query string.
    pub async fn contact_search_by_string(
        &mut self,
        query_str: String,
        search_id_list: Option<Vec<String>>,
    ) -> Result<Vec<String>, PuppetError> {
        debug!("contact_search_by_string(query_str = {})", query_str);
        let search_by_id = self
            .contact_search(
                ContactQueryFilter {
                    alias: None,
                    alias_regex: None,
                    id: Some(query_str.clone()),
                    name: None,
                    name_regex: None,
                    weixin: None,
                },
                search_id_list.clone(),
            )
            .await;
        let search_by_alias = self
            .contact_search(
                ContactQueryFilter {
                    alias: Some(query_str.clone()),
                    alias_regex: None,
                    id: None,
                    name: None,
                    name_regex: None,
                    weixin: None,
                },
                search_id_list,
            )
            .await;
        let mut filtered_contact_id_list = vec![];
        if let Ok(contact_id_list) = search_by_id {
            for contact_id in contact_id_list {
                filtered_contact_id_list.push(contact_id);
            }
        }
        if let Ok(contact_id_list) = search_by_alias {
            for contact_id in contact_id_list {
                filtered_contact_id_list.push(contact_id);
            }
        }
        Ok(filtered_contact_id_list
            .into_iter()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>())
    }

    /// Search contacts by query.
    pub async fn contact_search(
        &mut self,
        query: ContactQueryFilter,
        contact_id_list: Option<Vec<String>>,
    ) -> Result<Vec<String>, PuppetError> {
        debug!("contact_search(query = {:?})", query);
        let contact_id_list = match contact_id_list {
            Some(contact_id_list) => contact_id_list,
            None => match self.puppet_impl.contact_list().await {
                Ok(contact_id_list) => contact_id_list,
                Err(e) => return Err(e),
            },
        };
        debug!("contact_search(search_id_list.len() = {})", contact_id_list.len());

        let filter = Puppet::<T>::contact_query_filter_factory(query);

        Ok(self
            .contact_payload_batch(contact_id_list)
            .await
            .into_iter()
            .filter_map(|payload| {
                if filter(payload.clone()) {
                    Some(payload.id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>())
    }

    fn contact_query_filter_factory(query: ContactQueryFilter) -> impl Fn(ContactPayload) -> bool {
        debug!("contact_query_filter_factory(query = {:?})", query);
        move |payload| -> bool {
            let query = query.clone();
            if let Some(id) = query.id {
                if payload.id != id {
                    return false;
                }
            }
            if let Some(name) = query.name {
                if payload.name != name {
                    return false;
                }
            }
            if let Some(alias) = query.alias {
                if payload.alias != alias {
                    return false;
                }
            }
            if let Some(weixin) = query.weixin {
                if payload.weixin != weixin {
                    return false;
                }
            }
            if let Some(name_regex) = query.name_regex {
                if !name_regex.is_match(&payload.name) {
                    return false;
                }
            }
            if let Some(alias_regex) = query.alias_regex {
                if !alias_regex.is_match(&payload.alias) {
                    return false;
                }
            }
            true
        }
    }

    /*
        Message
    */

    /// Load a message by id.
    pub async fn message_payload(&self, message_id: String) -> Result<MessagePayload, PuppetError> {
        debug!("message_payload(message_id = {})", message_id);
        let cache = &*self.cache_message_payload;
        if cache.lock().unwrap().contains(&message_id) {
            Ok(cache.lock().unwrap().get(&message_id).unwrap().clone())
        } else {
            match self.puppet_impl.message_raw_payload(message_id.clone()).await {
                Ok(payload) => {
                    cache.lock().unwrap().put(message_id.clone(), payload.clone());
                    Ok(payload)
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Batch load messages with a default batch size of 16.
    #[allow(dead_code)]
    async fn message_payload_batch(&mut self, message_id_list: Vec<String>) -> Vec<MessagePayload> {
        debug!("message_payload_batch(message_id_list = {:?})", message_id_list);
        let mut message_list = vec![];
        let mut stream = tokio_stream::iter(message_id_list)
            .map(|message_id| self.message_payload(message_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(message) = result {
                message_list.push(message);
            }
        }
        message_list
    }

    /// Get all cached messages.
    pub fn message_list(&self) -> Vec<String> {
        debug!("message_list()");
        let mut message_id_list = vec![];
        for (key, _val) in self.cache_message_payload.lock().unwrap().iter() {
            message_id_list.push(key.clone());
        }
        message_id_list
    }

    pub async fn message_search(&mut self, query: MessageQueryFilter) -> Result<Vec<String>, PuppetError> {
        debug!("message_search(query = {:?})", query);

        let message_id_list = self.message_list();
        debug!("message_search(message_id_list.len() = {})", message_id_list.len());

        let mut filtered_message_id_list = vec![];
        let filter = Puppet::<T>::message_query_filter_factory(query);
        for message_id in message_id_list {
            if let Ok(payload) = self.message_payload(message_id.clone()).await {
                if filter(payload) {
                    filtered_message_id_list.push(message_id.clone());
                }
            } else {
                error!("Failed to get message payload for {}", message_id);
            }
        }

        Ok(filtered_message_id_list)
    }

    fn message_query_filter_factory(query: MessageQueryFilter) -> impl Fn(MessagePayload) -> bool {
        debug!("message_query_filter_factory(query = {:?})", query);
        move |payload| -> bool {
            let query = query.clone();
            if let Some(id) = query.id {
                if payload.id != id {
                    return false;
                }
            }
            if let Some(message_type) = query.message_type {
                if payload.message_type != message_type {
                    return false;
                }
            }
            if let Some(from_id) = query.from_id {
                if payload.from_id != from_id {
                    return false;
                }
            }
            if let Some(to_id) = query.to_id {
                if payload.to_id != to_id {
                    return false;
                }
            }
            if let Some(room_id) = query.room_id {
                if payload.room_id != room_id {
                    return false;
                }
            }
            if let Some(text) = query.text {
                if payload.text != text {
                    return false;
                }
            }
            if let Some(text_regex) = query.text_regex {
                if !text_regex.is_match(&payload.text) {
                    return false;
                }
            }
            true
        }
    }

    pub async fn message_forward(
        &mut self,
        conversation_id: String,
        message_id: String,
    ) -> Result<Option<String>, PuppetError> {
        debug!(
            "message_forward(conversation_id = {}, message_id = {})",
            conversation_id, message_id
        );
        let payload = self.message_payload(message_id.clone()).await;
        match payload {
            Ok(payload) => match payload.message_type {
                MessageType::Attachment | MessageType::Audio | MessageType::Image | MessageType::Video => {
                    match self.puppet_impl.message_file(message_id).await {
                        Ok(file) => self.puppet_impl.message_send_file(conversation_id, file).await,
                        Err(e) => Err(e),
                    }
                }
                MessageType::Text => {
                    self.puppet_impl
                        .message_send_text(conversation_id, payload.text, Vec::new())
                        .await
                }
                MessageType::MiniProgram => match self.puppet_impl.message_mini_program(message_id).await {
                    Ok(mini_program_payload) => {
                        self.puppet_impl
                            .message_send_mini_program(conversation_id, mini_program_payload)
                            .await
                    }
                    Err(e) => Err(e),
                },
                MessageType::Url => match self.puppet_impl.message_url(message_id).await {
                    Ok(url_link_payload) => {
                        self.puppet_impl
                            .message_send_url(conversation_id, url_link_payload)
                            .await
                    }
                    Err(e) => Err(e),
                },
                MessageType::Contact => match self.puppet_impl.message_contact(message_id).await {
                    Ok(contact_id) => self.puppet_impl.message_send_contact(conversation_id, contact_id).await,
                    Err(e) => Err(e),
                },
                MessageType::ChatHistory
                | MessageType::Location
                | MessageType::Emoticon
                | MessageType::GroupNote
                | MessageType::Transfer
                | MessageType::RedEnvelope
                | MessageType::Recalled => Err(PuppetError::Unsupported(format!(
                    "sending {:?} messages",
                    payload.message_type
                ))),
                MessageType::Unknown => Err(PuppetError::UnknownMessageType),
            },
            Err(e) => Err(e),
        }
    }

    /*
        Friendship
    */

    /// Search friendship.
    ///
    /// First search by phone, then search by weixin.
    pub async fn friendship_search(
        &mut self,
        query: FriendshipSearchQueryFilter,
    ) -> Result<Option<String>, PuppetError> {
        if let Some(phone) = query.phone {
            self.friendship_search_phone(phone).await
        } else if let Some(weixin) = query.weixin {
            self.friendship_search_weixin(weixin).await
        } else {
            Ok(None)
        }
    }

    /// Load a friendship by id.
    pub async fn friendship_payload(&self, friendship_id: String) -> Result<FriendshipPayload, PuppetError> {
        debug!("friendship_payload(friendship_id = {})", friendship_id);
        let cache = &*self.cache_friendship_payload;
        if cache.lock().unwrap().contains(&friendship_id) {
            Ok(cache.lock().unwrap().get(&friendship_id).unwrap().clone())
        } else {
            match self.puppet_impl.friendship_raw_payload(friendship_id.clone()).await {
                Ok(payload) => {
                    cache.lock().unwrap().put(friendship_id.clone(), payload.clone());
                    Ok(payload)
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Batch load friendships with a default batch size of 16.
    #[allow(dead_code)]
    async fn friendship_payload_batch(&mut self, friendship_id_list: Vec<String>) -> Vec<FriendshipPayload> {
        debug!(
            "friendship_payload_batch(friendship_id_list = {:?})",
            friendship_id_list
        );
        let mut friendship_list = vec![];
        let mut stream = tokio_stream::iter(friendship_id_list)
            .map(|friendship_id| self.friendship_payload(friendship_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(friendship) = result {
                friendship_list.push(friendship);
            }
        }
        friendship_list
    }

    /// Friendship payload setter.
    pub async fn friendship_payload_set(
        &mut self,
        friendship_id: String,
        new_payload: FriendshipPayload,
    ) -> Result<(), PuppetError> {
        debug!(
            "friendship_payload_set(id = {}, new_payload = {:?})",
            friendship_id, new_payload
        );
        (*self.cache_friendship_payload)
            .lock()
            .unwrap()
            .put(friendship_id, new_payload);
        Ok(())
    }

    /*
       Room Invitation
    */

    /// Load a room invitation by id.
    pub async fn room_invitation_payload(
        &self,
        room_invitation_id: String,
    ) -> Result<RoomInvitationPayload, PuppetError> {
        debug!("room_invitation_payload(room_invitation_id = {})", room_invitation_id);
        let cache = &*self.cache_room_invitation_payload;
        if cache.lock().unwrap().contains(&room_invitation_id) {
            Ok(cache.lock().unwrap().get(&room_invitation_id).unwrap().clone())
        } else {
            match self
                .puppet_impl
                .room_invitation_raw_payload(room_invitation_id.clone())
                .await
            {
                Ok(payload) => {
                    cache.lock().unwrap().put(room_invitation_id.clone(), payload.clone());
                    Ok(payload)
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Batch load room invitations with a default batch size of 16.
    #[allow(dead_code)]
    async fn room_invitation_payload_batch(
        &mut self,
        room_invitation_id_list: Vec<String>,
    ) -> Vec<RoomInvitationPayload> {
        debug!(
            "room_invitation_payload_batch(room_invitation_id_list = {:?})",
            room_invitation_id_list
        );
        let mut room_invitation_list = vec![];
        let mut stream = tokio_stream::iter(room_invitation_id_list)
            .map(|room_invitation_id| self.room_invitation_payload(room_invitation_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(room_invitation) = result {
                room_invitation_list.push(room_invitation);
            }
        }
        room_invitation_list
    }

    /// Room invitation payload setter.
    pub async fn room_invitation_payload_set(
        &mut self,
        room_invitation_id: String,
        new_payload: RoomInvitationPayload,
    ) -> Result<(), PuppetError> {
        debug!(
            "room_invitation_payload_set(id = {}, new_payload = {:?})",
            room_invitation_id, new_payload
        );
        (*self.cache_room_invitation_payload)
            .lock()
            .unwrap()
            .put(room_invitation_id, new_payload);
        Ok(())
    }

    /*
       Room
    */

    /// Load a room by id.
    pub async fn room_payload(&self, room_id: String) -> Result<RoomPayload, PuppetError> {
        debug!("room_payload(room_id = {})", room_id);
        let cache = &*self.cache_room_payload;
        if cache.lock().unwrap().contains(&room_id) {
            Ok(cache.lock().unwrap().get(&room_id).unwrap().clone())
        } else {
            match self.puppet_impl.room_raw_payload(room_id.clone()).await {
                Ok(payload) => {
                    cache.lock().unwrap().put(room_id.clone(), payload.clone());
                    Ok(payload)
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Batch load rooms with a default batch size of 16.
    async fn room_payload_batch(&mut self, room_id_list: Vec<String>) -> Vec<RoomPayload> {
        debug!("room_payload_batch(room_id_list = {:?})", room_id_list);
        let mut room_list = vec![];
        let mut stream = tokio_stream::iter(room_id_list)
            .map(|room_id| self.room_payload(room_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(room) = result {
                room_list.push(room);
            }
        }
        room_list
    }

    /// Helper function to generate room member cache key.
    fn cache_key_room_member(room_id: String, contact_id: String) -> String {
        format!("{}@@@{}", contact_id, room_id)
    }

    /// Search room members by string.
    pub async fn room_member_search_by_string(
        &mut self,
        room_id: String,
        query_str: String,
    ) -> Result<Vec<String>, PuppetError> {
        debug!("room_member_search_by_string(query_str = {})", query_str);
        let search_by_id = self
            .room_member_search(
                room_id.clone(),
                RoomMemberQueryFilter {
                    name: Some(query_str.clone()),
                    room_alias: None,
                    name_regex: None,
                    room_alias_regex: None,
                },
            )
            .await;
        let search_by_alias = self
            .room_member_search(
                room_id,
                RoomMemberQueryFilter {
                    name: None,
                    room_alias: Some(query_str),
                    name_regex: None,
                    room_alias_regex: None,
                },
            )
            .await;
        let mut filtered_room_member_id_list = vec![];
        if let Ok(room_member_id_list) = search_by_id {
            for room_member_id in room_member_id_list {
                filtered_room_member_id_list.push(room_member_id);
            }
        }
        if let Ok(room_member_id_list) = search_by_alias {
            for room_member_id in room_member_id_list {
                filtered_room_member_id_list.push(room_member_id);
            }
        }
        Ok(filtered_room_member_id_list
            .into_iter()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>())
    }

    /// Search room members.
    ///
    /// Currently, searching by contact alias is not supported.
    pub async fn room_member_search(
        &mut self,
        room_id: String,
        query: RoomMemberQueryFilter,
    ) -> Result<Vec<String>, PuppetError> {
        debug!("room_member_search(query = {:?})", query);
        let member_id_list = match self.puppet_impl.room_member_list(room_id.clone()).await {
            Ok(member_id_list) => member_id_list,
            Err(e) => return Err(e),
        };
        debug!("room_member_search(member_id_list.len() = {})", member_id_list.len());

        let filter = Puppet::<T>::room_member_query_filter_factory(query);

        Ok(self
            .room_member_payload_batch(room_id, member_id_list)
            .await
            .into_iter()
            .filter_map(|payload| {
                if filter(payload.clone()) {
                    Some(payload.id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>())
    }

    fn room_member_query_filter_factory(query: RoomMemberQueryFilter) -> impl Fn(RoomMemberPayload) -> bool {
        debug!("room_member_query_filter_factory(query = {:?})", query);
        move |payload| -> bool {
            let query = query.clone();
            if let Some(name) = query.name {
                if payload.name != name {
                    return false;
                }
            }
            if let Some(room_alias) = query.room_alias {
                if payload.room_alias != room_alias {
                    return false;
                }
            }
            if let Some(name_regex) = query.name_regex {
                if !name_regex.is_match(&payload.name) {
                    return false;
                }
            }
            if let Some(room_alias_regex) = query.room_alias_regex {
                if !room_alias_regex.is_match(&payload.room_alias) {
                    return false;
                }
            }
            true
        }
    }

    /// Batch load room members with a default batch size of 16.
    async fn room_member_payload_batch(&self, room_id: String, member_id_list: Vec<String>) -> Vec<RoomMemberPayload> {
        debug!(
            "room_member_payload_batch(room_id = {}, member_id_list = {:?})",
            room_id, member_id_list
        );
        let mut member_list = vec![];
        let mut stream = tokio_stream::iter(member_id_list)
            .map(|member_id| self.room_member_payload(room_id.clone(), member_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(member) = result {
                member_list.push(member);
            }
        }
        member_list
    }

    /// Load a room member by room id and payload id.
    pub async fn room_member_payload(
        &self,
        room_id: String,
        member_id: String,
    ) -> Result<RoomMemberPayload, PuppetError> {
        debug!("room_member_payload(room_id = {}, member_id = {})", room_id, member_id);
        let cache_key = Puppet::<T>::cache_key_room_member(room_id.clone(), member_id.clone());
        let cache = &*self.cache_room_member_payload;
        if cache.lock().unwrap().contains(&cache_key) {
            Ok(cache.lock().unwrap().get(&cache_key).unwrap().clone())
        } else {
            match self
                .puppet_impl
                .room_member_raw_payload(room_id.clone(), member_id.clone())
                .await
            {
                Ok(payload) => {
                    cache.lock().unwrap().put(cache_key, payload.clone());
                    Ok(payload)
                }
                Err(e) => Err(e),
            }
        }
    }

    pub async fn room_search(&mut self, query: RoomQueryFilter) -> Result<Vec<String>, PuppetError> {
        debug!("room_search(query = {:?})", query);
        let room_id_list = match self.puppet_impl.room_list().await {
            Ok(room_id_list) => room_id_list,
            _ => Vec::new(),
        };
        debug!("room_search(room_id_list.len() = {})", room_id_list.len());

        let filter = Puppet::<T>::room_query_filter_factory(query);

        Ok(self
            .room_payload_batch(room_id_list)
            .await
            .into_iter()
            .filter_map(|payload| {
                if filter(payload.clone()) {
                    Some(payload.id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>())
    }

    fn room_query_filter_factory(query: RoomQueryFilter) -> impl Fn(RoomPayload) -> bool {
        debug!("room_query_filter_factory(query = {:?})", query);
        move |payload| -> bool {
            if let Some(id) = query.clone().id {
                if payload.id != id {
                    return false;
                }
            }
            if let Some(topic) = query.clone().topic {
                if payload.topic != topic {
                    return false;
                }
            }
            if let Some(topic_regex) = query.clone().topic_regex {
                if !topic_regex.is_match(&payload.topic) {
                    return false;
                }
            }
            true
        }
    }

    /*
       Dirty payload
    */

    async fn dirty_payload_message(&mut self, message_id: String) -> Result<(), PuppetError> {
        debug!("dirty_payload_message(message_id = {})", message_id);
        (*self.cache_message_payload).lock().unwrap().pop(&message_id);
        Ok(())
    }

    async fn dirty_payload_contact(&mut self, contact_id: String) -> Result<(), PuppetError> {
        debug!("dirty_payload_contact(contact_id = {})", contact_id);
        (*self.cache_contact_payload).lock().unwrap().pop(&contact_id);
        Ok(())
    }

    async fn dirty_payload_room(&mut self, room_id: String) -> Result<(), PuppetError> {
        debug!("dirty_payload_room(room_id = {})", room_id);
        (*self.cache_contact_payload).lock().unwrap().pop(&room_id);
        Ok(())
    }

    async fn dirty_payload_room_member(&mut self, room_id: String) -> Result<(), PuppetError> {
        debug!("dirty_payload_room_member(room_id = {})", room_id);

        match self.puppet_impl.room_member_list(room_id.clone()).await {
            Ok(contact_id_list) => {
                for contact_id in contact_id_list {
                    let cache_key = Puppet::<T>::cache_key_room_member(room_id.clone(), contact_id);
                    (*self.cache_room_member_payload).lock().unwrap().pop(&cache_key);
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    async fn dirty_payload_friendship(&mut self, friendship_id: String) -> Result<(), PuppetError> {
        debug!("dirty_payload_friendship(friendship_id = {})", friendship_id);
        (*self.cache_friendship_payload).lock().unwrap().pop(&friendship_id);
        Ok(())
    }

    pub async fn dirty_payload(&mut self, payload_type: PayloadType, id: String) -> Result<(), PuppetError> {
        debug!("dirty_payload(payload_type = {:?}, id = {})", payload_type, id);

        match payload_type {
            PayloadType::Message => self.dirty_payload_message(id).await,
            PayloadType::Contact => self.dirty_payload_contact(id).await,
            PayloadType::Room => self.dirty_payload_room(id).await,
            PayloadType::RoomMember => self.dirty_payload_room_member(id).await,
            PayloadType::Friendship => self.dirty_payload_friendship(id).await,
            PayloadType::Unknown => Err(PuppetError::UnknownPayloadType),
        }
    }
}

#[async_trait]
impl<T> PuppetImpl for Puppet<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    async fn contact_self_name_set(&self, name: String) -> Result<(), PuppetError> {
        self.puppet_impl.contact_self_name_set(name).await
    }

    async fn contact_self_qr_code(&self) -> Result<String, PuppetError> {
        self.puppet_impl.contact_self_qr_code().await
    }

    async fn contact_self_signature_set(&self, signature: String) -> Result<(), PuppetError> {
        self.puppet_impl.contact_self_signature_set(signature).await
    }

    async fn tag_contact_add(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.tag_contact_add(tag_id, contact_id).await
    }

    async fn tag_contact_remove(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.tag_contact_remove(tag_id, contact_id).await
    }

    async fn tag_contact_delete(&self, tag_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.tag_contact_delete(tag_id).await
    }

    async fn tag_contact_list(&self, contact_id: String) -> Result<Vec<String>, PuppetError> {
        self.puppet_impl.tag_contact_list(contact_id).await
    }

    async fn tag_list(&self) -> Result<Vec<String>, PuppetError> {
        self.puppet_impl.tag_list().await
    }

    async fn contact_alias(&self, contact_id: String) -> Result<String, PuppetError> {
        self.puppet_impl.contact_alias(contact_id).await
    }

    async fn contact_alias_set(&self, contact_id: String, alias: String) -> Result<(), PuppetError> {
        self.puppet_impl.contact_alias_set(contact_id, alias).await
    }

    async fn contact_avatar(&self, contact_id: String) -> Result<FileBox, PuppetError> {
        self.puppet_impl.contact_avatar(contact_id).await
    }

    async fn contact_avatar_set(&self, contact_id: String, file: FileBox) -> Result<(), PuppetError> {
        self.puppet_impl.contact_avatar_set(contact_id, file).await
    }

    async fn contact_phone_set(&self, contact_id: String, phone_list: Vec<String>) -> Result<(), PuppetError> {
        self.puppet_impl.contact_phone_set(contact_id, phone_list).await
    }

    async fn contact_corporation_remark_set(
        &self,
        contact_id: String,
        corporation_remark: Option<String>,
    ) -> Result<(), PuppetError> {
        self.puppet_impl
            .contact_corporation_remark_set(contact_id, corporation_remark)
            .await
    }

    async fn contact_description_set(
        &self,
        contact_id: String,
        description: Option<String>,
    ) -> Result<(), PuppetError> {
        self.puppet_impl.contact_description_set(contact_id, description).await
    }

    async fn contact_list(&self) -> Result<Vec<String>, PuppetError> {
        self.puppet_impl.contact_list().await
    }

    async fn contact_raw_payload(&self, contact_id: String) -> Result<ContactPayload, PuppetError> {
        self.puppet_impl.contact_raw_payload(contact_id).await
    }

    async fn message_contact(&self, message_id: String) -> Result<String, PuppetError> {
        self.puppet_impl.message_contact(message_id).await
    }

    async fn message_file(&self, message_id: String) -> Result<FileBox, PuppetError> {
        self.puppet_impl.message_file(message_id).await
    }

    async fn message_image(&self, message_id: String, image_type: ImageType) -> Result<FileBox, PuppetError> {
        self.puppet_impl.message_image(message_id, image_type).await
    }

    async fn message_mini_program(&self, message_id: String) -> Result<MiniProgramPayload, PuppetError> {
        self.puppet_impl.message_mini_program(message_id).await
    }

    async fn message_url(&self, message_id: String) -> Result<UrlLinkPayload, PuppetError> {
        self.puppet_impl.message_url(message_id).await
    }

    async fn message_send_contact(
        &self,
        conversation_id: String,
        contact_id: String,
    ) -> Result<Option<String>, PuppetError> {
        self.puppet_impl.message_send_contact(conversation_id, contact_id).await
    }

    async fn message_send_file(&self, conversation_id: String, file: FileBox) -> Result<Option<String>, PuppetError> {
        self.puppet_impl.message_send_file(conversation_id, file).await
    }

    async fn message_send_mini_program(
        &self,
        conversation_id: String,
        mini_program_payload: MiniProgramPayload,
    ) -> Result<Option<String>, PuppetError> {
        self.puppet_impl
            .message_send_mini_program(conversation_id, mini_program_payload)
            .await
    }

    async fn message_send_text(
        &self,
        conversation_id: String,
        text: String,
        mention_id_list: Vec<String>,
    ) -> Result<Option<String>, PuppetError> {
        self.puppet_impl
            .message_send_text(conversation_id, text, mention_id_list)
            .await
    }

    async fn message_send_url(
        &self,
        conversation_id: String,
        url_link_payload: UrlLinkPayload,
    ) -> Result<Option<String>, PuppetError> {
        self.puppet_impl
            .message_send_url(conversation_id, url_link_payload)
            .await
    }

    async fn message_raw_payload(&self, message_id: String) -> Result<MessagePayload, PuppetError> {
        self.puppet_impl.message_raw_payload(message_id).await
    }

    async fn friendship_accept(&self, friendship_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.friendship_accept(friendship_id).await
    }

    async fn friendship_add(&self, contact_id: String, hello: Option<String>) -> Result<(), PuppetError> {
        self.puppet_impl.friendship_add(contact_id, hello).await
    }

    async fn friendship_search_phone(&self, phone: String) -> Result<Option<String>, PuppetError> {
        self.puppet_impl.friendship_search_phone(phone).await
    }

    async fn friendship_search_weixin(&self, weixin: String) -> Result<Option<String>, PuppetError> {
        self.puppet_impl.friendship_search_weixin(weixin).await
    }

    async fn friendship_raw_payload(&self, friendship_id: String) -> Result<FriendshipPayload, PuppetError> {
        self.puppet_impl.friendship_raw_payload(friendship_id).await
    }

    async fn room_invitation_accept(&self, room_invitation_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.room_invitation_accept(room_invitation_id).await
    }

    async fn room_invitation_raw_payload(
        &self,
        room_invitation_id: String,
    ) -> Result<RoomInvitationPayload, PuppetError> {
        self.puppet_impl.room_invitation_raw_payload(room_invitation_id).await
    }

    async fn room_add(&self, room_id: String, contact_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.room_add(room_id, contact_id).await
    }

    async fn room_avatar(&self, room_id: String) -> Result<FileBox, PuppetError> {
        self.puppet_impl.room_avatar(room_id).await
    }

    async fn room_create(&self, contact_id_list: Vec<String>, topic: Option<String>) -> Result<String, PuppetError> {
        self.puppet_impl.room_create(contact_id_list, topic).await
    }

    async fn room_del(&self, room_id: String, contact_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.room_del(room_id, contact_id).await
    }

    async fn room_qr_code(&self, room_id: String) -> Result<String, PuppetError> {
        self.puppet_impl.room_qr_code(room_id).await
    }

    async fn room_quit(&self, room_id: String) -> Result<(), PuppetError> {
        self.puppet_impl.room_quit(room_id).await
    }

    async fn room_topic(&self, room_id: String) -> Result<String, PuppetError> {
        self.puppet_impl.room_topic(room_id).await
    }

    async fn room_topic_set(&self, room_id: String, topic: String) -> Result<(), PuppetError> {
        self.puppet_impl.room_topic_set(room_id, topic).await
    }

    async fn room_list(&self) -> Result<Vec<String>, PuppetError> {
        self.puppet_impl.room_list().await
    }

    async fn room_raw_payload(&self, room_id: String) -> Result<RoomPayload, PuppetError> {
        self.puppet_impl.room_raw_payload(room_id).await
    }

    async fn room_announce(&self, room_id: String) -> Result<String, PuppetError> {
        self.puppet_impl.room_announce(room_id).await
    }

    async fn room_announce_set(&self, room_id: String, text: String) -> Result<(), PuppetError> {
        self.puppet_impl.room_announce_set(room_id, text).await
    }

    async fn room_member_list(&self, room_id: String) -> Result<Vec<String>, PuppetError> {
        self.puppet_impl.room_member_list(room_id).await
    }

    async fn room_member_raw_payload(
        &self,
        room_id: String,
        contact_id: String,
    ) -> Result<RoomMemberPayload, PuppetError> {
        self.puppet_impl.room_member_raw_payload(room_id, contact_id).await
    }

    async fn start(&self) -> Result<(), PuppetError> {
        self.puppet_impl.start().await
    }

    async fn stop(&self) -> Result<(), PuppetError> {
        self.puppet_impl.stop().await
    }

    async fn ding(&self, data: String) -> Result<(), PuppetError> {
        self.puppet_impl.ding(data).await
    }

    async fn version(&self) -> Result<String, PuppetError> {
        self.puppet_impl.version().await
    }

    async fn logout(&self) -> Result<(), PuppetError> {
        self.puppet_impl.logout().await
    }
}

#[async_trait]
pub trait PuppetImpl {
    async fn contact_self_name_set(&self, name: String) -> Result<(), PuppetError>;
    async fn contact_self_qr_code(&self) -> Result<String, PuppetError>;
    async fn contact_self_signature_set(&self, signature: String) -> Result<(), PuppetError>;

    async fn tag_contact_add(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError>;
    async fn tag_contact_remove(&self, tag_id: String, contact_id: String) -> Result<(), PuppetError>;
    async fn tag_contact_delete(&self, tag_id: String) -> Result<(), PuppetError>;
    async fn tag_contact_list(&self, contact_id: String) -> Result<Vec<String>, PuppetError>;
    async fn tag_list(&self) -> Result<Vec<String>, PuppetError>;

    async fn contact_alias(&self, contact_id: String) -> Result<String, PuppetError>;
    async fn contact_alias_set(&self, contact_id: String, alias: String) -> Result<(), PuppetError>;
    async fn contact_avatar(&self, contact_id: String) -> Result<FileBox, PuppetError>;
    async fn contact_avatar_set(&self, contact_id: String, file: FileBox) -> Result<(), PuppetError>;
    async fn contact_phone_set(&self, contact_id: String, phone_list: Vec<String>) -> Result<(), PuppetError>;
    async fn contact_corporation_remark_set(
        &self,
        contact_id: String,
        corporation_remark: Option<String>,
    ) -> Result<(), PuppetError>;
    async fn contact_description_set(&self, contact_id: String, description: Option<String>)
        -> Result<(), PuppetError>;
    async fn contact_list(&self) -> Result<Vec<String>, PuppetError>;
    async fn contact_raw_payload(&self, contact_id: String) -> Result<ContactPayload, PuppetError>;

    async fn message_contact(&self, message_id: String) -> Result<String, PuppetError>;
    async fn message_file(&self, message_id: String) -> Result<FileBox, PuppetError>;
    async fn message_image(&self, message_id: String, image_type: ImageType) -> Result<FileBox, PuppetError>;
    async fn message_mini_program(&self, message_id: String) -> Result<MiniProgramPayload, PuppetError>;
    async fn message_url(&self, message_id: String) -> Result<UrlLinkPayload, PuppetError>;
    async fn message_send_contact(
        &self,
        conversation_id: String,
        contact_id: String,
    ) -> Result<Option<String>, PuppetError>;
    async fn message_send_file(&self, conversation_id: String, file: FileBox) -> Result<Option<String>, PuppetError>;
    async fn message_send_mini_program(
        &self,
        conversation_id: String,
        mini_program_payload: MiniProgramPayload,
    ) -> Result<Option<String>, PuppetError>;
    async fn message_send_text(
        &self,
        conversation_id: String,
        text: String,
        mention_id_list: Vec<String>,
    ) -> Result<Option<String>, PuppetError>;
    async fn message_send_url(
        &self,
        conversation_id: String,
        url_link_payload: UrlLinkPayload,
    ) -> Result<Option<String>, PuppetError>;
    async fn message_raw_payload(&self, message_id: String) -> Result<MessagePayload, PuppetError>;

    async fn friendship_accept(&self, friendship_id: String) -> Result<(), PuppetError>;
    async fn friendship_add(&self, contact_id: String, hello: Option<String>) -> Result<(), PuppetError>;
    async fn friendship_search_phone(&self, phone: String) -> Result<Option<String>, PuppetError>;
    async fn friendship_search_weixin(&self, weixin: String) -> Result<Option<String>, PuppetError>;
    async fn friendship_raw_payload(&self, friendship_id: String) -> Result<FriendshipPayload, PuppetError>;

    async fn room_invitation_accept(&self, room_invitation_id: String) -> Result<(), PuppetError>;
    async fn room_invitation_raw_payload(
        &self,
        room_invitation_id: String,
    ) -> Result<RoomInvitationPayload, PuppetError>;

    async fn room_add(&self, room_id: String, contact_id: String) -> Result<(), PuppetError>;
    async fn room_avatar(&self, room_id: String) -> Result<FileBox, PuppetError>;
    async fn room_create(&self, contact_id_list: Vec<String>, topic: Option<String>) -> Result<String, PuppetError>;
    async fn room_del(&self, room_id: String, contact_id: String) -> Result<(), PuppetError>;
    async fn room_qr_code(&self, room_id: String) -> Result<String, PuppetError>;
    async fn room_quit(&self, room_id: String) -> Result<(), PuppetError>;
    async fn room_topic(&self, room_id: String) -> Result<String, PuppetError>;
    async fn room_topic_set(&self, room_id: String, topic: String) -> Result<(), PuppetError>;
    async fn room_list(&self) -> Result<Vec<String>, PuppetError>;
    async fn room_raw_payload(&self, room_id: String) -> Result<RoomPayload, PuppetError>;

    async fn room_announce(&self, room_id: String) -> Result<String, PuppetError>;
    async fn room_announce_set(&self, room_id: String, text: String) -> Result<(), PuppetError>;
    async fn room_member_list(&self, room_id: String) -> Result<Vec<String>, PuppetError>;
    async fn room_member_raw_payload(
        &self,
        room_id: String,
        contact_id: String,
    ) -> Result<RoomMemberPayload, PuppetError>;

    async fn start(&self) -> Result<(), PuppetError>;
    async fn stop(&self) -> Result<(), PuppetError>;
    async fn ding(&self, data: String) -> Result<(), PuppetError>;
    async fn version(&self) -> Result<String, PuppetError>;
    async fn logout(&self) -> Result<(), PuppetError>;
}
