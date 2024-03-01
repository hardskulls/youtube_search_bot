use teloxide::prelude::Message;
use teloxide::types::Me;
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

use crate::model::commands::types::Command;
use crate::model::dialogue::types::TheDialogue;
use crate::view::funcs::update_view;
use crate::view::types::Sendable;
use crate::StdRes;

#[inline]
pub fn is_other_command<B: BotCommands>(msg: Message, me: Me) -> bool {
    log::info!(" [:: LOG ::]     @[fn]:[controllers::is_other_command] :: [Started]");

    let bot_name = me.username();
    let is_not_that_cmd = |t, b_name| B::parse(t, b_name).is_err();
    msg.text()
        .is_some_and(|t| t.starts_with('/') && is_not_that_cmd(t, bot_name))
}

/// Tell user that an unknown command was received.
#[inline]
pub async fn handle_unknown_command(
    bot: Bot,
    msg: Message,
    dialogue: TheDialogue,
) -> StdRes<(), ()> {
    log::info!(" [:: LOG ::]     @[fn]:[controllers::handle_unknown_command] :: [Started]");

    update_view(
        &bot,
        msg.chat.id,
        Sendable::SendError("Unknown command ❌"),
        dialogue,
        None,
    )
    .await;
    Ok(())
}

pub async fn handle_commands(
    bot: Bot,
    msg: Message,
    dialogue: TheDialogue,
    cmd: Command,
) -> StdRes<(), ()> {
    log::info!(" [:: LOG ::]     @[fn]:[controllers::handle_commands] :: [Started]");

    let chat_id = msg.chat.id;
    let sendable =
        crate::model::handlers::commands::handle_commands(msg, dialogue.clone(), cmd).await;

    update_view(&bot, chat_id, sendable, dialogue, None).await;
    Ok(())
}
