
use teloxide::types::{InlineKeyboardMarkup, Message};
use crate::model::dialogue::types::DialogueData;

#[derive(Debug, Clone)]
pub(crate) enum Sendable<T, S>
    where
        S : Into<String>
{
    SendError(S),
    SendOrEditMessage(S, Option<Message>, Option<DialogueData>),
    SendKeyboard { text : S, kb : InlineKeyboardMarkup, save_msg_id : bool, d_data : Option<DialogueData> },
    EditKeyboard(Option<S>, InlineKeyboardMarkup, Message, Option<DialogueData>),
    SendResults 
    {
        prefix : Option<S>,
        postfix : Option<S>,
        values : Vec<T>
    }
}


