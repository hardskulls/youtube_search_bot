
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]


use std::env;

use teloxide::
{
    Bot,
    dispatching::Dispatcher,
    dptree,
    error_handlers::LoggingErrorHandler,
    requests::Requester,
    utils::command::BotCommands,
};
use teloxide::update_listeners::webhooks;

use app::bot::{build_storage, schema_and_storage};
use app::formatting::format_logs;
use app::net;
use internal::commands::Command;
use internal::errors::NetworkError;

#[tokio::main]
async fn main() -> eyre::Result<()>
{
    env_logger::Builder::from_default_env()
        .format(format_logs)
        .try_init()?;

    log::info!("[ LOG ] ⚙ <| Building bot... |>");
    log::info!("[ LOG ] 📝 <| Command description: {} |>", Command::descriptions());

    let token = env!("TELEGRAM_BOT_TOKEN");
    let bot = Bot::new(token);
    
    let port = env!("PORT").parse::<u16>()?;
    let host = env!("HOST");
    let addr = ([0,0,0,0], port).into();
    let url = reqwest::Url::parse(&format!("{host}/bot{token}"))?;

    // bot.delete_webhook().await?;
    bot.set_my_commands(Command::bot_commands()).await?;
    
    // [!!] Must be after `bot.delete_webhook()` [!!]
    let update_listener = net::webhook_with_custom_server(bot.clone(), webhooks::Options::new(addr, url)).await?;
    let err_handler = LoggingErrorHandler::with_custom_text(NetworkError::UpdateListenerError.to_string());

    let (schema, storage) = schema_and_storage(build_storage(None)).await;

    log::info!("[ LOG ] ⚙(✅) <| Build finished |>");
    log::info!("[ LOG ] 🚀 <| Bot is running |> ");

    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![storage])
        .build()
        .dispatch_with_listener(update_listener, err_handler)
        .await;

    Ok(())
}


