use teloxide::
{
    Bot,
    dispatching::dialogue::GetChatId,
    types::{CallbackQuery, InlineKeyboardMarkup},
};

use State::{ListCommandActive, SearchCommandActive};
use KeyBoard::{ListCommand, SearchCommand};

use crate::mods::
{
    dialogue::helpers::{get_callback_data, get_dialogue_data, list_config_update_or_default},
    dialogue::helpers::{search_config_update_or_default, update_state_and_send_message},
    dialogue::types::{DialogueData, State, ListConfigData, SearchConfigData, TheDialogue},
    errors::EndpointErrors,
    inline_keyboards::funcs::{CreateKB, KeyboardText},
    inline_keyboards::types::{KeyBoard, ListCommandKB, SearchCommandKB},
};

pub async fn handle_callback_data(bot: Bot, callback: CallbackQuery, dialogue: TheDialogue)
    -> eyre::Result<()>
{
    let dialogue_data = get_dialogue_data(&dialogue).await?;
    let chat_id = callback.chat_id().ok_or(EndpointErrors::GameError)?;
    let callback_data = get_callback_data(&callback).await?;
    let keyboard: KeyBoard = serde_json::from_str(&callback_data)?;
    let (message_text, opt_keyboard, opt_dialogue_data): (String, Option<InlineKeyboardMarkup>, Option<DialogueData>) =
        match &keyboard
        {
            SearchCommand(search_kb) => callback_helper_for_search_kb(search_kb, dialogue_data, callback),
            ListCommand(list_kb) => callback_helper_for_list_kb(list_kb, dialogue_data, callback),
        };
    update_state_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, chat_id, message_text).await?;
    Ok(())
}

pub(crate) fn callback_helper_for_search_kb(search_kb: &SearchCommandKB, dialogue_data: DialogueData, callback: CallbackQuery)
    -> (String, Option<InlineKeyboardMarkup>, Option<DialogueData>)
{
    let opt_dialogue_data =
        match search_kb
        {
            SearchCommandKB::SearchByContent(search_by) =>
                {
                    let (search_by, last_callback) = (Some(search_by.clone()), Some(callback));
                    let state = SearchCommandActive(SearchConfigData { search_by, ..search_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, last_callback }.into()
                }
            SearchCommandKB::TargetContent(target) =>
                {
                    let (target, last_callback) = (Some(target.clone()), Some(callback));
                    let state = SearchCommandActive(SearchConfigData { target, ..search_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, last_callback }.into()
                }
            _ => None
        };
    (search_kb.keyboard_text(), search_kb.create_kb(), opt_dialogue_data)
}

pub(crate) fn callback_helper_for_list_kb(list_kb: &ListCommandKB, dialogue_data: DialogueData, callback: CallbackQuery)
    -> (String, Option<InlineKeyboardMarkup>, Option<DialogueData>)
{
    let opt_dialogue_data =
        match list_kb
        {
            ListCommandKB::FilterContent(filter) =>
                {
                    let (filter, last_callback) = (Some(filter.clone()), Some(callback));
                    let state = ListCommandActive(ListConfigData { filter, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, last_callback }.into()
                }
            ListCommandKB::TargetContent(target) =>
                {
                    let (target, last_callback) = (Some(target.clone()), Some(callback));
                    let state = ListCommandActive(ListConfigData { target, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, last_callback }.into()
                }
            ListCommandKB::SortContent(sort_by) =>
                {
                    let (sort_by, last_callback) = (Some(sort_by.clone()), Some(callback));
                    let state = ListCommandActive(ListConfigData { sort_by, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, last_callback }.into()
                }
            _ => None
        };
    (list_kb.keyboard_text(), list_kb.create_kb(), opt_dialogue_data)
}


