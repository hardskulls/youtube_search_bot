use crate::model::dialogue::types::{CommandConfig, DialogueData};
use teloxide::types::{InlineKeyboardMarkup, Message};

#[derive(Debug, Clone)]
pub(crate) enum Sendable<S>
where
    S: Into<String>,
{
    SendError(S),
    SendOrEditMessage(S, Option<Message>, Option<DialogueData>),
    SendKeyboard {
        text: S,
        kb: InlineKeyboardMarkup,
        save_msg_id: bool,
        d_data: Option<DialogueData>,
    },
    EditKeyboard(
        Option<S>,
        InlineKeyboardMarkup,
        Message,
        Option<DialogueData>,
    ),
    ExecuteCommand(CommandConfig),
}
