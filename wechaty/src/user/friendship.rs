use std::fmt;

use log::{debug, error};
use wechaty_puppet::{FriendshipPayload, FriendshipType, PuppetImpl};

use crate::{Contact, Entity, IntoContact, WechatyContext, WechatyError};

pub type Friendship<T> = Entity<T, FriendshipPayload>;

impl<T> Friendship<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub(crate) fn new(id: String, ctx: WechatyContext<T>, payload: Option<FriendshipPayload>) -> Self {
        debug!("create friendship {}", id);
        let payload = match payload {
            Some(_) => payload,
            None => match ctx.friendships().get(&id) {
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

    pub(crate) async fn ready(&mut self) -> Result<(), WechatyError> {
        debug!("Friendship.ready(id = {})", self.id_);
        if self.is_ready() {
            Ok(())
        } else {
            let puppet = self.ctx_.puppet();
            match puppet.friendship_payload(self.id()).await {
                Ok(payload) => {
                    self.ctx_.friendships().insert(self.id(), payload.clone());
                    self.payload_ = Some(payload.clone());
                    if !payload.contact_id.is_empty() {
                        let _result = self.ctx_.contact_load(payload.contact_id.clone()).await;
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

    /// Get friendship's type.
    pub fn friendship_type(&self) -> Option<FriendshipType> {
        debug!("Friendship.friendship_type(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => Some(payload.friendship_type.clone()),
            None => None,
        }
    }

    /// Get friendship's contact.
    pub fn contact(&self) -> Option<Contact<T>> {
        debug!("Friendship.contact(id = {})", self.id_);
        match &self.payload_ {
            Some(payload) => {
                if !payload.contact_id.is_empty() {
                    Some(Contact::new(payload.contact_id.clone(), self.ctx_.clone(), None))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Accept a friendship
    pub async fn accept(&mut self) -> Result<(), WechatyError> {
        debug!("Friendship.accept()");
        if !self.is_ready() {
            Err(WechatyError::NoPayload)
        } else if self.friendship_type().unwrap() != FriendshipType::Receive {
            Err(WechatyError::InvalidOperation(
                "Can only accept a friendship of the Receive type".to_owned(),
            ))
        } else {
            match self.ctx().puppet().friendship_accept(self.id()).await {
                Ok(_) => {
                    let mut contact = self.contact().unwrap();
                    contact.sync().await.unwrap_or_default();
                    if contact.is_ready() {
                        Ok(())
                    } else {
                        Err(WechatyError::Maybe(format!(
                            "Failed to accept the friendship, contact: {}",
                            contact
                        )))
                    }
                }
                Err(e) => Err(WechatyError::from(e)),
            }
        }
    }
}

impl<T> fmt::Debug for Friendship<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Friendship({})", self)
    }
}

impl<T> fmt::Display for Friendship<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let friendship_info = if self.is_ready() {
            format!(
                "From: {}",
                match self.contact() {
                    Some(contact) => contact.to_string(),
                    None => "Unknown".to_owned(),
                }
            )
        } else {
            "loading".to_owned()
        };
        write!(fmt, "{}", friendship_info)
    }
}
