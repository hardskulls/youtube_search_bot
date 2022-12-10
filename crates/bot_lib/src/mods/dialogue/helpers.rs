use teloxide::
{
    Bot,
    requests::Requester,
    types::{CallbackQuery, ChatId, InlineKeyboardMarkup, Message},
};
use teloxide::payloads::EditMessageTextSetters;

use crate::mods::
{
    dialogue::types::{DialogueData, State, ListConfigData, SearchConfigData, TheDialogue},
    errors::{DialogueStateStorageError, NoCallbackDataError, NoTextError},
};
use crate::mods::dialogue::types::MessageWithKB;
use crate::mods::errors::NoMessageWithKB;

type StdResult<T, E> = Result<T, E>;

pub(crate) async fn edit_keyboard<S: Into<String>>(bot: &Bot, text: S, inline_keyboard: InlineKeyboardMarkup, msg_with_kb: &MessageWithKB)
    -> eyre::Result<()>
{
    let text = text.into();
    // log::info!("[:: LOG ::]  :  [:: 'edit_keyboard' executed ::]");
    // log::info!("[:: LOG ::]  :  [:: text: {:?} | inline_keyboard: {:?} | msg_with_kb: {:?} ::]", text, inline_keyboard, msg_with_kb.opt_message.is_some());
    let msg = msg_with_kb.opt_message.as_ref().ok_or(NoMessageWithKB)?;
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(inline_keyboard)
        .await?;
    Ok(())
}

pub(crate) fn search_config_update_or_default(d_state: State) -> SearchConfigData
{
    if let State::SearchCommandActive(search_config) = d_state
    { search_config }
    else
    { SearchConfigData::default() }
}

pub(crate) fn list_config_update_or_default(d_state: State) -> ListConfigData
{
    if let State::ListCommandActive(list_config) = d_state
    { list_config }
    else
    { ListConfigData::default() }
}

/// Update state if state is `Some()` and send a message to user.
/*
#[inline]
pub(crate) async fn update_state_and_send_message<S: Into<String> + Send>
(
    opt_dialogue: Option<TheDialogue>,
    opt_dialogue_data: Option<DialogueData>,
    opt_keyboard: Option<InlineKeyboardMarkup>,
    bot: Bot,
    chat_id: ChatId,
    text_to_send: S
)
    -> eyre::Result<()>
{
    if let (Some(dialogue), Some(dialogue_data)) = (opt_dialogue, opt_dialogue_data)
    { dialogue.update(dialogue_data).await.map_err(|e| eyre::anyhow!(e))?; }

    let mut send_msg = bot.send_message(chat_id, text_to_send);
    if let Some(kb) = opt_keyboard { send_msg = send_msg.reply_markup(kb) }
    send_msg.await?;
    Ok(())
}*/

pub(crate) async fn update_optionally_and_send_message<S: Into<String> + Send>
(
    opt_dialogue: Option<TheDialogue>,
    opt_dialogue_data: Option<DialogueData>,
    opt_keyboard: Option<InlineKeyboardMarkup>,
    bot: Bot,
    chat_id: ChatId,
    text_to_send: S
)
    -> eyre::Result<()>
{
    let msg_text = text_to_send.into();
    // log::info!("[:: LOG ::]  :  [:: 'update_optionally_and_send_message' executed ::]");
    // let print = (&opt_dialogue_data.is_some(), &opt_keyboard.is_some(), &msg_text);
    // log::info!("[:: LOG DATA ::]  :  [:: INPUT of 'update_optionally_and_send_message' is: {:#?} ::]", print);
    match (opt_keyboard, opt_dialogue, opt_dialogue_data)
    {
        (
            Some(kb),
            Some(dialogue),
            Some(DialogueData { message_with_kb: msg_with_kb, state, last_callback })
        )
            =>
            {
                // log::info!("[:: LOG ::]  :  [:: 'update_optionally_and_send_message' executed, editing keyboard | <path 1/3 > ::]");
                edit_keyboard(&bot, msg_text, kb, &msg_with_kb).await?;
                dialogue.update(DialogueData { message_with_kb: msg_with_kb, state, last_callback })
                    .await
                    .map_err(|e| eyre::anyhow!(e))?;
            }
        (None, Some(dialogue), Some(d_data)) =>
            {
                // log::info!("[:: LOG ::]  :  [:: 'update_optionally_and_send_message' executed, updating dialogue and sending message | < path 2/3 > ::]");
                dialogue.update(d_data)
                    .await
                    .map_err(|e| eyre::anyhow!(e))?;
                bot.send_message(chat_id, msg_text).await?;
            }
        _ =>
            {
                // log::info!("[:: LOG ::]  :  [:: 'update_optionally_and_send_message' executed, just sending message | < path 3/3 > ::]");
                bot.send_message(chat_id, msg_text).await?;
            }
    }
    Ok(())
}

/// Get state from dialogue.
#[inline]
pub(crate) async fn get_dialogue_data(dialogue: &TheDialogue) -> Result<DialogueData, DialogueStateStorageError>
{
    dialogue.get()
        .await
        .map_err(|_| DialogueStateStorageError)?
        .ok_or(DialogueStateStorageError)
}

/// Get text from message.
#[inline]
pub(crate) async fn get_text(msg: &Message) -> StdResult<&str, NoTextError>
{
    msg.text()
        .ok_or(NoTextError)
}

/// Get text from message.
#[inline]
pub(crate) async fn get_callback_data(callback: &CallbackQuery) -> StdResult<String, NoCallbackDataError>
{
    callback.data.to_owned()
        .ok_or(NoCallbackDataError)
}


