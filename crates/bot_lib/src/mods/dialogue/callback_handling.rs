use teloxide::types::CallbackQuery;

use State::{ListCommandActive, SearchCommandActive};

use crate::mods::dialogue::funcs::{list_config_update_or_default, search_config_update_or_default};
use crate::mods::dialogue::types::{DialogueData, ListConfigData, MessageContents, SearchConfigData, State};
use crate::mods::inline_keyboards::traits::{CreateKB, KeyboardText};
use crate::mods::inline_keyboards::types::{ListCommandKB, SearchCommandKB};

/// Helper function used for `handle_callback_data` handler.
pub(crate) fn callback_helper_for_search_kb(search_kb: &SearchCommandKB, dialogue_data: DialogueData, callback: CallbackQuery)
    -> MessageContents
{
    let opt_dialogue_data =
        match search_kb
        {
            SearchCommandKB::SearchByContent(search_by) =>
                {
                    let search_by = search_by.clone().into();
                    let state = SearchCommandActive(SearchConfigData { search_by, ..search_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            SearchCommandKB::TargetContent(target) =>
                {
                    let target = target.clone().into();
                    let state = SearchCommandActive(SearchConfigData { target, ..search_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            SearchCommandKB::ResultLimit =>
                {
                    let state = SearchCommandActive(SearchConfigData { ..search_config_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    DialogueData { state, last_callback, ..dialogue_data }.into()
                }
            _ => dialogue_data.into()
        };
    (search_kb.keyboard_text(), search_kb.create_kb(), opt_dialogue_data)
}

/// Helper function used for `handle_callback_data` handler.
pub(crate) fn callback_helper_for_list_kb(list_kb: &ListCommandKB, dialogue_data: DialogueData, callback: CallbackQuery)
    -> MessageContents
{
    let opt_dialogue_data =
        match list_kb
        {
            ListCommandKB::FilterContent(filter) =>
                {
                    let filter = filter.clone().into();
                    let state = ListCommandActive(ListConfigData { filter, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            ListCommandKB::TargetContent(target) =>
                {
                    let target = target.clone().into();
                    let state = ListCommandActive(ListConfigData { target, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            ListCommandKB::SortContent(sort_by) =>
                {
                    let sort_by = sort_by.clone().into();
                    let state = ListCommandActive(ListConfigData { sort_by, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            ListCommandKB::ResultLimit =>
                {
                    let state = ListCommandActive(ListConfigData { ..list_config_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    DialogueData { state, last_callback, ..dialogue_data }.into()
                }
            _ => dialogue_data.into()
        };
    (list_kb.keyboard_text(), list_kb.create_kb(), opt_dialogue_data)
}


