use teloxide::prelude::Message;
use teloxide::Bot;

use crate::model::dialogue::types::TheDialogue;
use crate::view::funcs::update_view;
use crate::StdRes;

pub async fn handle_text(bot: Bot, msg: Message, dialogue: TheDialogue) -> StdRes<(), ()> {
    log::info!(" [:: LOG ::]     @[fn]:[controllers::handle_text] :: [Started]");
    let chat_id = msg.chat.id;
    let sendable = crate::model::handlers::text::handle_text(msg, dialogue.clone()).await;
    update_view(&bot, chat_id, sendable, dialogue, None).await;
    Ok(())
}
