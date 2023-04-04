
use teloxide::Bot;
use teloxide::prelude::Message;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;

use crate::model::commands::types::Command;
use crate::model::dialogue::types::TheDialogue;
use crate::model::youtube::types::SearchableItem;
use crate::StdResult;
use crate::view::funcs::update_view;
use crate::view::types::Sendable;

#[inline]
pub fn is_other_command<B : BotCommands>(msg : Message, me : Me) -> bool
{
    let bot_name = me.username();
    if let Some(text) = msg.text()
    { matches!(text.chars().next(), Some('/')) && B::parse(text, bot_name).is_err() }
    else
    { false }
}

/// Tell user that an unknown command was received.
#[inline]
pub async fn handle_unknown_command(bot : Bot, msg : Message, dialogue : TheDialogue) -> StdResult<(), ()>
{
    let chat_id = msg.chat.id;
    update_view(&bot, chat_id, Sendable::SendError("Unknown command ❌"), dialogue).await;
    Ok(())
}

pub async fn handle_commands(bot : Bot, msg : Message, dialogue : TheDialogue, cmd : Command) -> StdResult<(), ()>
{
    let chat_id = msg.chat.id;
    let sendable =
        crate::model::handlers::commands::handle_commands::<SearchableItem>(msg, dialogue.clone(), cmd).await;
    update_view(&bot, chat_id, sendable, dialogue).await;
    Ok(())
}


