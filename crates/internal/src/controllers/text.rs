
use teloxide::Bot;
use teloxide::prelude::Message;

use crate::model::dialogue::types::TheDialogue;
use crate::model::youtube::types::SearchableItem;
use crate::StdResult;
use crate::view::funcs::update_view;


pub async fn handle_text(bot : Bot, msg : Message, dialogue : TheDialogue) -> StdResult<(), ()>
{
    log::info!(" [:: LOG ::]     @[fn]:[controllers::handle_text] :: [Started]");
    let chat_id = msg.chat.id;
    let sendable =
        crate::model::handlers::text::handle_text::<SearchableItem>(msg, dialogue.clone()).await;
    update_view(&bot, chat_id, sendable, dialogue).await;
    Ok(())
}


