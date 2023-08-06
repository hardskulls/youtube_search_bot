
use error_traits::{LogErr, ToEmpty};
use teloxide::Bot;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::requests::Requester;
use teloxide::types::{ChatId, InlineKeyboardMarkup, Message, CallbackQuery};
use teloxide::types::ParseMode::Html;

use crate::model::dialogue::types::{DialogueData, MessageWithKB, TheDialogue, CommandConfig};
use crate::model::handlers::callback::list_cmd::execute_list_command;
use crate::model::handlers::callback::search_cmd::execute_search_command;
use crate::model::handlers::callback::search_videos_in_playlits::execute_search_videos_in_playlists_command;
use crate::model::utils::HTMLise;
use crate::model::youtube::types::SearchableItem;
use crate::view::funcs::shorthands::{send_message, update_dialogue};
use crate::view::types::Sendable;


#[allow(clippy::unwrap_used)]
pub(crate) async fn update_view<S>
(
    bot: &Bot,
    send_to: ChatId,
    sendable: Sendable<S>,
    dialogue: TheDialogue,
    opt_callback: Option<CallbackQuery>
)
    where
        S: Into<String>
{
    match sendable
    {
        Sendable::SendError(text) =>
            send_user_error(bot, send_to, text).await,
        Sendable::SendOrEditMessage(text, opt_msg, opd_dialogue_data) =>
            send_or_edit_message(bot, send_to, text, opt_msg, dialogue.into(), opd_dialogue_data).await,
        Sendable::SendKeyboard { text, kb, save_msg_id, d_data } =>
            send_keyboard(bot, send_to, text, kb, save_msg_id, d_data, dialogue).await,
        Sendable::EditKeyboard(opt_text, kb, msg, opd_dialogue_data) =>
            edit_message_and_keyboard(bot, send_to, opt_text, kb, msg, dialogue.into(), opd_dialogue_data).await,
        Sendable::ExecuteCommand(command_config) =>
            execute_command(bot, send_to, opt_callback.unwrap(), command_config).await
    }
}

async fn send_user_error(bot: &Bot, send_to: ChatId, text: impl Into<String>)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_user_error] error: ";
    send_message(bot, send_to, text, log_prefix).await
}

async fn send_or_edit_message
(
    bot: &Bot,
    send_to: ChatId,
    text: impl Into<String>,
    opt_msg: Option<Message>,
    opt_dialogue: Option<TheDialogue>,
    opt_d_data: Option<DialogueData>
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_or_edit_message] error: ";
    if let Some(msg) = opt_msg
    {
        bot.edit_message_text(send_to, msg.id, text)
            .parse_mode(Html)
            .await
            .log_err(log_prefix)
            .to_empty()
    }
    else
    { send_message(bot, send_to, text, log_prefix).await }

    if let (Some(dialogue), Some(d_data)) = (opt_dialogue, opt_d_data)
    { update_dialogue(dialogue, d_data, log_prefix).await }
}

async fn send_keyboard
(
    bot: &Bot,
    send_to: ChatId,
    text: impl Into<String>,
    kb: InlineKeyboardMarkup,
    pin_msg: bool,
    opt_dialogue_data: Option<DialogueData>,
    dialogue: TheDialogue
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_keyboard] error: ";
    let res =
        bot.send_message(send_to, text)
            .reply_markup(kb)
            .parse_mode(Html)
            .await
            .log_err(log_prefix);
    match (res, opt_dialogue_data)
    {
        (Ok(sent_msg), Some(d_data)) if pin_msg =>
            {
                let message_with_kb = MessageWithKB { opt_message: sent_msg.into() };
                let new_dialogue_data = DialogueData { message_with_kb, ..d_data };
                update_dialogue(dialogue, new_dialogue_data, log_prefix).await;
            }
        (_, Some(d_data)) => update_dialogue(dialogue, d_data, log_prefix).await,
        _ => {}
    };
}

async fn edit_message_and_keyboard
(
    bot: &Bot,
    send_to: ChatId,
    opt_text: Option<impl Into<String>>,
    kb: InlineKeyboardMarkup,
    msg: Message,
    opt_dialogue: Option<TheDialogue>,
    opt_d_data: Option<DialogueData>
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::edit_message_and_keyboard] error: ";
    if let Some(text) = opt_text
    {
        bot.edit_message_text(send_to, msg.id, text)
            .reply_markup(kb)
            .parse_mode(Html)
            .await
            .log_err(log_prefix)
            .to_empty()
    }
    else
    {
        bot.edit_message_reply_markup(send_to, msg.id)
            .await
            .log_err(log_prefix)
            .to_empty()
    }

    if let (Some(dialogue), Some(d_data)) = (opt_dialogue, opt_d_data)
    { update_dialogue(dialogue, d_data, log_prefix).await }
}

async fn send_results
(
    bot: &Bot,
    prefix: Option<impl Into<String>>,
    postfix: Option<impl Into<String>>,
    send_to: ChatId,
    values: Vec<SearchableItem>,
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_results] error: ";
    let formatting =
        |i: SearchableItem|
            {
                let title = i.title.as_deref().unwrap_or("No title 🤷‍♂️");
                let descr = i.description.as_deref().unwrap_or("No description 🤷‍♂️");
                let link = i.link.as_deref().unwrap_or("No link 🤷‍♂️");
                format!("{}{}{}", title.to_bold() + " \n\n", descr.to_owned() + " \n\n", link)
            };

    if let Some(p) = prefix
    { send_message(bot, send_to, p, log_prefix).await; }

    for v in values
    {
        let text = formatting(v);
        send_message(bot, send_to, text, log_prefix).await;
    }

    if let Some(p) = postfix
    { send_message(bot, send_to, p, log_prefix).await }
}

async fn execute_command(bot: &Bot, send_to: ChatId, callback: CallbackQuery, config: CommandConfig)
{
    let log_prefix = " [:: LOG ::]   @:[fn::execute_command] error: ";
    
    send_message(bot, send_to, "Searching, please wait 🕵️‍♂️", log_prefix).await;
    let results =
        match config
        {
            CommandConfig::SearchConfig(s) => execute_search_command(s, callback).await,
            CommandConfig::ListConfig(l) => execute_list_command(l, callback).await,
            CommandConfig::SearchVideosInPlaylistsConfig(sv) =>
                execute_search_videos_in_playlists_command(sv, callback).await
        };
    match results
    {
        Err(e) => send_message(bot, send_to, e, log_prefix).await,
        Ok((prefix, results, postfix)) => send_results(bot, prefix, postfix, send_to, results).await
    };
}

mod shorthands
{
    use super::*;

    pub(super) async fn update_dialogue(dialogue: TheDialogue, d_data: DialogueData, log_prefix: &str)
    {
        dialogue.update(d_data)
            .await
            .log_err(log_prefix)
            .to_empty()
    }

    pub(super) async fn send_message(bot: &Bot, send_to: ChatId, text: impl Into<String>, log_prefix: &str)
    {
        bot.send_message(send_to, text)
            .parse_mode(Html)
            .await
            .log_err(log_prefix)
            .to_empty()
    }
}


