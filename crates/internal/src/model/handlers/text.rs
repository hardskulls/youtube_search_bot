
use error_traits::{PassErrWith, MapErrBy, WrapInRes};

use teloxide::types::Message;

use crate::model::dialogue::funcs::{get_callback_data, get_dialogue_data, get_text, parse_number, save_text};
use crate::model::dialogue::types::{DialogueData, Either, State, TheDialogue};
use crate::model::dialogue::types::State::{ListCommandActive, SearchCommandActive};
use crate::model::keyboards::types::{Buttons, ListCommandButtons, SearchCommandButtons};
use crate::StdResult;
use crate::view::types::Sendable;


pub(crate) async fn get_required_text_state(msg: Message, dialogue: TheDialogue)
    -> StdResult<(String, DialogueData, Buttons), String>
{
    log::info!(" [:: LOG ::]     @[fn]:[get_required_text_state] :: [Started]");
    
    let log_prefix = " [:: LOG ::]    | @:[fn::send_message] error: ";
    let user_error = || "⚠ Internal error ⚠";
    
    let dialogue_data = get_dialogue_data(&dialogue).await.pass_err_with(|e| log::error!("{log_prefix}{e}")).map_err_by(user_error)?;
    if dialogue_data.last_callback.as_ref().is_none()
    { return "Bot is running! 🚀 \nSend /start command to start a game 🕹".to_owned().in_err() }
    
    let callback = dialogue_data.last_callback.as_ref().ok_or_else(user_error).pass_err_with(|e| log::error!("{log_prefix}{e}"))?;
    let callback_data = get_callback_data(callback).await.pass_err_with(|e| log::error!("{log_prefix}{e}")).map_err_by(user_error)?;
    let keyboard: Buttons = serde_json::from_str(&callback_data).pass_err_with(|e| log::error!("{log_prefix}{e}")).map_err_by(user_error)?;
    let text = get_text(&msg).await.pass_err_with(|e| log::error!("{log_prefix}{e}")).map_err_by(user_error)?;
    
    (text.into(), dialogue_data, keyboard).in_ok()
}

pub(crate) async fn handle_text(msg: Message, dialogue: TheDialogue)
    -> Sendable<String>
{
    log::info!(" [:: LOG ::]     @[fn]:[handlers::handle_text] :: [Started]");
    let (text, d_data, buttons): (String, DialogueData, Buttons) =
        match get_required_text_state(msg, dialogue).await
        {
            Ok(ok) => ok,
            Err(e) => return Sendable::SendError(e)
        };
    
    let (message_text, opt_dialogue_data): (&str, Option<DialogueData>) =
        match (d_data.state.as_ref(), buttons)
        {
            (State::Starting, ..) => ("Bot is running! 🚀 \nSend /start command to start a game 🕹", None),
            (SearchCommandActive(search_config), Buttons::SearchButtons(SearchCommandButtons::ResultLimit)) =>
                parse_number(&text, Either::First(search_config), &d_data),
            (ListCommandActive(list_config), Buttons::ListButtons(ListCommandButtons::ResultLimit)) =>
                parse_number(&text, Either::Last(list_config), &d_data),
            (SearchCommandActive(search_config), Buttons::SearchButtons(SearchCommandButtons::TextToSearch)) =>
                save_text(&text, (search_config).clone(), &d_data),
            other =>
                {
                    log::info!(" [:: LOG ::] ... ( @[fn]:[handle_text] [:: {:?} ::] )", other);
                    ("Oops! 🤷‍♂️", None)
                }
        };
    Sendable::SendOrEditMessage(message_text.into(), None, opt_dialogue_data)
}


