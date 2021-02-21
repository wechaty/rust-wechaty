use std::fmt;
use std::time::SystemTime;

use log::{debug, error, info};
use wechaty_puppet::{FileBox, MessagePayload, MessageType, MiniProgramPayload, PuppetImpl, UrlLinkPayload};

use crate::{Contact, Entity, IntoContact, Room, WechatyContext, WechatyError};

pub type Message<T> = Entity<T, MessagePayload>;

impl<T> Message<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub(crate) fn new(id: String, ctx: WechatyContext<T>, payload: Option<MessagePayload>) -> Self {
        debug!("create message {}", id);
        let payload = match payload {
            Some(_) => payload,
            None => match ctx.messages().get(&id) {
                Some(payload) => Some(payload.clone()),
                None => None,
            },
        };
        Self {
            id_: id,
            ctx_: ctx,
            payload_: payload,
        }
    }

    /// Check if the message is sent by the user self.
    pub fn is_self(&self) -> bool {
        debug!("Message.is_self(id = {})", self.id_);
        if !self.is_ready() {
            false
        } else {
            self.from().unwrap().is_self()
        }
    }

    /// Check if the message is sent in a room.
    pub fn is_in_room(&self) -> bool {
        debug!("Message.is_in_room(id = {})", self.id_);
        self.room().is_some()
    }

    /// Check if the message mentioned the user self.
    pub fn mentioned_self(&self) -> bool {
        debug!("Message.mentioned_self(id = {})", self.id_);
        if !self.is_ready() || !self.ctx_.is_logged_in() {
            false
        } else {
            self.payload()
                .unwrap()
                .mention_id_list
                .contains(&self.ctx_.id().unwrap())
        }
    }

    pub(crate) async fn ready(&mut self) -> Result<(), WechatyError> {
        debug!("Message.ready(id = {})", self.id_);
        if self.is_ready() {
            Ok(())
        } else {
            let puppet = self.ctx_.puppet();
            match puppet.message_payload(self.id()).await {
                Ok(payload) => {
                    self.ctx_.messages().insert(self.id(), payload.clone());
                    self.payload_ = Some(payload.clone());
                    if !payload.from_id.is_empty() {
                        let _result = self.ctx_.contact_load(payload.from_id.clone()).await;
                    }
                    if !payload.to_id.is_empty() {
                        let _result = self.ctx_.contact_load(payload.to_id.clone()).await;
                    }
                    if !payload.room_id.is_empty() {
                        let _result = self.ctx_.room_load(payload.room_id.clone()).await;
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Error occurred while syncing message {}: {}", self.id_, e);
                    Err(WechatyError::from(e))
                }
            }
        }
    }

    /// Get message's conversation id.
    pub fn conversation_id(&self) -> Option<String> {
        debug!("Message.conversation_id(id = {})", self.id_);
        if self.is_ready() {
            let payload = self.payload().unwrap();
            if !payload.room_id.is_empty() {
                Some(payload.room_id)
            } else if !payload.from_id.is_empty() {
                Some(payload.from_id)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get message's sender.
    pub fn from(&self) -> Option<Contact<T>> {
        debug!("Message.from(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => {
                if !payload.from_id.is_empty() {
                    Some(Contact::new(payload.from_id.clone(), self.ctx_.clone(), None))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Get message's receiver.
    pub fn to(&self) -> Option<Contact<T>> {
        debug!("Message.to(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => {
                if !payload.to_id.is_empty() {
                    Some(Contact::new(payload.to_id.clone(), self.ctx_.clone(), None))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Get the room that the message belongs to.
    pub fn room(&self) -> Option<Room<T>> {
        debug!("Message.room(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => {
                if !payload.room_id.is_empty() {
                    Some(Room::new(payload.room_id.clone(), self.ctx_.clone(), None))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Get message's timestamp.
    pub fn timestamp(&self) -> Option<u64> {
        debug!("Message.timestamp(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => Some(payload.timestamp),
            None => None,
        }
    }

    /// Get message's age in seconds.
    pub fn age(&self) -> u64 {
        debug!("Message.age(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => {
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .max(payload.timestamp)
                    - payload.timestamp
            }
            None => 0,
        }
    }

    /// Get the message type.
    pub fn message_type(&self) -> Option<MessageType> {
        debug!("Message.message_type(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => Some(payload.message_type.clone()),
            None => None,
        }
    }

    /// Get the message's text content, if it is a text message.
    pub fn text(&self) -> Option<String> {
        debug!("Message.text(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => Some(payload.text.clone()),
            None => None,
        }
    }

    /// Get the trimmed version (no mentions) of the message's text content.
    pub async fn text_trimmed(&mut self) -> String {
        unimplemented!()
    }

    /// Get the message's mention list.
    ///
    /// TODO: Analyze message text
    pub async fn mention_list(&mut self) -> Option<Vec<Contact<T>>> {
        debug!("Message.mention_list(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => Some(self.ctx_.contact_load_batch(payload.mention_id_list.clone()).await),
            None => None,
        }
    }

    /// Forward the current message to a conversation (contact or room).
    pub async fn forward(&mut self, conversation_id: String) -> Result<Option<Message<T>>, WechatyError> {
        debug!("Message.forward(id = {}", self.id_);
        match self
            .ctx_
            .puppet()
            .message_forward(conversation_id.clone(), self.id())
            .await
        {
            Ok(Some(message_id)) => {
                info!("Message {} was forwarded to {}", self.id(), conversation_id);
                match self.ctx_.message_load(message_id.clone()).await {
                    Ok(message) => Ok(Some(message)),
                    Err(e) => {
                        error!("Failed to load forwarded message {}, reason: {}", message_id, e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to forward message {}, reason: {}", self.id_, e);
                Err(WechatyError::from(e))
            }
        }
    }

    pub async fn reply_text(&mut self, text: String) -> Result<Option<Message<T>>, WechatyError> {
        debug!("Message.reply_text(id = {}, text = {})", self.id_, text);
        if !self.is_ready() {
            return Err(WechatyError::NoPayload);
        }
        if self.is_in_room() {
            unimplemented!()
        } else {
            self.from().unwrap().send_text(text).await
        }
    }

    pub async fn reply_contact(&mut self, contact_id: String) -> Result<Option<Message<T>>, WechatyError> {
        debug!("Message.reply_contact(id = {}, contact_id = {})", self.id_, contact_id);
        if !self.is_ready() {
            return Err(WechatyError::NoPayload);
        }
        if self.is_in_room() {
            unimplemented!()
        } else {
            self.from().unwrap().send_contact(contact_id).await
        }
    }

    pub async fn reply_file(&mut self, file: FileBox) -> Result<Option<Message<T>>, WechatyError> {
        debug!("Message.reply_file(id = {})", self.id_);
        if !self.is_ready() {
            return Err(WechatyError::NoPayload);
        }
        if self.is_in_room() {
            unimplemented!()
        } else {
            self.from().unwrap().send_file(file).await
        }
    }

    pub async fn reply_mini_program(
        &mut self,
        mini_program: MiniProgramPayload,
    ) -> Result<Option<Message<T>>, WechatyError> {
        debug!(
            "message.reply_mini_program(id = {}, mini_program = {:?})",
            self.id_, mini_program
        );
        if !self.is_ready() {
            return Err(WechatyError::NoPayload);
        }
        if self.is_in_room() {
            unimplemented!()
        } else {
            self.from().unwrap().send_mini_program(mini_program).await
        }
    }

    pub async fn reply_url(&mut self, url: UrlLinkPayload) -> Result<Option<Message<T>>, WechatyError> {
        debug!("Message.reply_url(id = {}, url = {:?})", self.id_, url);
        if !self.is_ready() {
            return Err(WechatyError::NoPayload);
        }
        if self.is_in_room() {
            unimplemented!()
        } else {
            self.from().unwrap().send_url(url).await
        }
    }
}

impl<T> fmt::Debug for Message<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Message({})", self)
    }
}

impl<T> fmt::Display for Message<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = match self.from() {
            Some(contact) => format!("From: {} ", contact),
            None => String::new(),
        };
        let to = match self.to() {
            Some(contact) => format!("To: {} ", contact),
            None => String::new(),
        };
        let room = match self.room() {
            Some(room) => format!("Room: {} ", room),
            None => String::new(),
        };
        let message_type = match self.message_type() {
            Some(message_type) => format!("Type: {:?} ", message_type),
            None => String::new(),
        };
        let text = if self.is_ready() && self.message_type().unwrap() == MessageType::Text {
            let text = self.text().unwrap().chars().collect::<Vec<_>>();
            let len = text.len().min(70);
            format!("Text: {} ", text[0..len].iter().collect::<String>())
        } else {
            String::new()
        };
        write!(fmt, "{}", [from, to, room, message_type, text].join(""))
    }
}
