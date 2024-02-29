use error_traits::PassErrWith;
use maptypings::MapType;
use maptypings::WrapInRes;

use teloxide::types::Message;

use crate::model::dialogue::funcs::{
    get_callback_data, get_dialogue_data, get_text, parse_number, save_text,
};
use crate::model::dialogue::types::State::{ListCommandActive, SearchCommandActive};
use crate::model::dialogue::types::{DialogueData, Either, State, TheDialogue};
use crate::model::keyboards::types::{Buttons, ListCommandButtons, SearchCommandButtons};
use crate::view::types::Sendable;

pub(crate) async fn get_required_text_state(
    msg: Message,
    dialogue: TheDialogue,
) -> eyre::Result<Either<(String, DialogueData, Buttons), &'static str>> {
    log::info!(" [:: LOG ::]     @[fn]:[get_required_text_state] :: [Started]");

    let user_error = || eyre::eyre!("⚠ Internal error ⚠");

    let dialogue_data = get_dialogue_data(&dialogue).await?;
    if dialogue_data.last_callback.as_ref().is_none() {
        return "Bot is running! 🚀 \nSend /start command to start a game 🕹"
            .map_type(Either::Last)
            .in_ok();
    }

    let callback = dialogue_data
        .last_callback
        .as_ref()
        .ok_or_else(user_error)?;
    let callback_data_as_string = get_callback_data(callback).await?;
    let keyboard = serde_json::from_str::<Buttons>(&callback_data_as_string)?;
    let text = get_text(&msg).await?;

    (text.into(), dialogue_data, keyboard)
        .map_type(Either::First)
        .in_ok()
}

pub(crate) async fn handle_text(msg: Message, dialogue: TheDialogue) -> Sendable<String> {
    log::info!(" [:: LOG ::]     @[fn]:[handlers::handle_text] :: [Started]");

    let log_prefix = " [:: LOG ::]    | @:[fn::send_message] error: ";

    let (text, d_data, buttons): (String, DialogueData, Buttons) =
        match get_required_text_state(msg, dialogue)
            .await
            .pass_err_with(|e| log::error!("{log_prefix}{e:?}"))
        {
            Ok(ok) => match ok {
                Either::First(f) => f,
                Either::Last(l) => return Sendable::SendError(l.to_owned()),
            },
            Err(_) => return Sendable::SendError("⚠ Internal error ⚠".to_owned()),
        };

    let (message_text, opt_dialogue_data): (&str, Option<DialogueData>) =
        match (d_data.state.as_ref(), buttons) {
            (State::Starting, ..) => (
                "Bot is running! 🚀 \nSend /start command to start a game 🕹",
                None,
            ),
            (
                SearchCommandActive(search_config),
                Buttons::SearchButtons(SearchCommandButtons::ResultLimit),
            ) => parse_number(&text, Either::First(search_config), &d_data),
            (
                ListCommandActive(list_config),
                Buttons::ListButtons(ListCommandButtons::ResultLimit),
            ) => parse_number(&text, Either::Last(list_config), &d_data),
            (
                SearchCommandActive(search_config),
                Buttons::SearchButtons(SearchCommandButtons::TextToSearch),
            ) => save_text(&text, (search_config).clone(), &d_data),
            other => {
                log::info!(
                    " [:: LOG ::] ... ( @[fn]:[handle_text] [:: {:?} ::] )",
                    other
                );
                ("Oops! 🤷‍♂️", None)
            }
        };
    Sendable::SendOrEditMessage(message_text.into(), None, opt_dialogue_data)
}
