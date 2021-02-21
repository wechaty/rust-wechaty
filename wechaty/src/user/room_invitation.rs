use std::fmt;

use log::{debug, error};
use wechaty_puppet::{PuppetImpl, RoomInvitationPayload};

use crate::{Entity, WechatyContext, WechatyError};

pub type RoomInvitation<T> = Entity<T, RoomInvitationPayload>;

impl<T> RoomInvitation<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub(crate) fn new(id: String, ctx: WechatyContext<T>, payload: Option<RoomInvitationPayload>) -> Self {
        debug!("create room invitation {}", id);
        let payload = match payload {
            Some(_) => payload,
            None => match ctx.room_invitations().get(&id) {
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

    pub async fn accept(&self) -> Result<(), WechatyError> {
        debug!("RoomInvitation.accept(id = {})", self.id_);
        match self.ctx().puppet().room_invitation_accept(self.id()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(WechatyError::from(e)),
        }
    }

    pub(crate) async fn ready(&mut self) -> Result<(), WechatyError> {
        debug!("RoomInvitation.ready(id = {})", self.id_);
        if self.is_ready() {
            Ok(())
        } else {
            let puppet = self.ctx_.puppet();
            match puppet.room_invitation_payload(self.id()).await {
                Ok(payload) => {
                    self.ctx_.room_invitations().insert(self.id(), payload.clone());
                    self.payload_ = Some(payload.clone());
                    if !payload.inviter_id.is_empty() {
                        let _result = self.ctx_.contact_load(payload.inviter_id.clone()).await;
                    }
                    if !payload.receiver_id.is_empty() {
                        let _result = self.ctx_.contact_load(payload.receiver_id.clone()).await;
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Error occurred while syncing room_invitation {}: {}", self.id_, e);
                    Err(WechatyError::from(e))
                }
            }
        }
    }
}

impl<T> fmt::Debug for RoomInvitation<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "RoomInvitation({})", self)
    }
}

impl<T> fmt::Display for RoomInvitation<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.id())
    }
}
