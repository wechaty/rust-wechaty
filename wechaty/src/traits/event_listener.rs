use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;

use actix::{Actor, ActorFutureExt, AtomicResponse, Context, Handler, Recipient, WrapFuture};
use log::{error, info};
use wechaty_puppet::{
    AsyncFnPtr, EventDongPayload, EventErrorPayload, EventFriendshipPayload, EventHeartbeatPayload, EventLoginPayload,
    EventLogoutPayload, EventMessagePayload, EventReadyPayload, EventResetPayload, EventRoomInvitePayload,
    EventRoomJoinPayload, EventRoomLeavePayload, EventRoomTopicPayload, EventScanPayload, IntoAsyncFnPtr, PayloadType,
    Puppet, PuppetEvent, PuppetImpl, Subscribe,
};

use crate::{
    Contact, ContactSelf, DongPayload, ErrorPayload, Friendship, FriendshipPayload, HeartbeatPayload, IntoContact,
    LoginPayload, LogoutPayload, Message, MessagePayload, ReadyPayload, ResetPayload, Room, RoomInvitation,
    RoomInvitePayload, RoomJoinPayload, RoomLeavePayload, RoomTopicPayload, ScanPayload, WechatyContext,
};

pub trait EventListener<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    fn get_listener(&self) -> &EventListenerInner<T>;
    fn get_puppet(&self) -> Puppet<T>;
    fn get_addr(&self) -> Recipient<PuppetEvent>;
    fn get_name(&self) -> String {
        self.get_listener().name.clone()
    }

    fn on_event_with_handle<Payload>(
        &mut self,
        handler: AsyncFnPtr<Payload, WechatyContext<T>, ()>,
        limit: Option<usize>,
        handlers: HandlersPtr<T, Payload>,
        event_name: &'static str,
    ) -> (&mut Self, usize) {
        if let Err(e) = self.get_puppet().get_subscribe_addr().do_send(Subscribe {
            addr: self.get_addr(),
            name: self.get_name(),
            event_name,
        }) {
            error!("{} failed to subscribe to event {}: {}", self.get_name(), event_name, e);
        }
        let counter = handlers.borrow().len();
        let limit = match limit {
            Some(limit) => limit,
            None => usize::MAX,
        };
        handlers.borrow_mut().push((handler, limit));
        (self, counter)
    }

    fn on_dong<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<DongPayload, WechatyContext<T>, ()>,
    {
        self.on_dong_with_handle(handler, None);
        self
    }

    fn on_dong_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<DongPayload, WechatyContext<T>, ()>,
    {
        let dong_handlers = self.get_listener().dong_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, dong_handlers, "dong")
            .1
    }

    fn on_error<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<ErrorPayload, WechatyContext<T>, ()>,
    {
        self.on_error_with_handle(handler, None);
        self
    }

    fn on_error_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<ErrorPayload, WechatyContext<T>, ()>,
    {
        let error_handlers = self.get_listener().error_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, error_handlers, "error")
            .1
    }

    fn on_friendship<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<FriendshipPayload<T>, WechatyContext<T>, ()>,
    {
        self.on_friendship_with_handle(handler, None);
        self
    }

    fn on_friendship_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<FriendshipPayload<T>, WechatyContext<T>, ()>,
    {
        let friendship_handlers = self.get_listener().friendship_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, friendship_handlers, "friendship")
            .1
    }

    fn on_heartbeat<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<HeartbeatPayload, WechatyContext<T>, ()>,
    {
        self.on_heartbeat_with_handle(handler, None);
        self
    }

    fn on_heartbeat_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<HeartbeatPayload, WechatyContext<T>, ()>,
    {
        let heartbeat_handlers = self.get_listener().heartbeat_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, heartbeat_handlers, "heartbeat")
            .1
    }

    fn on_login<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<LoginPayload<T>, WechatyContext<T>, ()>,
    {
        self.on_login_with_handle(handler, None);
        self
    }

    fn on_login_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<LoginPayload<T>, WechatyContext<T>, ()>,
    {
        let login_handlers = self.get_listener().login_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, login_handlers, "login")
            .1
    }

    fn on_logout<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<LogoutPayload<T>, WechatyContext<T>, ()>,
    {
        self.on_logout_with_handle(handler, None);
        self
    }

    fn on_logout_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<LogoutPayload<T>, WechatyContext<T>, ()>,
    {
        let logout_handlers = self.get_listener().logout_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, logout_handlers, "logout")
            .1
    }

    fn on_message<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<MessagePayload<T>, WechatyContext<T>, ()>,
    {
        self.on_message_with_handle(handler, None);
        self
    }

    fn on_message_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<MessagePayload<T>, WechatyContext<T>, ()>,
    {
        let message_handlers = self.get_listener().message_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, message_handlers, "message")
            .1
    }

    fn on_ready<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<ReadyPayload, WechatyContext<T>, ()>,
    {
        self.on_ready_with_handle(handler, None);
        self
    }

    fn on_ready_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<ReadyPayload, WechatyContext<T>, ()>,
    {
        let ready_handlers = self.get_listener().ready_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, ready_handlers, "ready")
            .1
    }

    fn on_reset<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<ResetPayload, WechatyContext<T>, ()>,
    {
        self.on_reset_with_handle(handler, None);
        self
    }

    fn on_reset_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<ResetPayload, WechatyContext<T>, ()>,
    {
        let reset_handlers = self.get_listener().reset_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, reset_handlers, "reset")
            .1
    }

    fn on_room_invite<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<RoomInvitePayload<T>, WechatyContext<T>, ()>,
    {
        self.on_room_invite_with_handle(handler, None);
        self
    }

    fn on_room_invite_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<RoomInvitePayload<T>, WechatyContext<T>, ()>,
    {
        let room_invite_handlers = self.get_listener().room_invite_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, room_invite_handlers, "room-invite")
            .1
    }

    fn on_room_join<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<RoomJoinPayload<T>, WechatyContext<T>, ()>,
    {
        self.on_room_join_with_handle(handler, None);
        self
    }

    fn on_room_join_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<RoomJoinPayload<T>, WechatyContext<T>, ()>,
    {
        let room_join_handlers = self.get_listener().room_join_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, room_join_handlers, "room-join")
            .1
    }

    fn on_room_leave<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<RoomLeavePayload<T>, WechatyContext<T>, ()>,
    {
        self.on_room_leave_with_handle(handler, None);
        self
    }

    fn on_room_leave_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<RoomLeavePayload<T>, WechatyContext<T>, ()>,
    {
        let room_leave_handlers = self.get_listener().room_leave_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, room_leave_handlers, "room-leave")
            .1
    }

    fn on_room_topic<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<RoomTopicPayload<T>, WechatyContext<T>, ()>,
    {
        self.on_room_topic_with_handle(handler, None);
        self
    }

    fn on_room_topic_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<RoomTopicPayload<T>, WechatyContext<T>, ()>,
    {
        let room_topic_handlers = self.get_listener().room_topic_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, room_topic_handlers, "room-topic")
            .1
    }

    fn on_scan<F>(&mut self, handler: F) -> &mut Self
    where
        F: IntoAsyncFnPtr<ScanPayload, WechatyContext<T>, ()>,
    {
        self.on_scan_with_handle(handler, None);
        self
    }

    fn on_scan_with_handle<F>(&mut self, handler: F, limit: Option<usize>) -> usize
    where
        F: IntoAsyncFnPtr<ScanPayload, WechatyContext<T>, ()>,
    {
        let scan_handlers = self.get_listener().scan_handlers.clone();
        self.on_event_with_handle(handler.into(), limit, scan_handlers, "scan")
            .1
    }
}

type HandlersPtr<T, Payload> = Rc<RefCell<Vec<(AsyncFnPtr<Payload, WechatyContext<T>, ()>, usize)>>>;

#[derive(Clone)]
pub struct EventListenerInner<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    name: String,
    ctx: WechatyContext<T>,
    dong_handlers: HandlersPtr<T, DongPayload>,
    error_handlers: HandlersPtr<T, ErrorPayload>,
    friendship_handlers: HandlersPtr<T, FriendshipPayload<T>>,
    heartbeat_handlers: HandlersPtr<T, HeartbeatPayload>,
    login_handlers: HandlersPtr<T, LoginPayload<T>>,
    logout_handlers: HandlersPtr<T, LogoutPayload<T>>,
    message_handlers: HandlersPtr<T, MessagePayload<T>>,
    ready_handlers: HandlersPtr<T, ReadyPayload>,
    reset_handlers: HandlersPtr<T, ResetPayload>,
    room_invite_handlers: HandlersPtr<T, RoomInvitePayload<T>>,
    room_join_handlers: HandlersPtr<T, RoomJoinPayload<T>>,
    room_leave_handlers: HandlersPtr<T, RoomLeavePayload<T>>,
    room_topic_handlers: HandlersPtr<T, RoomTopicPayload<T>>,
    scan_handlers: HandlersPtr<T, ScanPayload>,
}

impl<T> Actor for EventListenerInner<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("{} started", self.name);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("{} stopped", self.name);
    }
}

impl<T> Handler<PuppetEvent> for EventListenerInner<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    type Result = AtomicResponse<Self, ()>;

    fn handle(&mut self, msg: PuppetEvent, _ctx: &mut Context<Self>) -> Self::Result {
        info!("{} receives puppet event: {:?}", self.name.clone(), msg);
        match msg {
            PuppetEvent::Dong(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_dong_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::Error(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_error_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::Friendship(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_friendship_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::Heartbeat(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_heartbeat_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::Login(payload) => {
                self.ctx.set_id(payload.contact_id.clone());
                AtomicResponse::new(Box::pin(
                    async {}
                        .into_actor(self)
                        .then(move |_, this, _| this.trigger_login_handlers(payload).into_actor(this)),
                ))
            }
            PuppetEvent::Logout(payload) => {
                self.ctx.clear_id();
                AtomicResponse::new(Box::pin(
                    async {}
                        .into_actor(self)
                        .then(move |_, this, _| this.trigger_logout_handlers(payload).into_actor(this)),
                ))
            }
            PuppetEvent::Message(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_message_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::Ready(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_ready_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::Reset(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_reset_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::RoomInvite(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_room_invite_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::RoomJoin(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_room_join_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::RoomLeave(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_room_leave_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::RoomTopic(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_room_topic_handlers(payload).into_actor(this)),
            )),
            PuppetEvent::Scan(payload) => AtomicResponse::new(Box::pin(
                async {}
                    .into_actor(self)
                    .then(move |_, this, _| this.trigger_scan_handlers(payload).into_actor(this)),
            )),
            _ => AtomicResponse::new(Box::pin(async {}.into_actor(self))),
        }
    }
}

impl<T> EventListenerInner<T>
where
    T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
{
    pub(crate) fn new(name: String, ctx: WechatyContext<T>) -> Self {
        Self {
            name,
            ctx,
            dong_handlers: Rc::new(RefCell::new(vec![])),
            error_handlers: Rc::new(RefCell::new(vec![])),
            friendship_handlers: Rc::new(RefCell::new(vec![])),
            heartbeat_handlers: Rc::new(RefCell::new(vec![])),
            login_handlers: Rc::new(RefCell::new(vec![])),
            logout_handlers: Rc::new(RefCell::new(vec![])),
            message_handlers: Rc::new(RefCell::new(vec![])),
            ready_handlers: Rc::new(RefCell::new(vec![])),
            reset_handlers: Rc::new(RefCell::new(vec![])),
            room_invite_handlers: Rc::new(RefCell::new(vec![])),
            room_join_handlers: Rc::new(RefCell::new(vec![])),
            room_leave_handlers: Rc::new(RefCell::new(vec![])),
            room_topic_handlers: Rc::new(RefCell::new(vec![])),
            scan_handlers: Rc::new(RefCell::new(vec![])),
        }
    }

    async fn trigger_handlers<Payload: Clone + 'static>(
        ctx: WechatyContext<T>,
        payload: Payload,
        handlers: HandlersPtr<T, Payload>,
    ) where
        T: 'static + PuppetImpl + Clone + Unpin + Send + Sync,
    {
        let len = handlers.borrow_mut().len();
        for i in 0..len {
            let mut handler = &mut handlers.borrow_mut()[i];
            if handler.1 > 0 {
                handler.0.run(payload.clone(), ctx.clone()).await;
                handler.1 -= 1;
            }
        }
    }

    fn trigger_dong_handlers(&mut self, payload: EventDongPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.dong_handlers.clone();
        async move { EventListenerInner::<T>::trigger_handlers(ctx, payload, handlers).await }
    }

    fn trigger_error_handlers(&mut self, payload: EventErrorPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.error_handlers.clone();
        async move { EventListenerInner::<T>::trigger_handlers(ctx, payload, handlers).await }
    }

    fn trigger_friendship_handlers(&mut self, payload: EventFriendshipPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let mut friendship = Friendship::new(payload.friendship_id, ctx.clone(), None);
        let handlers = self.friendship_handlers.clone();
        async move {
            friendship.ready().await.unwrap_or_default();
            EventListenerInner::<T>::trigger_handlers(ctx, FriendshipPayload { friendship }, handlers).await
        }
    }

    fn trigger_heartbeat_handlers(&mut self, payload: EventHeartbeatPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.heartbeat_handlers.clone();
        async move { EventListenerInner::<T>::trigger_handlers(ctx, payload, handlers).await }
    }

    fn trigger_login_handlers(&mut self, payload: EventLoginPayload) -> impl Future<Output = ()> + 'static {
        let mut contact = ContactSelf::new(payload.contact_id, self.ctx.clone(), None);
        let ctx = self.ctx.clone();
        let handlers = self.login_handlers.clone();
        async move {
            contact.sync().await.unwrap_or_default();
            EventListenerInner::<T>::trigger_handlers(ctx, LoginPayload { contact }, handlers).await
        }
    }

    fn trigger_logout_handlers(&mut self, payload: EventLogoutPayload) -> impl Future<Output = ()> + 'static {
        let mut contact = ContactSelf::new(payload.contact_id.clone(), self.ctx.clone(), None);
        let ctx = self.ctx.clone();
        let handlers = self.logout_handlers.clone();
        async move {
            contact.ready(false).await.unwrap_or_default();
            EventListenerInner::<T>::trigger_handlers(
                ctx,
                LogoutPayload {
                    contact,
                    data: payload.data,
                },
                handlers,
            )
            .await
        }
    }

    fn trigger_message_handlers(&mut self, payload: EventMessagePayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let mut message = Message::new(payload.message_id, ctx.clone(), None);
        let handlers = self.message_handlers.clone();
        async move {
            message.ready().await.unwrap_or_default();
            EventListenerInner::<T>::trigger_handlers(ctx, MessagePayload { message }, handlers).await
        }
    }

    fn trigger_ready_handlers(&mut self, payload: EventReadyPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.ready_handlers.clone();
        async move { EventListenerInner::<T>::trigger_handlers(ctx, payload, handlers).await }
    }

    fn trigger_reset_handlers(&mut self, payload: EventResetPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.reset_handlers.clone();
        async move { EventListenerInner::<T>::trigger_handlers(ctx, payload, handlers).await }
    }

    fn trigger_room_invite_handlers(&mut self, payload: EventRoomInvitePayload) -> impl Future<Output = ()> + 'static {
        let mut room_invitation = RoomInvitation::new(payload.room_invitation_id, self.ctx.clone(), None);
        let ctx = self.ctx.clone();
        let handlers = self.room_invite_handlers.clone();
        async move {
            room_invitation.ready().await.unwrap_or_default();
            EventListenerInner::<T>::trigger_handlers(ctx, RoomInvitePayload { room_invitation }, handlers).await
        }
    }

    fn trigger_room_join_handlers(&mut self, payload: EventRoomJoinPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.room_join_handlers.clone();
        let mut room = Room::new(payload.room_id.clone(), ctx.clone(), None);
        let mut inviter = Contact::new(payload.inviter_id.clone(), ctx.clone(), None);
        async move {
            room.sync().await.unwrap_or_default();
            inviter.sync().await.unwrap_or_default();
            let invitee_list = ctx.contact_load_batch(payload.invitee_id_list).await;
            EventListenerInner::<T>::trigger_handlers(
                ctx,
                RoomJoinPayload {
                    room,
                    invitee_list,
                    inviter,
                    timestamp: payload.timestamp,
                },
                handlers,
            )
            .await
        }
    }

    fn trigger_room_leave_handlers(&mut self, payload: EventRoomLeavePayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.room_leave_handlers.clone();
        let mut room = Room::new(payload.room_id.clone(), ctx.clone(), None);
        let mut remover = Contact::new(payload.remover_id.clone(), ctx.clone(), None);
        async move {
            room.sync().await.unwrap_or_default();
            remover.sync().await.unwrap_or_default();
            let removee_list = ctx.contact_load_batch(payload.removee_id_list.clone()).await;
            EventListenerInner::<T>::trigger_handlers(
                ctx.clone(),
                RoomLeavePayload {
                    room,
                    removee_list,
                    timestamp: payload.timestamp,
                    remover,
                },
                handlers,
            )
            .await;
            let self_id = ctx.id().unwrap();
            if payload.removee_id_list.contains(&self_id) {
                ctx.puppet()
                    .dirty_payload(PayloadType::Room, payload.room_id.clone())
                    .await
                    .unwrap_or_default();
                ctx.puppet()
                    .dirty_payload(PayloadType::RoomMember, payload.room_id)
                    .await
                    .unwrap_or_default();
            }
        }
    }

    fn trigger_room_topic_handlers(&mut self, payload: EventRoomTopicPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.room_topic_handlers.clone();
        let mut room = Room::new(payload.room_id.clone(), ctx.clone(), None);
        let mut changer = Contact::new(payload.changer_id.clone(), ctx.clone(), None);
        async move {
            room.sync().await.unwrap_or_default();
            changer.sync().await.unwrap_or_default();
            EventListenerInner::<T>::trigger_handlers(
                ctx,
                RoomTopicPayload {
                    room,
                    old_topic: payload.old_topic,
                    new_topic: payload.new_topic,
                    changer,
                    timestamp: payload.timestamp,
                },
                handlers,
            )
            .await
        }
    }

    fn trigger_scan_handlers(&mut self, payload: EventScanPayload) -> impl Future<Output = ()> + 'static {
        let ctx = self.ctx.clone();
        let handlers = self.scan_handlers.clone();
        async move { EventListenerInner::<T>::trigger_handlers(ctx, payload, handlers).await }
    }
}
