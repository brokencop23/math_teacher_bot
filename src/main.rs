use std::error::Error;
use std::time::Duration;
use std::sync::Arc;
use tokio::time::interval;
use tokio::sync::Mutex;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Started app");

    let bot = Bot::from_env();
    let bot_clone = bot.clone();
    let chat_id = Arc::new(Mutex::new(ChatId(0)));  
    let chat_id_clone = Arc::clone(&chat_id);
    
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let chat_id = chat_id_clone.lock().await;
            if let Err(e) = create_task(&bot_clone, *chat_id).await {
                log::error!("Failed to send: {}", e);
            }
        }
    });

    let handler = Update::filter_message().branch(
        dptree::endpoint(move | bot: Bot, msg: Message | {
            let chat_id = Arc::clone(&chat_id);
            async move {
                let mut chat_id_guard = chat_id.lock().await;
                *chat_id_guard = msg.chat.id;
                Ok::<(), Box<dyn Error + Send + Sync>>(())
            }
        })
    );

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn create_task(bot: &Bot, chat_id: ChatId) -> HandlerResult {
    if chat_id != ChatId(0) {
        bot.send_message(chat_id, "Hi").await?;
    }
    Ok(())
}

