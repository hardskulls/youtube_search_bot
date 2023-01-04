use teloxide::Bot;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::requests::Requester;
use teloxide::types::{CallbackQuery, ChatId, InlineKeyboardMarkup, Message};

use crate::mods::dialogue::types::{DialogueData, ListConfigData, MessageWithKB, SearchConfigData, State, TheDialogue};
use crate::mods::errors::{DialogueStateStorageError, NoCallbackDataError, NoTextError};
use crate::mods::errors::NoMessageWithKB;
use crate::StdResult;

/// Edits `InlineKeyboard` in a certain message.
pub(crate) async fn edit_keyboard<S: Into<String>>(bot: &Bot, text: S, inline_keyboard: InlineKeyboardMarkup, msg_with_kb: &MessageWithKB)
    -> eyre::Result<()>
{
    let text = text.into();
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

/// Used in the end of main handlers.
/// Updates `dialogue` state when possible, and sends message.  
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
    match (opt_keyboard, opt_dialogue, opt_dialogue_data)
    {
        (Some(kb), Some(dialogue), Some(d_data)) =>
            {
                edit_keyboard(&bot, text_to_send, kb, &d_data.message_with_kb).await?;
                dialogue.update(d_data).await.map_err(|e| eyre::anyhow!(e))?;
            }
        (None, Some(dialogue), Some(d_data)) =>
            {
                bot.send_message(chat_id, text_to_send).await?;
                dialogue.update(d_data).await.map_err(|e| eyre::anyhow!(e))?;
            }
        _ => { bot.send_message(chat_id, text_to_send).await?; }
    }
    Ok(())
}

/// Get `state` from dialogue.
#[inline]
pub(crate) async fn get_dialogue_data(dialogue: &TheDialogue) -> Result<DialogueData, DialogueStateStorageError>
{
    dialogue.get()
        .await
        .map_err(|_| DialogueStateStorageError)?
        .ok_or(DialogueStateStorageError)
}

/// Get `text` from message.
#[inline]
pub(crate) async fn get_text(msg: &Message) -> StdResult<&str, NoTextError>
{
    msg.text()
        .ok_or(NoTextError)
}

/// Get `callback` data as a `String`.
#[inline]
pub(crate) async fn get_callback_data(callback: &CallbackQuery) -> StdResult<String, NoCallbackDataError>
{
    callback.data.to_owned()
        .ok_or(NoCallbackDataError)
}


