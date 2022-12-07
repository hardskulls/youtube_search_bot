use teloxide::
{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, ChatId, InlineKeyboardMarkup, Message},
};

use crate::mods::
{
    dialogue::types::{DialogueData, State, ListConfigData, SearchConfigData, TheDialogue},
    errors::{DialogueStateStorageError, NoCallbackDataError, NoTextError},
};

pub(crate) async fn edit_keyboard<S: Into<String>>(bot: &Bot, text: S, opt_keyboard: Option<InlineKeyboardMarkup>, callback: &CallbackQuery)
    -> Result<(), teloxide::RequestError>
{
    let mut edit_message = bot.edit_message_text_inline(callback.id.clone(), text);
    edit_message.reply_markup = opt_keyboard;
    edit_message.await?;
    Ok(())
}

pub(crate) fn search_config_update_or_default(d_state: State) -> SearchConfigData
{
    if let State::SearchCommandActive(search_config) = d_state
    { search_config }
    else
    { Default::default() }
}

pub(crate) fn list_config_update_or_default(d_state: State) -> ListConfigData
{
    if let State::ListCommandActive(list_config) = d_state
    { list_config }
    else
    { Default::default() }
}

/// Update state if state is `Some()` and send a message to user.
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
pub(crate) async fn get_text(msg: &Message) -> std::result::Result<&str, NoTextError>
{
    msg.text()
        .ok_or(NoTextError)
}

/// Get text from message.
#[inline]
pub(crate) async fn get_callback_data(callback: &CallbackQuery) -> std::result::Result<String, NoCallbackDataError>
{
    callback.data.to_owned()
        .ok_or(NoCallbackDataError)
}


