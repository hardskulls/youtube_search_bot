
use error_traits::LogErr;

use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::types::{ChatId, InlineKeyboardMarkup, Message};

use crate::view::types::Sendable;
use crate::model::dialogue::types::{DialogueData, MessageWithKB, TheDialogue};
use crate::model::utils::HTMLise;
use crate::model::youtube::types::SearchableItem;

pub(crate) async fn update_view<S>
(
    bot : &Bot,
    send_to : ChatId,
    sendable : Sendable<SearchableItem, S>,
    dialogue : TheDialogue,
)
    where
        S : Into<String>
{
    match sendable
    {
        Sendable::SendError(text) => send_user_error(bot, send_to, text).await,
        Sendable::SendOrEditMessage(text, opt_msg, opd_dialogue_data) =>
            send_or_edit_message(bot, send_to, text, opt_msg, dialogue.into(), opd_dialogue_data).await,
        Sendable::SendKeyboard { text, kb, save_msg_id, d_data } =>
            send_keyboard(bot, send_to, text, kb, save_msg_id, d_data, dialogue).await,
        Sendable::EditKeyboard(opt_text, kb, msg, opd_dialogue_data) =>
            edit_message_and_keyboard(bot, send_to, opt_text, kb, msg, dialogue.into(), opd_dialogue_data).await,
        Sendable::SendResults { prefix, postfix, values } =>
            send_results(bot, prefix, postfix, send_to, values).await,
    }
}

async fn send_user_error(bot : &Bot, send_to : ChatId, text : impl Into<String>)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_user_error] error: ";
    let _ = bot.send_message(send_to, text).await.log_err(log_prefix);
}

async fn send_or_edit_message
(
    bot : &Bot, 
    send_to : ChatId, 
    text : impl Into<String>, 
    opt_msg : Option<Message>,
    opt_dialogue : Option<TheDialogue>,
    opt_d_data : Option<DialogueData>
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_or_edit_message] error: ";
    if let Some(msg) = opt_msg
    { let _ = bot.edit_message_text(send_to, msg.id, text).await.log_err(log_prefix); }
    else
    { let _ = bot.send_message(send_to, text).await.log_err(log_prefix); }
    
    if let (Some(dialogue), Some(d_data)) = (opt_dialogue, opt_d_data)
    { let _ = dialogue.update(d_data).await.log_err(log_prefix); }
}

async fn send_keyboard
(
    bot : &Bot,
    send_to : ChatId,
    text : impl Into<String>,
    kb : InlineKeyboardMarkup,
    pin_msg : bool,
    opt_dialogue_data : Option<DialogueData>,
    dialogue : TheDialogue
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_keyboard] error: ";
    let res = bot.send_message(send_to, text).reply_markup(kb).await.log_err(log_prefix);
    match (res, opt_dialogue_data)
    {
        (Ok(sent_msg), Some(d_data)) =>
            {
                if pin_msg
                {
                    let new_dialogue_data = DialogueData { message_with_kb : MessageWithKB { opt_message : sent_msg.into() }, ..d_data };
                    let _ = dialogue.update(new_dialogue_data).await.log_err(log_prefix);
                }
            }
        (_, Some(d_data)) => { let _ = dialogue.update(d_data).await.log_err(log_prefix); }
        _ => {}
    };
}

async fn edit_message_and_keyboard
(
    bot : &Bot, 
    send_to : ChatId, 
    opt_text : Option<impl Into<String>>, 
    kb : InlineKeyboardMarkup, 
    msg : Message,
    opt_dialogue : Option<TheDialogue>,
    opt_d_data : Option<DialogueData>
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::edit_message_and_keyboard] error: ";
    if let Some(text) = opt_text
    { let _ = bot.edit_message_text(send_to, msg.id, text).reply_markup(kb).await.log_err(log_prefix); }
    else
    { let _ = bot.edit_message_reply_markup(send_to, msg.id).await.log_err(log_prefix); }
    
    if let (Some(dialogue), Some(d_data)) = (opt_dialogue, opt_d_data)
    { let _ = dialogue.update(d_data).await.log_err(log_prefix); }
}

async fn send_results
(
    bot : &Bot,
    prefix : Option<impl Into<String>>,
    postfix : Option<impl Into<String>>,
    send_to : ChatId,
    values : Vec<SearchableItem>,
)
{
    let log_prefix = " [:: LOG ::]    | @:[fn::send_results] error: ";
    let formatting =
        |i : SearchableItem|
            {
                let title = i.title.as_deref().unwrap_or("No title 🤷‍♂️");
                let descr = i.description.as_deref().unwrap_or("No description 🤷‍♂️");
                let link = i.link.as_deref().unwrap_or("No link 🤷‍♂️");
                format!("{}{}{}", title.to_bold() + " \n\n", descr.to_owned() + " \n\n", link)
            };
    
    if let Some(p) = prefix
    { let _ = bot.send_message(send_to, p).await.log_err(log_prefix); }
    
    for v in values
    {
        let text = formatting(v);
        let _ = bot.send_message(send_to, text).await;
    }
    
    if let Some(p) = postfix
    { let _ = bot.send_message(send_to, p).await.log_err(log_prefix); }
}


