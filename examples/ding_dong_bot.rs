#![feature(async_closure)]
use std::env;

use wechaty::prelude::*;
use wechaty_puppet_service::PuppetService;

#[wechaty_rt::main]
async fn main() {
    env_logger::init();
    let options = PuppetOptions {
        endpoint: match env::var("WECHATY_ENDPOINT") {
            Ok(endpoint) => Some(endpoint),
            Err(_) => None,
        },
        timeout: None,
        token: match env::var("WECHATY_TOKEN") {
            Ok(endpoint) => Some(endpoint),
            Err(_) => None,
        },
    };
    let mut bot = Wechaty::new(PuppetService::new(options).await.unwrap());

    bot.on_scan(async move |payload: ScanPayload, _ctx| {
        if let Some(qrcode) = payload.qrcode {
            println!(
                "Visit {} to log in",
                format!("https://wechaty.js.org/qrcode/{}", qrcode)
            );
        }
    })
    .on_login(
        async move |payload: LoginPayload<PuppetService>, ctx: WechatyContext<PuppetService>| {
            println!("User {} has logged in", payload.contact);
            println!("Contact list: {:?}", ctx.contact_find_all(None).await);
        },
    )
    .on_logout(async move |payload: LogoutPayload<PuppetService>, _ctx| {
        println!("User {} has logged out", payload.contact);
    })
    .on_message(
        async move |payload: MessagePayload<PuppetService>, ctx: WechatyContext<PuppetService>| {
            let mut message = payload.message;
            let mentioned = message.mention_list().await;
            println!(
                "Got message: {}, mentioned: {:?}, age: {}",
                message,
                mentioned,
                message.age()
            );
            if message.is_self() {
                println!("Message discarded because it's outgoing");
                return;
            }
            if message.is_in_room() {
                println!("Message discarded because it's from a room");
                return;
            }
            if let Some(message_type) = message.message_type() {
                if message_type != MessageType::Text {
                    println!("Message discarded because it is not a text");
                } else {
                    let text = message.text().unwrap_or_default();
                    if text == "bye" {
                        println!("Good bye!");
                        ctx.logout().await.unwrap_or_default();
                        return;
                    }
                    if text == "ding" {
                        if let Err(e) = message.reply_text("dong".to_owned()).await {
                            println!("Failed to send message, reason: {}", e);
                        } else {
                            println!("REPLY: dong");
                        }
                        return;
                    }
                    println!("Message discarded because it does not match any keyword");
                }
            }
        },
    )
    .start()
    .await;
}
