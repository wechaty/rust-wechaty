mod context;
mod error;
mod payload;
mod traits;
mod user;
mod wechaty;

pub use actix_rt as wechaty_rt;
pub use wechaty_puppet::{MessageType, PuppetOptions};

pub use crate::context::WechatyContext;
pub use crate::error::WechatyError;
pub use crate::payload::*;
pub use crate::traits::contact::IntoContact;
pub use crate::traits::event_listener::EventListener;
pub(crate) use crate::traits::event_listener::EventListenerInner;
pub use crate::user::contact::Contact;
pub use crate::user::contact_self::ContactSelf;
pub(crate) use crate::user::entity::Entity;
pub use crate::user::favorite::Favorite;
pub use crate::user::friendship::Friendship;
pub use crate::user::image::Image;
pub use crate::user::location::Location;
pub use crate::user::message::Message;
pub use crate::user::mini_program::MiniProgram;
pub use crate::user::moment::Moment;
pub use crate::user::money::Money;
pub use crate::user::room::Room;
pub use crate::user::room_invitation::RoomInvitation;
pub use crate::user::tag::Tag;
pub use crate::user::url_link::UrlLink;
pub use crate::wechaty::Wechaty;

pub mod prelude {
    pub use actix_rt as wechaty_rt;
    pub use wechaty_puppet::{MessageType, PuppetOptions};

    pub use crate::context::WechatyContext;
    pub use crate::error::WechatyError;
    pub use crate::payload::*;
    pub use crate::traits::contact::IntoContact;
    pub use crate::traits::event_listener::EventListener;
    pub use crate::user::contact::Contact;
    pub use crate::user::contact_self::ContactSelf;
    pub use crate::user::favorite::Favorite;
    pub use crate::user::friendship::Friendship;
    pub use crate::user::image::Image;
    pub use crate::user::location::Location;
    pub use crate::user::message::Message;
    pub use crate::user::mini_program::MiniProgram;
    pub use crate::user::moment::Moment;
    pub use crate::user::money::Money;
    pub use crate::user::room::Room;
    pub use crate::user::room_invitation::RoomInvitation;
    pub use crate::user::tag::Tag;
    pub use crate::user::url_link::UrlLink;
    pub use crate::wechaty::Wechaty;
}
