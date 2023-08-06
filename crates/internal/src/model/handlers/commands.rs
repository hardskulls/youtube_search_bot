
use error_traits::{MapErrBy, MergeOkErr, PassErrWith};
use teloxide::prelude::Message;

use crate::model::commands::funcs::{info, log_out};
use crate::model::commands::types::Command;
use crate::model::dialogue::types::{DialogueData, ListCommandSettings, MessageTriplet, SearchCommandSettings, SearchVideosInPlaylistsCommandSettings, State, TheDialogue};
use crate::model::keyboards::traits::{CreateKB, KeyboardText};
use crate::model::keyboards::types::{ListCommandButtons, SearchCommandButtons, SearchVideoInPlaylistsCommandButtons};
use crate::view::types::Sendable;

pub(crate) async fn handle_commands(msg: Message, dialogue: TheDialogue, cmd: Command)
    -> Sendable<impl Into<String>>
{
    log::info!(" [:: LOG ::]     @[fn]:[handlers::handle_commands] :: [Started]");

    let log_prefix = " [:: LOG ::]  :  @fn:[commands::common::log_out]  ->  error: ";
    let err = || ("Couldn't log out ❌".to_owned(), None, None);

    let (message_text, opt_keyboard, opt_dialogue_data): MessageTriplet =
        match cmd
        {
            Command::Start => ("Bot started, send something ⌨ \n Use /search or /list commands 🚀".into(), None, None),
            Command::Info => info(&dialogue).await.merge_ok_err(),
            Command::Search =>
                {
                    let state = State::SearchCommandActive(SearchCommandSettings::default());
                    let d_data = DialogueData { state, ..Default::default() };
                    (SearchCommandButtons::ButtonList.kb_text(), SearchCommandButtons::ButtonList.create_kb(), d_data.into())
                }
            Command::List =>
                {
                    let state = State::ListCommandActive(ListCommandSettings::default());
                    let d_data = DialogueData { state, ..Default::default() };
                    (ListCommandButtons::ButtonList.kb_text(), ListCommandButtons::ButtonList.create_kb(), d_data.into())
                }
            Command::LogOut =>
                {
                    let Some(user_id) = msg.from() else { return Sendable::SendError("⚠ Internal error ⚠".to_owned()) };
                    let user_id = user_id.id.to_string();
                    log_out(&user_id, env!("REDIS_YOUTUBE_ACCESS_TOKEN_STORAGE")).await
                        .pass_err_with(|e| log::error!("{log_prefix}{e}"))
                        .map_err_by(err)
                        .merge_ok_err()
                }
            Command::SearchVideosInYourPlaylists =>
                {
                    let state = State::SearchVideosInPlaylistsCommandActive(SearchVideosInPlaylistsCommandSettings::default());
                    let d_data = DialogueData { state, ..Default::default() };
                    let buttons = SearchVideoInPlaylistsCommandButtons::ButtonList;
                    (buttons.kb_text(), buttons.create_kb(), d_data.into())
                }
        };
    if let (d, Some(kb)) = (opt_dialogue_data, opt_keyboard)
    { Sendable::SendKeyboard { text: message_text, kb, save_msg_id: true, d_data: d } }
    else
    { Sendable::SendOrEditMessage(message_text, None, None) }
}


