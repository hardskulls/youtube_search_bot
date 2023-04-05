
use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;

use teloxide::types::CallbackQuery;
use crate::model::dialogue::types::TheDialogue;
use crate::StdResult;

use crate::view::funcs::update_view;

pub async fn handle_callback(bot : Bot, callback : CallbackQuery, dialogue : TheDialogue) -> StdResult<(), ()>
{
    log::info!(" [:: LOG ::]     @[fn]:[controllers::handle_callback] :: [Started]");
    let Some(chat_id) =
        callback.chat_id()
        else
        {
            log::error!(" [:: LOG ::]   (@[fn]:[controllers::handle_callback] error : 'No 'chat_id' in CallbackQuery') ");
            return Err(());
        };
    let sendable = crate::model::handlers::callback::handle_callback(callback, dialogue.clone()).await;
    update_view(&bot, chat_id, sendable, dialogue).await;
    Ok(())
}


