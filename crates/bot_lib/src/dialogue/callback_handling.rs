use teloxide::Bot;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::types::CallbackQuery;

use error_traits::{LogErr, MapErrBy, WrapInOk};

use crate::dialogue::funcs::{list_settings_update_or_default, search_settings_update_or_default};
use crate::dialogue::text_handling::{execute_list_command, execute_search_command};
use crate::dialogue::types::{DialogueData, ListCommandSettings, MessageTriplet, SearchCommandSettings};
use crate::dialogue::types::State::{ListCommandActive, SearchCommandActive};
use crate::FlatRes;
use crate::keyboards::traits::{CreateKB, KeyboardText};
use crate::keyboards::types::{ListCommandButtons, SearchCommandButtons, Target};
use crate::net::traits::{RespTargetPlaylists, RespTargetSubscriptions};

/// Helper function used for `handle_callback_data` handler.
pub(crate) async fn callback_helper_for_search_kb
(
    bot: &Bot,
    search_kb: &SearchCommandButtons,
    dialogue_data: DialogueData,
    callback: CallbackQuery,
)
    -> FlatRes<MessageTriplet>
{
    let opt_dialogue_data =
        match (search_kb, dialogue_data.state.as_ref())
        {
            (SearchCommandButtons::SearchIn(search_in), _) =>
                {
                    let search_in = search_in.clone().into();
                    let search_settings = SearchCommandSettings { search_in, ..search_settings_update_or_default(dialogue_data.state) };
                    let state = SearchCommandActive(search_settings);
                    Some(DialogueData { state, ..dialogue_data })
                }
            (SearchCommandButtons::Target(target), _) =>
                {
                    let target = target.clone().into();
                    let search_settings = SearchCommandSettings { target, ..search_settings_update_or_default(dialogue_data.state) };
                    let state = SearchCommandActive(search_settings);
                    Some(DialogueData { state, ..dialogue_data })
                }
            (SearchCommandButtons::ResultLimit, _) =>
                {
                    let state = SearchCommandActive(SearchCommandSettings { ..search_settings_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    Some(DialogueData { state, last_callback, ..dialogue_data })
                }
            (SearchCommandButtons::TextToSearch, _) =>
                {
                    let state = SearchCommandActive(SearchCommandSettings { ..search_settings_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    Some(DialogueData { state, last_callback, ..dialogue_data })
                }
            (SearchCommandButtons::Execute, SearchCommandActive(search_settings)) =>
                return exec_search_helper(bot, search_settings, &dialogue_data, callback).await,
            _ => dialogue_data.into()
        };
    (search_kb.kb_text(), search_kb.create_kb(), opt_dialogue_data).in_ok()
}

async fn exec_search_helper
(
    bot: &Bot, 
    search_settings: &SearchCommandSettings, 
    dialogue_data: &DialogueData, 
    callback: CallbackQuery
)
    -> FlatRes<MessageTriplet>
{
    let search_config = search_settings.clone().build_config().map_err(|e| (e, None, dialogue_data.clone().into()))?;
    let (search_for, res_limit, search_in) = 
        (search_config.text_to_search, search_config.result_limit, search_config.search_in);
    let user_err = || ("Couldn't execute command ❌".to_owned(), None, dialogue_data.clone().into());
    let err_log_prefix = " [:: LOG ::]  :  @fn:[dialogue::callback_handling]  ->  error: ";
    let send_to = callback.chat_id().ok_or(()).map_err_by(user_err)?;
    let user_id = callback.from;
    let res =
        match search_config.target
        {
            Target::Subscription =>
                execute_search_command(bot, user_id, send_to, &search_for, res_limit, &search_in, RespTargetSubscriptions).await,
            Target::PlayList =>
                execute_search_command(bot, user_id, send_to, &search_for, res_limit, &search_in, RespTargetPlaylists).await,
        };
    res.log_err(err_log_prefix).map_err_by(user_err)
}

/// Helper function used for `handle_callback_data` handler.
pub(crate) async fn callback_helper_for_list_kb
(
    bot: &Bot, 
    list_kb: &ListCommandButtons, 
    dialogue_data: DialogueData,
    callback: CallbackQuery
)
    -> FlatRes<MessageTriplet>
{
    let opt_dialogue_data =
        match (list_kb, dialogue_data.state.as_ref())
        {
            (ListCommandButtons::Target(target), _) =>
                {
                    let target = target.clone().into();
                    let state = ListCommandActive(ListCommandSettings { target, ..list_settings_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            (ListCommandButtons::Sorting(sorting), _) =>
                {
                    let sorting = sorting.clone().into();
                    let state = ListCommandActive(ListCommandSettings { sorting, ..list_settings_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            (ListCommandButtons::ResultLimit, _) =>
                {
                    let state = ListCommandActive(ListCommandSettings { ..list_settings_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    DialogueData { state, last_callback, ..dialogue_data }.into()
                }
            (ListCommandButtons::Execute, ListCommandActive(list_setting)) =>
                return exec_list_helper(bot, list_setting, &dialogue_data, callback).await,
            _ => dialogue_data.into()
        };
    (list_kb.kb_text(), list_kb.create_kb(), opt_dialogue_data).in_ok()
}

async fn exec_list_helper
(
    bot: &Bot,
    list_setting: &ListCommandSettings, 
    dialogue_data: &DialogueData,
    callback: CallbackQuery
)
    -> FlatRes<MessageTriplet>
{
    let list_config = list_setting.clone().build_config().map_err(|e| (e, None, dialogue_data.clone().into()))?;
    let (res_limit, sorting) = (list_config.result_limit, list_config.sorting);
    let user_err = || ("Couldn't execute command ❌".to_owned(), None, dialogue_data.clone().into());
    let err_log_prefix = " [:: LOG ::]  :  @fn:[dialogue::callback_handling]  ->  error: ";
    let send_to = callback.chat_id().ok_or(()).map_err_by(user_err)?;
    let res =
        match list_config.target
        {
            Target::Subscription =>
                execute_list_command(bot, callback.from, send_to, res_limit, &sorting, RespTargetSubscriptions).await,
            Target::PlayList =>
                execute_list_command(bot, callback.from, send_to, res_limit, &sorting, RespTargetPlaylists).await
        };
    res.log_err(err_log_prefix).map_err_by(user_err)
}


