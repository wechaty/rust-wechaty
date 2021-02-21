use std::any;
use std::fmt::Debug;

use log::trace;
use wechaty_puppet::PuppetImpl;

use crate::WechatyContext;

#[derive(Clone)]
pub struct Entity<T, Payload>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub(crate) ctx_: WechatyContext<T>,
    pub(crate) id_: String,
    pub(crate) payload_: Option<Payload>,
}

impl<T, Payload> Entity<T, Payload>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
    Payload: Debug + Clone,
{
    /// Get type name
    fn type_name() -> String {
        any::type_name::<Payload>()
            .split("::")
            .last()
            .unwrap()
            .split("Payload")
            .next()
            .unwrap_or_default()
            .to_owned()
    }

    /// Get entity's id.
    pub fn id(&self) -> String {
        trace!("{}.id(id = {})", Entity::<T, Payload>::type_name(), self.id_);
        self.id_.clone()
    }

    /// Check if an entity is ready.
    pub(crate) fn is_ready(&self) -> bool {
        trace!("{}.is_ready(id = {})", Entity::<T, Payload>::type_name(), self.id_);
        match self.payload_ {
            None => false,
            Some(_) => true,
        }
    }

    /// Get the Wechaty context.
    pub(crate) fn ctx(&self) -> WechatyContext<T> {
        trace!("{}.ctx(id = {})", Entity::<T, Payload>::type_name(), self.id_);
        self.ctx_.clone()
    }

    /// Get the entity's payload.
    pub(crate) fn payload(&self) -> Option<Payload> {
        trace!("{}.payload(id = {})", Entity::<T, Payload>::type_name(), self.id_);
        self.payload_.clone()
    }

    /// Set the entity's payload.
    pub(crate) fn set_payload(&mut self, payload: Option<Payload>) {
        trace!(
            "{}.set_payload(id = {}, payload = {:?})",
            Entity::<T, Payload>::type_name(),
            self.id_,
            payload
        );
        self.payload_ = payload;
    }
}
