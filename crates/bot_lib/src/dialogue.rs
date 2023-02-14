use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::requests::Requester;
use teloxide::types::{CallbackQuery, Message};

use error_traits::MergeOkErr;

use crate::dialogue::callback_handling::{callback_helper_for_list_kb, callback_helper_for_search_kb};
use crate::dialogue::funcs::{get_callback_data, get_dialogue_data, get_text, update_optionally_and_send_message};
use crate::dialogue::text_handling::{parse_number, save_text};
use crate::dialogue::types::{Either, MessageTriplet, State, TheDialogue};
use crate::dialogue::types::State::{ListCommandActive, SearchCommandActive};
use crate::errors::{DialogueStateStorageError, EndpointErrors};
use crate::keyboards::types::{Buttons, ListCommandButtons, SearchCommandButtons};
use crate::keyboards::types::Buttons::{ListButtons, SearchButtons};

pub mod funcs;
pub mod types;
pub(crate) mod text_handling;
pub(crate) mod callback_handling;

/// Main `text` handler.
pub async fn handle_callback_data(bot: Bot, callback: CallbackQuery, dialogue: TheDialogue) -> eyre::Result<()>
{
    let dialogue_data = get_dialogue_data(&dialogue).await?;
    let chat_id = callback.chat_id().ok_or(EndpointErrors::GameError)?;
    let keyboard: Buttons = serde_json::from_str(&get_callback_data(&callback).await?)?;
    let (message_text, opt_keyboard, opt_dialogue_data): MessageTriplet =
        match &keyboard
        {
            SearchButtons(search_kb) => callback_helper_for_search_kb(&bot, search_kb, dialogue_data, callback).await,
            ListButtons(list_kb) => callback_helper_for_list_kb(&bot, list_kb, dialogue_data, callback).await
        }
        .merge_ok_err();
    update_optionally_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, chat_id, message_text).await
}

/// Main `callback` handler.
pub async fn handle_text(bot: Bot, msg: Message, dialogue: TheDialogue) -> eyre::Result<()>
{
    let dialogue_data = get_dialogue_data(&dialogue).await?;
    if dialogue_data.last_callback.as_ref().is_none()
    { bot.send_message(msg.chat.id, "Bot is running! 🚀 \nSend /start command to start a game 🕹").await?; }

    let callback = dialogue_data.last_callback.as_ref().ok_or(DialogueStateStorageError)?;
    let keyboard: Buttons = serde_json::from_str(&get_callback_data(callback).await?)?;
    let text = get_text(&msg).await?;
    
    let (message_text, opt_keyboard, opt_dialogue_data): MessageTriplet =
        match (dialogue_data.state.as_ref(), keyboard)
        {
            (State::Starting, ..) => ("Bot is running! 🚀 \nSend /start command to start a game 🕹".to_owned(), None, None),
            (SearchCommandActive(search_config), SearchButtons(SearchCommandButtons::ResultLimit)) =>
                parse_number(text, Either::First(search_config), &dialogue_data),
            (ListCommandActive(list_config), ListButtons(ListCommandButtons::ResultLimit)) =>
                parse_number(text, Either::Last(list_config), &dialogue_data),
            (SearchCommandActive(search_config), SearchButtons(SearchCommandButtons::TextToSearch)) =>
                save_text(text, search_config.clone(), &dialogue_data),
            other =>
                {
                    log::info!(" [:: LOG ::] ... ( @[fn]:[handle_text] [:: {:?} ::] )", other);
                    ("Oops! 🤷‍♂️".to_owned(), None, None)
                }
        };
    update_optionally_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, msg.chat.id, message_text).await
}


