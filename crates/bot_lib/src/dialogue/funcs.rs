use teloxide::Bot;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::requests::Requester;
use teloxide::types::{CallbackQuery, ChatId, InlineKeyboardMarkup, Message, ParseMode};
use error_traits::WrapInOk;

use crate::dialogue::types::{DialogueData, ListCommandSettings, MessageWithKB, SearchCommandSettings, State, TheDialogue};
use crate::errors::{DialogueStateStorageError, NoCallbackDataError, NoMessageWithKB, NoTextError};
use crate::StdResult;

/// Edits `InlineKeyboard` in a certain message.
pub(crate) async fn edit_keyboard<S: Into<String>>(bot: &Bot, text: S, inline_keyboard: InlineKeyboardMarkup, msg_with_kb: &MessageWithKB)
    -> eyre::Result<()>
{
    let text = text.into();
    let msg = msg_with_kb.opt_message.as_ref().ok_or(NoMessageWithKB)?;
    bot.edit_message_text(msg.chat.id, msg.id, text)
        .reply_markup(inline_keyboard)
        .parse_mode(ParseMode::Html)
        .await?;
    ().in_ok()
}

pub(crate) fn search_settings_update_or_default(d_state: State) -> SearchCommandSettings
{
    if let State::SearchCommandActive(search_settings) = d_state
    { search_settings }
    else
    { SearchCommandSettings::default() }
}

pub(crate) fn list_settings_update_or_default(d_state: State) -> ListCommandSettings
{
    if let State::ListCommandActive(list_settings) = d_state
    { list_settings }
    else
    { ListCommandSettings::default() }
}

/// Used in the end of main handlers (text, callback and commands).
/// Updates `dialogue` state when possible, and sends message.  
pub(crate) async fn update_optionally_and_send_message
(
    opt_dialogue: Option<TheDialogue>,
    opt_d_data: Option<DialogueData>,
    opt_kb: Option<InlineKeyboardMarkup>,
    bot: Bot,
    chat_id: ChatId,
    text: impl Into<String> + Send
)
    -> eyre::Result<()>
{
    match (opt_kb, opt_dialogue, opt_d_data)
    {
        (Some(kb), Some(dialogue), Some(d_data)) =>
            {
                edit_keyboard(&bot, text, kb, &d_data.message_with_kb).await?;
                dialogue.update(d_data).await.map_err(|e| eyre::anyhow!(e))?;
            }
        (None, Some(dialogue), Some(d_data)) =>
            {
                bot.send_message(chat_id, text).parse_mode(ParseMode::Html).await?;
                dialogue.update(d_data).await.map_err(|e| eyre::anyhow!(e))?;
            }
        _ => { bot.send_message(chat_id, text).await?; }
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
    callback.data
        .clone()
        .ok_or(NoCallbackDataError)
}


