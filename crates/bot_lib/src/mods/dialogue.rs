use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::requests::Requester;
use teloxide::types::{CallbackQuery, Message};

use crate::mods::
{
    dialogue::callback_handling::{callback_helper_for_list_kb, callback_helper_for_search_kb},
    dialogue::funcs::{get_callback_data, get_dialogue_data, get_text, update_optionally_and_send_message},
    dialogue::text_handling::{execute_search_command, parse_number},
    dialogue::types::{Either, ListCommandSettings, SearchCommandSettings, State::{self, ListCommandActive, SearchCommandActive}, TheDialogue},
    errors::{DialogueStateStorageError, EndpointErrors},
    inline_keyboards::types::{Buttons::{self, ListButtons, SearchButtons}, SearchCommandButtons},
};
use crate::mods::dialogue::text_handling::execute_list_command;
use crate::mods::dialogue::types::MessageTriplet;
use crate::mods::inline_keyboards::types::Target;
use crate::mods::net::traits::{ListPlaylists, ListSubscriptions};

pub(crate) mod funcs;
pub(crate) mod types;
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
            SearchButtons(search_kb) => callback_helper_for_search_kb(search_kb, dialogue_data, callback),
            ListButtons(list_kb) => callback_helper_for_list_kb(list_kb, dialogue_data, callback),
        };
    update_optionally_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, chat_id, message_text).await?;
    Ok(())
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
            (SearchCommandActive(SearchCommandSettings { search_by: Some(s), target: Some(Target::Subscription), result_limit: Some(r) }), _) =>
                execute_search_command(&bot, &msg, &dialogue_data, text, *r, s, ListSubscriptions)
                    .await?,
            (SearchCommandActive(SearchCommandSettings { search_by: Some(s), target: Some(Target::PlayList), result_limit: Some(r) }), _) =>
                execute_search_command(&bot, &msg, &dialogue_data, text, *r, s, ListPlaylists)
                    .await?,
            (ListCommandActive(ListCommandSettings { target: Some(Target::Subscription), .. }), _) =>
                execute_list_command(&bot, &msg, ListSubscriptions).await?,
            (ListCommandActive(ListCommandSettings { target: Some(Target::PlayList), .. }), _) =>
                execute_list_command(&bot, &msg, ListPlaylists).await?,
            (SearchCommandActive(search_config), SearchButtons(SearchCommandButtons::ResultLimit)) =>
                parse_number(text, Either::First(search_config), &dialogue_data),
            //(ListCommandActive(list_config), ListButtons(ListCommandButtons::ResultLimit)) =>
            //    parse_number(text, Either::Last(list_config), &dialogue_data),
            other =>
                {
                    log::info!(" [:: LOG ::] : [:: {:?} ::]", other);
                    ("Oops! 🤷‍♂️".to_owned(), None, None)
                }
        };
    update_optionally_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, msg.chat.id, message_text).await?;
    Ok(())
}


