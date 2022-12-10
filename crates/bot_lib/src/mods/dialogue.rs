use teloxide::
{
    Bot,
    dispatching::dialogue::GetChatId,
    types::{CallbackQuery, InlineKeyboardMarkup, Message},
};

use teloxide::requests::Requester;

use KeyBoard::{ListCommand, SearchCommand};
use State::{ListCommandActive, SearchCommandActive};


use crate::mods::
{
    dialogue::callback_handling::{callback_helper_for_list_kb, callback_helper_for_search_kb},
    dialogue::helpers::{get_callback_data, get_dialogue_data, get_text, update_optionally_and_send_message},
    dialogue::text_handling::execute_search,
    dialogue::types::{DialogueData, ListConfigData, SearchConfigData, State, TheDialogue},
    errors::EndpointErrors,
    inline_keyboards::types::{KeyBoard, ListCommandKB, SearchCommandKB, SearchMode},
};
use crate::mods::dialogue::text_handling::parse_number;
use crate::mods::dialogue::types::Either;


pub(crate) mod helpers;
pub(crate) mod types;
pub(crate) mod text_handling;
pub(crate) mod callback_handling;


pub async fn handle_callback_data(bot: Bot, callback: CallbackQuery, dialogue: TheDialogue) -> eyre::Result<()>
{
    // log::info!("[:: 'handle_callback_data' started ::]");
    // log::info!("[:: LOG DATA ::]  :  [:: INPUT of 'update_optionally_and_send_message' is: {:#?} ::]", (&callback));
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
    update_optionally_and_send_message
        (
            dialogue.into(),
            opt_dialogue_data,
            opt_keyboard,
            bot,
            chat_id,
            message_text,
        )
        .await?;
    Ok(())
}


pub async fn handle_text(bot: Bot, msg: Message, dialogue: TheDialogue) -> eyre::Result<()>
{
    // log::info!(" [:: LOG ::] : [:: 'handle_text' started ::] ");
    let dialogue_data = get_dialogue_data(&dialogue).await?;
    let callback =
        match dialogue_data.last_callback.as_ref()
        {
            Some(callback_query) => callback_query,
            None =>
                {
                    bot.send_message(msg.chat.id, "Bot is running! 🚀 \nSend /start command to start a game 🕹").await?;
                    return Ok(())
                }
        };
    let keyboard: KeyBoard = serde_json::from_str(&get_callback_data(callback).await?)?;
    let text = get_text(&msg).await?;
    // log::info!("[:: LOG DATA ::]  :  [:: INPUT of 'handle_text' is: {:#?} ::]", (&msg.reply_markup().is_some(), &dialogue_data, &callback, &keyboard, &text));
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
                parse_number(text, Either::Left(search_config), &dialogue_data),
            (ListCommandActive(list_config), ListCommand(ListCommandKB::ResultLimit)) =>
                parse_number(text, Either::Right(list_config), &dialogue_data),
            other =>
                {
                    log::info!(" [:: LOG ::] : [:: {:?} ::]", other);
                    ("Oops! 🤷‍♂️".to_owned(), None, None)
                }
        };
    update_optionally_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, msg.chat.id, message_text).await?;
    Ok(())
}

#[cfg(test)]
mod tests
{
    #[test]
    fn into_option_test()
    {
        let t = 8;
        assert!(matches!(t.into(), Some(_)));
    }
}


