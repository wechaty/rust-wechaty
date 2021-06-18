use async_trait::async_trait;
use log::{debug, error};
use wechaty_puppet::{FileBox, MiniProgramPayload, PuppetImpl, UrlLinkPayload};

use super::message_load;
use crate::{Message, WechatyContext, WechatyError};

#[async_trait]
pub trait Talkable<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn id(&self) -> String;
    fn ctx(&self) -> WechatyContext<T>;
    fn identity(&self) -> String;

    async fn send_text(&self, text: String) -> Result<Option<Message<T>>, WechatyError> {
        debug!("talkable.send_text(id = {}, text = {})", self.id(), text);
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

    async fn send_contact(&self, contact_id: String) -> Result<Option<Message<T>>, WechatyError> {
        debug!("talkable.send_contact(id = {}, contact_id = {})", self.id(), contact_id);
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

    async fn send_file(&self, file: FileBox) -> Result<Option<Message<T>>, WechatyError> {
        debug!("talkable.send_file(id = {})", self.id());
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

    async fn send_mini_program(&self, mini_program: MiniProgramPayload) -> Result<Option<Message<T>>, WechatyError> {
        debug!(
            "talkable.send_mini_program(id = {}, mini_program = {:?}",
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

    async fn send_url(&self, url: UrlLinkPayload) -> Result<Option<Message<T>>, WechatyError> {
        debug!("talkable.send_url(id = {}, url = {:?})", self.id(), url);
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
