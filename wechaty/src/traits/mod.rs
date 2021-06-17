pub(crate) mod contact;
pub(crate) mod event_listener;
pub(crate) mod talkable;

use log::{error, info};
use wechaty_puppet::PuppetImpl;

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
