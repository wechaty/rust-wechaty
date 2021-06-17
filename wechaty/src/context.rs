use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use futures::StreamExt;
use log::{debug, error};
use wechaty_puppet::{
    ContactPayload, ContactQueryFilter, FriendshipPayload, FriendshipSearchQueryFilter, MessagePayload,
    MessageQueryFilter, Puppet, PuppetImpl, RoomInvitationPayload, RoomPayload, RoomQueryFilter,
};

use crate::{Contact, Friendship, IntoContact, Message, Room, WechatyError};

#[derive(Clone)]
pub struct WechatyContext<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    id_: Option<String>,
    puppet_: Puppet<T>,
    contacts_: Arc<Mutex<HashMap<String, ContactPayload>>>,
    friendships_: Arc<Mutex<HashMap<String, FriendshipPayload>>>,
    messages_: Arc<Mutex<HashMap<String, MessagePayload>>>,
    rooms_: Arc<Mutex<HashMap<String, RoomPayload>>>,
    room_invitations_: Arc<Mutex<HashMap<String, RoomInvitationPayload>>>,
}

impl<T> WechatyContext<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub(crate) fn new(puppet: Puppet<T>) -> Self {
        Self {
            id_: None,
            puppet_: puppet,
            contacts_: Arc::new(Mutex::new(Default::default())),
            friendships_: Arc::new(Mutex::new(Default::default())),
            messages_: Arc::new(Mutex::new(Default::default())),
            rooms_: Arc::new(Mutex::new(Default::default())),
            room_invitations_: Arc::new(Mutex::new(Default::default())),
        }
    }

    pub(crate) fn puppet(&self) -> Puppet<T> {
        self.puppet_.clone()
    }

    pub(crate) fn contacts(&self) -> MutexGuard<HashMap<String, ContactPayload>> {
        self.contacts_.lock().unwrap()
    }

    pub(crate) fn friendships(&self) -> MutexGuard<HashMap<String, FriendshipPayload>> {
        self.friendships_.lock().unwrap()
    }

    pub(crate) fn messages(&self) -> MutexGuard<HashMap<String, MessagePayload>> {
        self.messages_.lock().unwrap()
    }

    pub(crate) fn rooms(&self) -> MutexGuard<HashMap<String, RoomPayload>> {
        self.rooms_.lock().unwrap()
    }

    pub(crate) fn room_invitations(&self) -> MutexGuard<HashMap<String, RoomInvitationPayload>> {
        self.room_invitations_.lock().unwrap()
    }

    pub(crate) fn id(&self) -> Option<String> {
        self.id_.clone()
    }

    pub(crate) fn set_id(&mut self, id: String) {
        self.id_ = Some(id);
    }

    pub(crate) fn clear_id(&mut self) {
        self.id_ = None;
    }

    pub(crate) fn is_logged_in(&self) -> bool {
        self.id_.is_some()
    }

    /// Load a contact.
    ///
    /// Use contact store first, if the contact cannot be found in the local store,
    /// try to fetch from the puppet instead.
    pub(crate) async fn contact_load(&self, contact_id: String) -> Result<Contact<T>, WechatyError> {
        debug!("contact_load(query = {})", contact_id);
        let payload = self.contacts().get(&contact_id).cloned();
        match payload {
            Some(payload) => Ok(Contact::new(contact_id.clone(), self.clone(), Some(payload))),
            None => {
                let mut contact = Contact::new(contact_id.clone(), self.clone(), None);
                if let Err(e) = contact.sync().await {
                    error!("Failed to get payload of contact {}", contact_id);
                    return Err(e);
                }
                Ok(contact)
            }
        }
    }

    /// Batch load contacts with a default batch size of 16.
    ///
    /// Reference: [Batch execution of futures in the tokio runtime](https://users.rust-lang.org/t/batch-execution-of-futures-in-the-tokio-runtime-or-max-number-of-active-futures-at-a-time/47659).
    ///
    /// Note the API change: `tokio::stream::iter` is now temporarily `tokio_stream::iter`, according to
    /// [tokio's tutorial](https://tokio.rs/tokio/tutorial/streams), it will be moved back to the `tokio`
    /// crate when the `Stream` trait is stable.
    pub(crate) async fn contact_load_batch(&self, contact_id_list: Vec<String>) -> Vec<Contact<T>> {
        debug!("contact_load_batch(contact_id_list = {:?})", contact_id_list);
        let mut contact_list = vec![];
        let mut stream = tokio_stream::iter(contact_id_list)
            .map(|contact_id| self.contact_load(contact_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(contact) = result {
                contact_list.push(contact);
            }
        }
        contact_list
    }

    /// Find the first contact that matches the query
    pub async fn contact_find(&self, query: ContactQueryFilter) -> Result<Option<Contact<T>>, WechatyError> {
        debug!("contact_find(query = {:?})", query);
        match self.contact_find_all(Some(query)).await {
            Ok(contact_list) => {
                if contact_list.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(contact_list[0].clone()))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Find the first contact that matches the query string
    pub async fn contact_find_by_string(&self, query_str: String) -> Result<Option<Contact<T>>, WechatyError> {
        debug!("contact_find_by_string(query_str = {:?})", query_str);
        match self.contact_find_all_by_string(query_str).await {
            Ok(contact_list) => {
                if contact_list.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(contact_list[0].clone()))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Find all contacts that match the query
    pub async fn contact_find_all(&self, query: Option<ContactQueryFilter>) -> Result<Vec<Contact<T>>, WechatyError> {
        debug!("contact_find_all(query = {:?})", query);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        let query = match query {
            Some(query) => query,
            None => ContactQueryFilter::default(),
        };
        match self.puppet().contact_search(query, None).await {
            Ok(contact_id_list) => Ok(self.contact_load_batch(contact_id_list).await),
            Err(e) => Err(WechatyError::from(e)),
        }
    }

    /// Find all contacts that match the query string
    pub async fn contact_find_all_by_string(&self, query_str: String) -> Result<Vec<Contact<T>>, WechatyError> {
        debug!("contact_find_all_by_string(query_str = {:?})", query_str);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        match self.puppet().contact_search_by_string(query_str, None).await {
            Ok(contact_id_list) => Ok(self.contact_load_batch(contact_id_list).await),
            Err(e) => Err(WechatyError::from(e)),
        }
    }

    /// Load a message.
    ///
    /// Use message store first, if the message cannot be found in the local store,
    /// try to fetch from the puppet instead.
    pub(crate) async fn message_load(&self, message_id: String) -> Result<Message<T>, WechatyError> {
        debug!("message_load(query = {})", message_id);
        let payload = self.messages().get(&message_id).cloned();
        match payload {
            Some(payload) => Ok(Message::new(message_id.clone(), self.clone(), Some(payload))),
            None => {
                let mut message = Message::new(message_id.clone(), self.clone(), None);
                if let Err(e) = message.ready().await {
                    return Err(e);
                }
                Ok(message)
            }
        }
    }

    /// Batch load messages with a default batch size of 16.
    pub(crate) async fn message_load_batch(&self, message_id_list: Vec<String>) -> Vec<Message<T>> {
        debug!("message_load_batch(message_id_list = {:?})", message_id_list);
        let mut message_list = vec![];
        let mut stream = tokio_stream::iter(message_id_list)
            .map(|message_id| self.message_load(message_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(message) = result {
                message_list.push(message);
            }
        }
        message_list
    }

    /// Find the first message that matches the query
    pub async fn message_find(&self, query: MessageQueryFilter) -> Result<Option<Message<T>>, WechatyError> {
        debug!("message_find(query = {:?})", query);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        match self.message_find_all(query).await {
            Ok(message_list) => {
                if message_list.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(message_list[0].clone()))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Find all messages that match the query
    pub async fn message_find_all(&self, query: MessageQueryFilter) -> Result<Vec<Message<T>>, WechatyError> {
        debug!("message_find_all(query = {:?}", query);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        match self.puppet().message_search(query).await {
            Ok(message_id_list) => Ok(self.message_load_batch(message_id_list).await),
            Err(e) => Err(WechatyError::from(e)),
        }
    }

    /// Load a room.
    ///
    /// Use room store first, if the room cannot be found in the local store,
    /// try to fetch from the puppet instead.
    pub(crate) async fn room_load(&self, room_id: String) -> Result<Room<T>, WechatyError> {
        debug!("room_load(room_id = {})", room_id);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        let payload = self.rooms().get(&room_id).cloned();
        match payload {
            Some(payload) => Ok(Room::new(room_id.clone(), self.clone(), Some(payload))),
            None => {
                let mut room = Room::new(room_id.clone(), self.clone(), None);
                if let Err(e) = room.sync().await {
                    return Err(e);
                }
                Ok(room)
            }
        }
    }

    /// Batch load rooms with a default batch size of 16.
    pub(crate) async fn room_load_batch(&self, room_id_list: Vec<String>) -> Vec<Room<T>> {
        debug!("room_load_batch(room_id_list = {:?})", room_id_list);
        let mut room_list = vec![];
        let mut stream = tokio_stream::iter(room_id_list)
            .map(|room_id| self.room_load(room_id))
            .buffer_unordered(16);
        while let Some(result) = stream.next().await {
            if let Ok(room) = result {
                room_list.push(room);
            }
        }
        room_list
    }

    /// Create a room.
    pub async fn room_create(
        &self,
        contact_list: Vec<Contact<T>>,
        topic: Option<String>,
    ) -> Result<Room<T>, WechatyError> {
        debug!("room_create(contact_list = {:?}, topic = {:?})", contact_list, topic);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        if contact_list.len() < 2 {
            Err(WechatyError::InvalidOperation(
                "Need at least 2 contacts to create a room".to_owned(),
            ))
        } else {
            let contact_id_list = contact_list.into_iter().map(|x| x.id()).collect();
            match self.puppet().room_create(contact_id_list, topic).await {
                Ok(room_id) => {
                    let mut room = Room::new(room_id, self.clone(), None);
                    room.sync().await.unwrap_or_default();
                    Ok(room)
                }
                Err(e) => Err(WechatyError::from(e)),
            }
        }
    }

    /// Find the first room that matches the query
    pub async fn room_find(&self, query: RoomQueryFilter) -> Result<Option<Room<T>>, WechatyError> {
        debug!("room_find(query = {:?})", query);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        match self.room_find_all(query).await {
            Ok(room_list) => {
                if room_list.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(room_list[0].clone()))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Find all rooms that match the query
    pub async fn room_find_all(&self, query: RoomQueryFilter) -> Result<Vec<Room<T>>, WechatyError> {
        debug!("room_find_all(query = {:?}", query);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        match self.puppet().room_search(query).await {
            Ok(room_id_list) => Ok(self.room_load_batch(room_id_list).await),
            Err(e) => Err(WechatyError::from(e)),
        }
    }

    /// Load a friendship.
    ///
    /// Use friendship store first, if the friendship cannot be found in the local store,
    /// try to fetch from the puppet instead.
    #[allow(dead_code)]
    pub(crate) async fn friendship_load(&self, friendship_id: String) -> Result<Friendship<T>, WechatyError> {
        debug!("friendship_load(friendship_id = {})", friendship_id);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        let payload = self.friendships().get(&friendship_id).cloned();
        match payload {
            Some(payload) => Ok(Friendship::new(friendship_id.clone(), self.clone(), Some(payload))),
            None => {
                let mut friendship = Friendship::new(friendship_id.clone(), self.clone(), None);
                if let Err(e) = friendship.ready().await {
                    return Err(e);
                }
                Ok(friendship)
            }
        }
    }

    /// Add friendship with contact.
    pub async fn friendship_add(&self, contact: Contact<T>, hello: Option<String>) -> Result<(), WechatyError> {
        debug!("friendship_add(contact = {}, hello = {:?}", contact, hello);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        match self.puppet().friendship_add(contact.id(), hello).await {
            Ok(_) => Ok(()),
            Err(e) => Err(WechatyError::from(e)),
        }
    }

    /// Search a friendship.
    ///
    /// First search by phone, then search by weixin.
    pub async fn friendship_search(
        &self,
        query: FriendshipSearchQueryFilter,
    ) -> Result<Option<Contact<T>>, WechatyError> {
        debug!("friendship_search(query = {:?}", query);
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        if query.phone.is_none() && query.weixin.is_none() {
            return Err(WechatyError::InvalidOperation(
                "Must specify either phone or weixin".to_owned(),
            ));
        }
        match self.puppet().friendship_search(query).await {
            Ok(Some(contact_id)) => {
                let mut contact = Contact::new(contact_id, self.clone(), None);
                contact.sync().await.unwrap_or_default();
                Ok(Some(contact))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(WechatyError::from(e)),
        }
    }

    /// Logout current account.
    pub async fn logout(&self) -> Result<(), WechatyError> {
        debug!("logout()");
        if !self.is_logged_in() {
            return Err(WechatyError::NotLoggedIn);
        }
        match self.puppet().logout().await {
            Ok(_) => Ok(()),
            Err(e) => Err(WechatyError::from(e)),
        }
    }
}
