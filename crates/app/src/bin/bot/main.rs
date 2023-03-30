#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use std::env;
use std::sync::Arc;

use teloxide::
{
    Bot,
    dispatching::{Dispatcher, HandlerExt, UpdateFilterExt},
    dispatching::dialogue::{ErasedStorage, InMemStorage, RedisStorage, Storage, TraceStorage},
    dispatching::dialogue::serializer::Json,
    dptree,
    error_handlers::LoggingErrorHandler,
    requests::Requester,
    types::Update,
    utils::command::BotCommands,
};
use teloxide::update_listeners::webhooks;

use bot_lib::commands::{Command, handle_commands, handle_unknown_command, is_other_command};
use bot_lib::dialogue::{handle_callback_data, handle_text};
use bot_lib::dialogue::types::DialogueData;
use bot_lib::errors::NetworkError;

#[tokio::main]
async fn main() -> eyre::Result<()>
{
    // !! All `logs::info!` work only after this line + env variable `RUST_LOG` set to `INFO`. !!
    simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info))?;

    log::info!("[ LOG ] ⚙ <| Building bot... |>");
    log::info!("[ LOG ] 📝 <| Command description: {} |>", Command::descriptions());

    let token = env::var("TELEGRAM_BOT_TOKEN")?;
    let bot = Bot::new(&token);

    let redis_url = env::var("REDIS_URL")?;
    let storage: Arc<ErasedStorage<DialogueData>> =
        if let Ok(redis_storage) = RedisStorage::open(redis_url, Json).await
        {
            log::info!("[ LOG ] 💾 <| Using `RedisStorage` to store dialogue state. |> ");
            TraceStorage::new(redis_storage).erase()
        }
        else
        {
            log::info!("[ LOG ] 💾(❌) <| Failed to get `RedisStorage` storage and `SqliteStorage` storage. |> ");
            log::info!("[ LOG ] 💾(✅) <| Using `InMemStorage` to store dialogue state. |> ");
            TraceStorage::new(InMemStorage::<DialogueData>::new()).erase()
        };
    
    let port = env::var("PORT")?.parse::<u16>()?;
    let host = env::var("HOST")?;
    let addr = ([0,0,0,0], port).into();
    let url = reqwest::Url::parse(&format!("{host}/bot{token}"))?;

    bot.delete_webhook().await?;
    bot.set_my_commands(Command::bot_commands()).await?;
    
    let message_handler =
        Update::filter_message()
            .branch(dptree::entry().filter_command::<Command>().endpoint(handle_commands))
            .branch(dptree::filter(is_other_command::<Command>).endpoint(handle_unknown_command))
            .branch(dptree::case![DialogueData { state, last_callback, message_with_kb }].endpoint(handle_text));
    let callback_handler =
        Update::filter_callback_query()
            .endpoint(handle_callback_data);
    let main_handler =
        dptree::entry()
            .enter_dialogue::<Update, ErasedStorage<DialogueData>, DialogueData>()
            .branch(message_handler)
            .branch(callback_handler);
    
    // [!!] Must be after `bot.delete_webhook()` [!!]
    let update_listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url)).await?;
    let err_handler = LoggingErrorHandler::with_custom_text(NetworkError::UpdateListenerError.to_string());

    log::info!("[ LOG ] ⚙(✅) <| Build finished |>");
    log::info!("[ LOG ] 🚀 <| Bot is running |> ");

    Dispatcher::builder(bot, main_handler)
        .dependencies(dptree::deps![storage])
        .build()
        .dispatch_with_listener(update_listener, err_handler)
        .await;

    Ok(())
}


