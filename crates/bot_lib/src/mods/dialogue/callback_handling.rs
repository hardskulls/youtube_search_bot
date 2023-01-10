use teloxide::types::CallbackQuery;

use State::{ListCommandActive, SearchCommandActive};

use crate::mods::dialogue::funcs::{list_config_update_or_default, search_config_update_or_default};
use crate::mods::dialogue::types::{DialogueData, ListCommandSettings, MessageTriplet, SearchCommandSettings, State};
use crate::mods::inline_keyboards::traits::{CreateKB, KeyboardText};
use crate::mods::inline_keyboards::types::{ListCommandButtons, SearchCommandButtons};

/// Helper function used for `handle_callback_data` handler.
pub(crate) fn callback_helper_for_search_kb(search_kb: &SearchCommandButtons, dialogue_data: DialogueData, callback: CallbackQuery)
    -> MessageTriplet
{
    let opt_dialogue_data =
        match search_kb
        {
            SearchCommandButtons::SearchIn(search_by) =>
                {
                    let search_by = search_by.clone().into();
                    let state = SearchCommandActive(SearchCommandSettings { search_by, ..search_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            SearchCommandButtons::Target(target) =>
                {
                    let target = target.clone().into();
                    let state = SearchCommandActive(SearchCommandSettings { target, ..search_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            SearchCommandButtons::ResultLimit =>
                {
                    let state = SearchCommandActive(SearchCommandSettings { ..search_config_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    DialogueData { state, last_callback, ..dialogue_data }.into()
                }
            _ => dialogue_data.into()
        };
    (search_kb.kb_text(), search_kb.create_kb(), opt_dialogue_data)
}

/// Helper function used for `handle_callback_data` handler.
pub(crate) fn callback_helper_for_list_kb(list_kb: &ListCommandButtons, dialogue_data: DialogueData, callback: CallbackQuery)
    -> MessageTriplet
{
    let opt_dialogue_data =
        match list_kb
        {
            ListCommandButtons::Target(target) =>
                {
                    let target = target.clone().into();
                    let state = ListCommandActive(ListCommandSettings { target, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            _ => dialogue_data.into()
        };
    (list_kb.kb_text(), list_kb.create_kb(), opt_dialogue_data)
}


