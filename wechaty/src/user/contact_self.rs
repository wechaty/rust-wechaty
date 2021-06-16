use std::fmt;

use log::{debug, error};
use wechaty_puppet::{ContactPayload, FileBox, PuppetImpl};

use crate::{Contact, IntoContact, Talkable, WechatyContext, WechatyError};

#[derive(Clone)]
pub struct ContactSelf<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    contact: Contact<T>,
}

impl<T> ContactSelf<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub(crate) fn new(id: String, ctx: WechatyContext<T>, payload: Option<ContactPayload>) -> Self {
        debug!("create contact self {}", id);
        let payload = match payload {
            Some(_) => payload,
            None => match ctx.contacts().get(&id) {
                Some(payload) => Some(payload.clone()),
                None => None,
            },
        };
        Self {
            contact: Contact::new(id, ctx, payload),
        }
    }

    pub async fn set_avatar(&mut self, file: FileBox) -> Result<(), WechatyError> {
        debug!("Contact_self.set_avatar(file = {})", file);

        if !self.is_self() {
            Err(WechatyError::NotLoggedIn)
        } else {
            let puppet = self.ctx().puppet();
            let id = self.id();
            match puppet.contact_avatar_set(id, file).await {
                Ok(_) => {
                    match self.sync().await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to sync contact self after setting avatar, reason: {}", e);
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(WechatyError::from(e)),
            }
        }
    }

    pub async fn set_name(&mut self, name: String) -> Result<(), WechatyError> {
        debug!("Contact_self.set_name(name = {})", name);

        if !self.is_self() {
            Err(WechatyError::NotLoggedIn)
        } else {
            let puppet = self.ctx().puppet();
            match puppet.contact_self_name_set(name).await {
                Ok(_) => {
                    match self.sync().await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to sync contact self after setting name, reason: {}", e);
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(WechatyError::from(e)),
            }
        }
    }

    pub async fn set_signature(&mut self, signature: String) -> Result<(), WechatyError> {
        debug!("Contact_self.set_signature(signature = {})", signature);

        if !self.is_self() {
            Err(WechatyError::NotLoggedIn)
        } else {
            let puppet = self.ctx().puppet();
            match puppet.contact_self_signature_set(signature).await {
                Ok(_) => {
                    match self.sync().await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to sync contact self after setting signature, reason: {}", e);
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(WechatyError::from(e)),
            }
        }
    }

    pub async fn qrcode(&self) -> Result<String, WechatyError> {
        debug!("Contact_self.qrcode()");

        if !self.is_self() {
            Err(WechatyError::NotLoggedIn)
        } else {
            let puppet = self.ctx().puppet();
            match puppet.contact_self_qr_code().await {
                Ok(qrcode) => Ok(qrcode),
                Err(e) => Err(WechatyError::from(e)),
            }
        }
    }
}

impl<T> Talkable<T> for ContactSelf<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn id(&self) -> String {
        self.contact.id()
    }

    fn ctx(&self) -> WechatyContext<T> {
        self.contact.ctx()
    }

    fn identity(&self) -> String {
        self.contact.identity()
    }
}

impl<T> IntoContact<T> for ContactSelf<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn payload(&self) -> Option<ContactPayload> {
        self.contact.payload()
    }

    fn set_payload(&mut self, payload: Option<ContactPayload>) {
        self.contact.set_payload(payload)
    }
}

impl<T> fmt::Debug for ContactSelf<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "ContactSelf({})", self)
    }
}

impl<T> fmt::Display for ContactSelf<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.identity())
    }
}
