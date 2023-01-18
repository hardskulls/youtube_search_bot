use teloxide::Bot;
use teloxide::types::{CallbackQuery, Message};
use error_traits::WrapInOk;

use State::{ListCommandActive, SearchCommandActive};
use crate::FlatRes;

use crate::mods::dialogue::funcs::{list_config_update_or_default, search_config_update_or_default};
use crate::mods::dialogue::text_handling::{execute_list_command, execute_search_command};
use crate::mods::dialogue::types::{DialogueData, ListCommandSettings, MessageTriplet, SearchCommandSettings, State};
use crate::mods::keyboards::traits::{CreateKB, KeyboardText};
use crate::mods::keyboards::types::{ListCommandButtons, SearchCommandButtons, Target};
use crate::mods::net::traits::{RespTargetPlaylists, RespTargetSubscriptions};

/// Helper function used for `handle_callback_data` handler.
pub(crate) async fn callback_helper_for_search_kb
(
    bot: &Bot,
    msg: &Message,
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
                    let search_settings = SearchCommandSettings { search_in, ..search_config_update_or_default(dialogue_data.state) };
                    let state = SearchCommandActive(search_settings);
                    Some(DialogueData { state, ..dialogue_data })
                }
            (SearchCommandButtons::Target(target), _) =>
                {
                    let target = target.clone().into();
                    let search_settings = SearchCommandSettings { target, ..search_config_update_or_default(dialogue_data.state) };
                    let state = SearchCommandActive(search_settings);
                    Some(DialogueData { state, ..dialogue_data })
                }
            (SearchCommandButtons::ResultLimit, _) =>
                {
                    let state = SearchCommandActive(SearchCommandSettings { ..search_config_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    Some(DialogueData { state, last_callback, ..dialogue_data })
                }
            (SearchCommandButtons::Execute, SearchCommandActive(search_settings)) =>
                {
                    let search_config =
                        search_settings.clone().build_config().map_err(|e| (e, None, dialogue_data.clone().into()))?;
                    let (search_for, res_limit, search_in) =
                        (search_config.text_to_search, search_config.result_limit, search_config.search_in);
                    let res =
                        match search_config.target
                        {
                            Target::Subscription =>
                                execute_search_command(bot, msg, &search_for, res_limit, &search_in, RespTargetSubscriptions)
                                    .await
                                    .map_err(|_| ("Couldn't execute command ❌".to_owned(), None, dialogue_data.into()))?,
                            Target::PlayList =>
                                execute_search_command(bot, msg, &search_for, res_limit, &search_in, RespTargetPlaylists)
                                    .await
                                    .map_err(|_| ("Couldn't execute command ❌".to_owned(), None, dialogue_data.into()))?,
                        };
                    return res.in_ok();
                }
            _ => dialogue_data.into()
        };
    (search_kb.kb_text(), search_kb.create_kb(), opt_dialogue_data).in_ok()
}

/// Helper function used for `handle_callback_data` handler.
pub(crate) async fn callback_helper_for_list_kb
(
    bot: &Bot,
    msg: &Message,
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
                    let state = ListCommandActive(ListCommandSettings { target, ..list_config_update_or_default(dialogue_data.state) });
                    DialogueData { state, ..dialogue_data }.into()
                }
            (ListCommandButtons::ResultLimit, _) =>
                {
                    let state = ListCommandActive(ListCommandSettings { ..list_config_update_or_default(dialogue_data.state) });
                    let last_callback = callback.into();
                    DialogueData { state, last_callback, ..dialogue_data }.into()
                }
            (ListCommandButtons::Execute, ListCommandActive(list_config)) =>
                {
                    let list_config =
                        list_config.clone().build_config().map_err(|e| (e, None, dialogue_data.clone().into()))?;
                    let (res_limit, sorting) =
                        (list_config.result_limit, list_config.sorting);
                    let res =
                        match list_config.target
                        {
                            Target::Subscription =>
                                execute_list_command(bot, msg, res_limit, &sorting, RespTargetSubscriptions)
                                    .await
                                    .map_err(|_| ("Couldn't execute command ❌".to_owned(), None, dialogue_data.into()))?,
                            Target::PlayList =>
                                execute_list_command(bot, msg, res_limit, &sorting, RespTargetPlaylists)
                                    .await
                                    .map_err(|_| ("Couldn't execute command ❌".to_owned(), None, dialogue_data.into()))?,
                        };
                    return res.in_ok();
                }
            _ => dialogue_data.into()
        };
    (list_kb.kb_text(), list_kb.create_kb(), opt_dialogue_data).in_ok()
}


