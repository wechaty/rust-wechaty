use async_trait::async_trait;
use log::{debug, error, info};
use wechaty_puppet::{
    ContactGender, ContactPayload, FileBox, MiniProgramPayload, PayloadType, PuppetImpl, UrlLinkPayload,
};

use crate::{Message, WechatyContext, WechatyError};

async fn message_load<T>(
    ctx: WechatyContext<T>,
    message_id: String,
    identity: String,
) -> Result<Option<Message<T>>, WechatyError>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    match ctx.message_load(message_id).await {
        Ok(message) => {
            info!("Message sent: {}", message);
            Ok(Some(message))
        }
        Err(e) => {
            error!(
                "Message has been sent to {} but cannot get message payload, reason: {}",
                identity, e
            );
            Ok(None)
        }
    }
}

#[async_trait]
pub trait IntoContact<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn id(&self) -> String;
    fn ctx(&self) -> WechatyContext<T>;
    fn identity(&self) -> String;
    fn payload(&self) -> Option<ContactPayload>;
    fn set_payload(&mut self, payload: Option<ContactPayload>);

    fn is_ready(&self) -> bool {
        debug!("contact.is_ready(id = {})", self.id());
        match self.payload() {
            None => false,
            Some(_) => true,
        }
    }

    async fn ready(&mut self, force_sync: bool) -> Result<(), WechatyError> {
        debug!("contact.ready(id = {}, force_sync = {})", self.id(), force_sync);
        if !force_sync && self.is_ready() {
            Ok(())
        } else {
            let id = self.id();
            let mut puppet = self.ctx().puppet();
            if force_sync {
                if let Err(e) = puppet.dirty_payload(PayloadType::Contact, id.clone()).await {
                    error!("Error occurred while syncing contact {}: {}", id, e);
                    return Err(WechatyError::from(e));
                }
            }
            match puppet.contact_payload(id.clone()).await {
                Ok(payload) => {
                    self.ctx().contacts().insert(id, payload.clone());
                    self.set_payload(Some(payload));
                    Ok(())
                }
                Err(e) => {
                    error!("Error occurred while syncing contact {}: {}", id, e);
                    Err(WechatyError::from(e))
                }
            }
        }
    }

    async fn sync(&mut self) -> Result<(), WechatyError> {
        debug!("contact.sync(id = {})", self.id());
        self.ready(true).await
    }

    fn name(&self) -> Option<String> {
        debug!("contact.name(id = {})", self.id());
        match &self.payload() {
            Some(payload) => Some(payload.name.clone()),
            None => None,
        }
    }

    fn gender(&self) -> Option<ContactGender> {
        debug!("contact.gender(id = {})", self.id());
        match &self.payload() {
            Some(payload) => Some(payload.gender.clone()),
            None => None,
        }
    }

    fn province(&self) -> Option<String> {
        debug!("contact.province(id = {})", self.id());
        match &self.payload() {
            Some(payload) => Some(payload.province.clone()),
            None => None,
        }
    }

    fn city(&self) -> Option<String> {
        debug!("contact.city(id = {})", self.id());
        match &self.payload() {
            Some(payload) => Some(payload.city.clone()),
            None => None,
        }
    }

    fn friend(&self) -> Option<bool> {
        debug!("contact.friend(id = {})", self.id());
        match &self.payload() {
            Some(payload) => Some(payload.friend),
            None => None,
        }
    }

    fn star(&self) -> Option<bool> {
        debug!("contact.star(id = {})", self.id());
        match &self.payload() {
            Some(payload) => Some(payload.star),
            None => None,
        }
    }

    fn alias(&self) -> Option<String> {
        debug!("contact.alias(id = {})", self.id());
        match &self.payload() {
            Some(payload) => Some(payload.alias.clone()),
            None => None,
        }
    }

    async fn set_alias(&mut self, new_alias: String) -> Result<(), WechatyError> {
        debug!("contact.set_alias(id = {}, new_alias = {})", self.id(), new_alias);
        let mut puppet = self.ctx().puppet();
        let id = self.id();
        match puppet.contact_alias_set(id.clone(), new_alias.clone()).await {
            Err(e) => {
                error!("Failed to set alias for {}, reason: {}", self.identity(), e);
                Err(WechatyError::from(e))
            }
            Ok(_) => {
                if let Err(e) = puppet.dirty_payload(PayloadType::Contact, id.clone()).await {
                    error!("Failed to dirty payload for {}, reason: {}", self.identity(), e);
                }
                match puppet.contact_payload(id.clone()).await {
                    Ok(payload) => {
                        if payload.alias != new_alias {
                            error!("Payload is not correctly set.");
                        }
                    }
                    Err(e) => {
                        error!("Failed to verify payload for {}, reason: {}", self.identity(), e);
                    }
                };
                Ok(())
            }
        }
    }

    /// Check if current contact is the bot self.
    fn is_self(&self) -> bool {
        debug!("contact.is_self(id = {})", self.id());
        match self.ctx().id() {
            Some(id) => self.id() == id,
            None => false,
        }
    }

    async fn send_text(&mut self, text: String) -> Result<Option<Message<T>>, WechatyError> {
        debug!("contact.send_text(id = {}, text = {})", self.id(), text);
        let ctx = self.ctx();
        let puppet = ctx.puppet();
        let conversation_id = self.id();
        let message_id = match puppet.message_send_text(conversation_id, text, vec![]).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                error!("Message has been sent to {} but cannot get message id", self.identity());
                return Ok(None);
            }
            Err(e) => return Err(WechatyError::from(e)),
        };
        let identity = self.identity();
        message_load(ctx, message_id, identity).await
    }

    async fn send_contact(&mut self, contact_id: String) -> Result<Option<Message<T>>, WechatyError> {
        debug!("contact.send_contact(id = {}, contact_id = {})", self.id(), contact_id);
        let ctx = self.ctx();
        let puppet = ctx.puppet();
        let conversation_id = self.id();
        let message_id = match puppet.message_send_contact(conversation_id, contact_id).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                error!("Message has been sent to {} but cannot get message id", self.identity());
                return Ok(None);
            }
            Err(e) => return Err(WechatyError::from(e)),
        };
        let identity = self.identity();
        message_load(ctx, message_id, identity).await
    }

    async fn send_file(&mut self, file: FileBox) -> Result<Option<Message<T>>, WechatyError> {
        debug!("contact.send_file(id = {})", self.id());
        let ctx = self.ctx();
        let puppet = ctx.puppet();
        let conversation_id = self.id();
        let message_id = match puppet.message_send_file(conversation_id, file).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                error!("Message has been sent to {} but cannot get message id", self.identity());
                return Ok(None);
            }
            Err(e) => return Err(WechatyError::from(e)),
        };
        let identity = self.identity();
        message_load(ctx, message_id, identity).await
    }

    async fn send_mini_program(
        &mut self,
        mini_program: MiniProgramPayload,
    ) -> Result<Option<Message<T>>, WechatyError> {
        debug!(
            "contact.send_mini_program(id = {}, mini_program = {:?}",
            self.id(),
            mini_program
        );
        let ctx = self.ctx();
        let puppet = ctx.puppet();
        let conversation_id = self.id();
        let message_id = match puppet.message_send_mini_program(conversation_id, mini_program).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                error!("Message has been sent to {} but cannot get message id", self.identity());
                return Ok(None);
            }
            Err(e) => return Err(WechatyError::from(e)),
        };
        let identity = self.identity();
        message_load(ctx, message_id, identity).await
    }

    async fn send_url(&mut self, url: UrlLinkPayload) -> Result<Option<Message<T>>, WechatyError> {
        debug!("contact.send_url(id = {}, url = {:?})", self.id(), url);
        let ctx = self.ctx();
        let puppet = ctx.puppet();
        let conversation_id = self.id();
        let message_id = match puppet.message_send_url(conversation_id, url).await {
            Ok(Some(id)) => id,
            Ok(None) => {
                error!("Message has been sent to {} but cannot get message id", self.identity());
                return Ok(None);
            }
            Err(e) => return Err(WechatyError::from(e)),
        };
        let identity = self.identity();
        message_load(ctx, message_id, identity).await
    }
}
