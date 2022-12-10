use std::sync::Arc;

use teloxide::
{
    Bot,
    dispatching::{Dispatcher, HandlerExt, UpdateFilterExt},
    dispatching::dialogue::{ErasedStorage, InMemStorage, RedisStorage, Storage, TraceStorage},
    dispatching::dialogue::serializer::Json,
    dispatching::update_listeners::webhooks,
    dptree,
    error_handlers::LoggingErrorHandler,
    requests::Requester,
    types::{Update},
    utils::command::BotCommands
};
use teloxide::types::{CallbackQuery, Message};

use bot_lib::
{
    commands::funcs::{handle_commands, handle_unknown_command, is_other_command},
    commands::types::Command,
    dialogue::funcs::{handle_callback_data, handle_text},
    dialogue::types::{DialogueData},
    errors::types::NetworkError
};

#[tokio::main]
async fn main() -> eyre::Result<()>
{
    // !! All `logs::info!` work only after this line + env variable `RUST_LOG` set to `INFO`. !!
    simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info))?;

    log::info!("[ LOG ] ⚙ <| Building bot... |>");
    log::info!("[ LOG ] 📝 <| Command description: {} |>", Command::descriptions());

    let token = std::env::var("TELEGRAM_BOT_TOKEN")?;
    let bot = Bot::new(&token);

    let storage: Arc<ErasedStorage<DialogueData>> =
        if let Ok(redis_storage) = RedisStorage::open("redis://127.0.0.1:6379", Json).await
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
    let port = std::env::var("PORT")?.parse::<u16>()?;

    let host = std::env::var("HOST")?;
    let addr = ([0,0,0,0], port).into();
    let url = reqwest::Url::parse(&format!("{host}/bot{token}"))?;

    bot.delete_webhook().await?;
    bot.set_my_commands(Command::bot_commands()).await?;

    let message_handler =
        Update::filter_message()
            .enter_dialogue::<Message, ErasedStorage<DialogueData>, DialogueData>()
            .branch(dptree::entry().filter_command::<Command>().endpoint(handle_commands))
            .branch(dptree::filter(is_other_command::<Command>).endpoint(handle_unknown_command))
            .branch(dptree::case![DialogueData { state, last_callback, message_with_kb }].endpoint(handle_text));
    let callback_handler =
        Update::filter_callback_query()
            .enter_dialogue::<CallbackQuery, ErasedStorage<DialogueData>, DialogueData>()
            .endpoint(handle_callback_data);
    let handler =
        dptree::entry()
            .branch(message_handler)
            .branch(callback_handler);
    // let handler =
    //     Update::filter_message()
    //         .enter_dialogue::<Message, ErasedStorage<DialogueData>, DialogueData>()
    //         .branch(dptree::entry().filter_command::<Command>().endpoint(handle_commands))
    //         .branch(dptree::filter(is_other_command::<Command>).endpoint(handle_unknown_command))
    //         .branch(dptree::case![DialogueData { state, last_callback }].endpoint(handle_callback_data).endpoint(handle_text))
    //         // .branch(dptree::case![DialogueData { state, last_callback }].endpoint(handle_callback_data))
    //     ;

    // Must be after `bot.delete_webhook()`.
    let update_listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url)).await?;
    let upd_list_err_handler = LoggingErrorHandler::with_custom_text(NetworkError::UpdateListenerError.to_string());

    log::info!("[ LOG ] ⚙(✅) <| Build finished |>");
    log::info!("[ LOG ] 🚀 <| Bot is running |> ");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .build()
        .dispatch_with_listener(update_listener, upd_list_err_handler)
        .await;

    Ok(())
}


