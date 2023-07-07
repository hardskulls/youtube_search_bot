
use error_traits::MergeOkErr;
use teloxide::prelude::Message;

use crate::model::commands::funcs::{info, log_out};
use crate::model::commands::types::Command;
use crate::model::dialogue::types::{DialogueData, ListCommandSettings, MessageTriplet, SearchCommandSettings, State, TheDialogue};
use crate::model::keyboards::traits::{CreateKB, KeyboardText};
use crate::model::keyboards::types::ListCommandButtons::ListSettings;
use crate::model::keyboards::types::SearchCommandButtons::SearchSettings;
use crate::view::types::Sendable;

pub(crate) async fn handle_commands(msg: Message, dialogue: TheDialogue, cmd: Command)
    -> Sendable<impl Into<String>>
{
    log::info!(" [:: LOG ::]     @[fn]:[handlers::handle_commands] :: [Started]");
    let (message_text, opt_keyboard, opt_dialogue_data): MessageTriplet =
        match cmd
        {
            Command::Start => ("Bot started, send something ⌨ \n Use /search or /list commands 🚀".into(), None, None),
            Command::Info => info(&dialogue).await.merge_ok_err(),
            Command::Search =>
                {
                    let state = State::SearchCommandActive(SearchCommandSettings::default());
                    (SearchSettings.kb_text(), SearchSettings.create_kb(), DialogueData { state, ..Default::default() }.into())
                }
            Command::List =>
                {
                    let state = State::ListCommandActive(ListCommandSettings::default());
                    (ListSettings.kb_text(), ListSettings.create_kb(), DialogueData { state, ..Default::default() }.into())
                }
            Command::LogOut =>
                {
                    let Some(user_id) = msg.from() else { return Sendable::SendError("⚠ Internal error ⚠".to_owned()) };
                    let user_id = user_id.id.to_string();
                    let db_url = env!("REDIS_URL");
                    log_out(&user_id, db_url).await.merge_ok_err()
                }
        };
    if let (d, Some(kb)) = (opt_dialogue_data, opt_keyboard)
    { Sendable::SendKeyboard { text: message_text, kb, save_msg_id: true, d_data: d } }
    else
    { Sendable::SendOrEditMessage(message_text, None, None) }
}


