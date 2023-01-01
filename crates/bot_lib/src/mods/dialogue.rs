use teloxide::
{
    Bot,
    dispatching::dialogue::GetChatId,
    requests::Requester,
    types::{CallbackQuery, InlineKeyboardMarkup, Message},
};

use crate::mods::
{
    dialogue::callback_handling::{callback_helper_for_list_kb, callback_helper_for_search_kb},
    dialogue::funcs::{get_callback_data, get_dialogue_data, get_text, update_optionally_and_send_message},
    dialogue::text_handling::{execute_search, parse_number},
    dialogue::types::{DialogueData, Either, ListConfigData, SearchConfigData, State::{self, ListCommandActive, SearchCommandActive}, TheDialogue},
    errors::{EndpointErrors, DialogueStateStorageError},
    inline_keyboards::types::{KeyBoard::{self, ListCommand, SearchCommand}, ListCommandKB, SearchCommandKB, SearchMode},
};

pub(crate) mod funcs;
pub(crate) mod types;
pub(crate) mod text_handling;
pub(crate) mod callback_handling;

/// Main `text` handler.
pub async fn handle_callback_data(bot: Bot, callback: CallbackQuery, dialogue: TheDialogue) -> eyre::Result<()>
{
    let dialogue_data = get_dialogue_data(&dialogue).await?;
    let chat_id = callback.chat_id().ok_or(EndpointErrors::GameError)?;
    let keyboard: KeyBoard = serde_json::from_str(&get_callback_data(&callback).await?)?;
    let (message_text, opt_keyboard, opt_dialogue_data):
        (String, Option<InlineKeyboardMarkup>, Option<DialogueData>) =
        match &keyboard
        {
            SearchCommand(search_kb) => callback_helper_for_search_kb(search_kb, dialogue_data, callback),
            ListCommand(list_kb) => callback_helper_for_list_kb(list_kb, dialogue_data, callback),
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
    let keyboard: KeyBoard = serde_json::from_str(&get_callback_data(callback).await?)?;
    let text = get_text(&msg).await?;

    let (message_text, opt_keyboard, opt_dialogue_data):
        (String, Option<InlineKeyboardMarkup>, Option<DialogueData>) =
        match (dialogue_data.state.as_ref(), keyboard)
        {
            (State::Starting, ..) => ("Bot is running! 🚀 \nSend /start command to start a game 🕹".to_owned(), None, None),
            (SearchCommandActive(SearchConfigData { search_by: Some(s), target: Some(_), result_limit: Some(r) }), _) =>
                execute_search(&bot, &msg, &dialogue_data, text, *r, s).await?,
            (ListCommandActive(ListConfigData { sort_by: Some(_), target: Some(_), filter: Some(_), result_limit: Some(r) }), _) =>
                execute_search(&bot, &msg, &dialogue_data, text, *r, &SearchMode::Title).await?,
            (SearchCommandActive(search_config), SearchCommand(SearchCommandKB::ResultLimit)) =>
                parse_number(text, Either::First(search_config), &dialogue_data),
            (ListCommandActive(list_config), ListCommand(ListCommandKB::ResultLimit)) =>
                parse_number(text, Either::Last(list_config), &dialogue_data),
            other =>
                {
                    log::info!(" [:: LOG ::] : [:: {:?} ::]", other);
                    ("Oops! 🤷‍♂️".to_owned(), None, None)
                }
        };
    update_optionally_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, msg.chat.id, message_text).await?;
    Ok(())
}


