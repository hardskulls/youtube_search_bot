use teloxide::dispatching::dialogue::GetChatId;
use teloxide::Bot;

use crate::model::dialogue::types::TheDialogue;
use crate::StdResult;
use teloxide::types::CallbackQuery;

use crate::view::funcs::update_view;

pub async fn handle_callback(
    bot: Bot,
    callback: CallbackQuery,
    dialogue: TheDialogue,
) -> StdResult<(), ()> {
    log::info!(" [:: LOG ::]     @[fn]:[controllers::handle_callback] :: [Started]");

    let err_msg = eyre::eyre!("No 'chat_id' in CallbackQuery");
    let log_err = || {
        log::error!(" [:: LOG ::]   (@[fn]:[controllers::handle_callback] error : ( {err_msg} )");
    };

    let chat_id = callback.chat_id().ok_or_else(log_err)?;

    let sendable = crate::model::handlers::callback::common::handle_callback(
        callback.clone(),
        dialogue.clone(),
    )
    .await;
    update_view(&bot, chat_id, sendable, dialogue, callback.into()).await;
    Ok(())
}
