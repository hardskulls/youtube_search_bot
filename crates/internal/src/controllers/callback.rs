
use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;

use teloxide::types::CallbackQuery;
use crate::model::dialogue::types::TheDialogue;
use crate::StdResult;

use crate::view::funcs::update_view;

pub async fn handle_callback(bot: Bot, callback: CallbackQuery, dialogue: TheDialogue) -> StdResult<(), ()>
{
    log::info!(" [:: LOG ::]     @[fn]:[controllers::handle_callback] :: [Started]");

    let err_msg = " [:: LOG ::]   (@[fn]:[controllers::handle_callback] error : 'No 'chat_id' in CallbackQuery') ";
    let log_err = || log::error!("{err_msg}");
    let chat_id = callback.chat_id().ok_or_else(log_err)?;

    let sendable = crate::model::handlers::callback::handle_callback(callback.clone(), dialogue.clone()).await;
    update_view(&bot, chat_id, sendable, dialogue, callback.into()).await;
    Ok(())
}


