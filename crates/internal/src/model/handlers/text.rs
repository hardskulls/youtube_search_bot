
use error_traits::{PassErrWith, WrapInRes, MapType};

use teloxide::types::Message;
use CommandSettings::{ListSettings, SearchSettings, SearchVideosInMyPlaylistsSettings};

use crate::model::dialogue::funcs::{get_callback_data, get_dialogue_data, get_text, parse_number, save_text};
use crate::model::dialogue::types::{CommandSettings, DialogueData, Either, State, TheDialogue};
use crate::model::dialogue::types::State::{ListCommandActive, SearchCommandActive, SearchVideosInMyPlaylistsCommandActive};
use crate::model::keyboards::types::{Buttons, ListCommandButtons, SearchCommandButtons, SearchVideosInMyPlaylistsCommandButtons};
use crate::view::types::Sendable;


pub(crate) async fn get_required_text_state(msg: Message, dialogue: TheDialogue)
    -> eyre::Result<Either<(String, DialogueData, Buttons), &'static str>>
{
    log::info!(" [:: LOG ::]     @[fn]:[get_required_text_state] :: [Started]");

    let user_error = || eyre::eyre!("⚠ Internal error ⚠");
    
    let dialogue_data = get_dialogue_data(&dialogue).await?;
    if dialogue_data.last_callback.as_ref().is_none()
    { return "Bot is running! 🚀 \nSend /start command to start a game 🕹".map_type(Either::Last).in_ok() }
    
    let callback = dialogue_data.last_callback.as_ref().ok_or_else(user_error)?;
    let callback_data_as_string = get_callback_data(callback).await?;
    let keyboard = serde_json::from_str::<Buttons>(&callback_data_as_string)?;
    let text = get_text(&msg).await?;
    
    (text.into(), dialogue_data, keyboard).map_type(Either::First).in_ok()
}

pub(crate) async fn handle_text(msg: Message, dialogue: TheDialogue)
    -> Sendable<String>
{
    log::info!(" [:: LOG ::]     @[fn]:[handlers::handle_text] :: [Started]");

    let log_prefix = " [:: LOG ::]    | @:[fn::send_message] error: ";

    let (text, d_data, buttons): (String, DialogueData, Buttons) =
        match get_required_text_state(msg, dialogue).await.pass_err_with(|e| log::error!("{log_prefix}{e}"))
        {
            Ok(ok) =>
                match ok
                {
                    Either::First(f) => f,
                    Either::Last(l) => return Sendable::SendError(l.to_owned())
                },
            Err(_) => return Sendable::SendError("⚠ Internal error ⚠".to_owned())
        };
    
    let (message_text, opt_dialogue_data): (&str, Option<DialogueData>) =
        match (&d_data.state, buttons)
        {
            (State::Starting, ..) => ("Bot is running! 🚀 \nSend /start command to start a game 🕹", None),
            (
                SearchCommandActive(settings),
                Buttons::SearchButtons(SearchCommandButtons::ResultLimit)
            )
            => parse_number(&text, SearchSettings(settings.clone()), &d_data),
            (
                ListCommandActive(settings),
                Buttons::ListButtons(ListCommandButtons::ResultLimit)
            )
            => parse_number(&text, ListSettings(settings.clone()), &d_data),
            (
                SearchVideosInMyPlaylistsCommandActive(settings),
                Buttons::SearchVideosInMyPlaylistsButtons(SearchVideosInMyPlaylistsCommandButtons::ResultLimit)
            )
            => parse_number(&text, SearchVideosInMyPlaylistsSettings(settings.clone()), &d_data),
            (
                SearchCommandActive(settings),
                Buttons::SearchButtons(SearchCommandButtons::TextToSearch)
            )
            => save_text(&text, SearchSettings(settings.clone()), &d_data),
            (
                SearchVideosInMyPlaylistsCommandActive(settings),
                Buttons::SearchVideosInMyPlaylistsButtons(SearchVideosInMyPlaylistsCommandButtons::TextToSearch)
            )
            => save_text(&text, SearchVideosInMyPlaylistsSettings(settings.clone()), &d_data),
            other =>
                {
                    log::error!("@[fn]:[handle_text] Something went wrong, dialogue <state> is: {other:#?} )");
                    ("Oops! 🤷‍♂️", None)
                }
        };
    Sendable::SendOrEditMessage(message_text.into(), None, opt_dialogue_data)
}


