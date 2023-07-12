
use error_traits::{LogErr, MapErrBy, MergeOkErr, WrapInRes};
use google_youtube3::oauth2::read_application_secret;
use teloxide::types::{CallbackQuery, User};

use crate::model::db::{get_access_token, refresh_access_token, build_refresh_access_token_req};
use crate::model::dialogue::funcs::{default_auth_url, get_callback_data, get_dialogue_data};
use crate::model::dialogue::funcs::{list_settings_update_or_default, search_settings_update_or_default};
use crate::model::dialogue::types::{DialogueData, TheDialogue, ListConfig, SearchConfig, CommandConfig};
use crate::model::dialogue::types::State::{ListCommandActive, SearchCommandActive};
use crate::model::keyboards::traits::{CreateKB, KeyboardText};
use crate::model::keyboards::types::{Buttons, ListCommandButtons, Requestable, SearchCommandButtons, SearchIn, Sorting};
use crate::model::keyboards::types::Buttons::{ListButtons, SearchButtons};
use crate::model::utils::HTMLise;
use crate::model::youtube::funcs::{list_items, search_items};
use crate::model::youtube::types::SearchableItem;
use crate::StdResult;
use crate::view::types::Sendable;


type ResTriplet = (Option<String>, Vec<SearchableItem>, Option<String>);


pub(crate) async fn get_required_callback_data(callback: &CallbackQuery, dialogue: TheDialogue)
    -> StdResult<(DialogueData, Buttons), String>
{
    log::info!(" [:: LOG ::]     @[fn]:[get_required_callback_data] :: [Started]");

    let user_error = || "⚠ Internal error ⚠";

    let dialogue_data = get_dialogue_data(&dialogue).await.map_err_by(user_error)?;
    let callback_data = get_callback_data(callback).await.map_err_by(user_error)?;
    let keyboard: Buttons = serde_json::from_str(&callback_data).map_err_by(user_error)?;
    
    (dialogue_data, keyboard).in_ok()
}

/// Main `text` handler.
pub(crate) async fn handle_callback(callback: CallbackQuery, dialogue: TheDialogue)
    -> Sendable<String>
{
    log::info!(" [:: LOG ::]     @[fn]:[handlers::handle_callback] :: [Started]");

    let (d_data, buttons) =
        match get_required_callback_data(&callback, dialogue).await
        {
            Ok(ok) => ok,
            Err(e) => return Sendable::SendError(e)
        };
    let res =
        match &buttons
        {
            SearchButtons(search_kb) => callback_helper_for_search_kb(search_kb, d_data, callback).await,
            ListButtons(list_kb) => callback_helper_for_list_kb(list_kb, d_data, callback).await
        };
    res.map_err(Sendable::SendError).merge_ok_err()
}

/// Helper function used for `handle_callback_data` handler.
pub(crate) async fn callback_helper_for_search_kb
(
    search_kb: &SearchCommandButtons,
    dialogue_data: DialogueData,
    callback: CallbackQuery,
)
    -> StdResult<Sendable<String>, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[callback_helper_for_search_kb] :: [Started]");

    let opt_dialogue_data =
        match (search_kb, dialogue_data.state.as_ref())
        {
            (SearchCommandButtons::Execute, SearchCommandActive(search_settings)) =>
                {
                    let config = search_settings.clone().build_config()?;
                    return Sendable::ExecuteCommand(CommandConfig::SearchConfig(config)).in_ok()
                }
            (SearchCommandButtons::SearchIn(search_in), _) =>
                {
                    let mut search_settings = search_settings_update_or_default(dialogue_data.state);
                    search_settings.update_search_in(search_in.clone());
                    Some(DialogueData { state: SearchCommandActive(search_settings), ..dialogue_data })
                }
            (SearchCommandButtons::Target(target), _) =>
                {
                    let mut search_settings = search_settings_update_or_default(dialogue_data.state);
                    search_settings.update_target(target.clone());
                    Some(DialogueData { state: SearchCommandActive(search_settings), ..dialogue_data })
                }
            (SearchCommandButtons::ResultLimit, _) =>
                {
                    let search_settings = search_settings_update_or_default(dialogue_data.state);
                    let last_callback = callback.into();
                    Some(DialogueData { state: SearchCommandActive(search_settings), last_callback, ..dialogue_data })
                }
            (SearchCommandButtons::TextToSearch, _) =>
                {
                    let search_settings = search_settings_update_or_default(dialogue_data.state);
                    let last_callback = callback.into();
                    Some(DialogueData { state: SearchCommandActive(search_settings), last_callback, ..dialogue_data })
                }
            _ => dialogue_data.into()
        };
    
    let (text, opt_kb) = (search_kb.kb_text(), search_kb.create_kb());
    let opt_msg_with_kb = opt_dialogue_data.as_ref().and_then(|d| d.message_with_kb.opt_message.clone());
    
    match (opt_kb, opt_dialogue_data, opt_msg_with_kb)
    {
        (Some(kb), Some(d), Some(m)) => Sendable::EditKeyboard(text.into(), kb, m, d.into()).in_ok(),
        (_, Some(d), _) => Sendable::SendOrEditMessage(text, None, d.into()).in_ok(),
        _ => Sendable::SendOrEditMessage(text, None, None).in_ok()
    }
}

pub(crate) async fn exec_search_helper
(
    search_config: SearchConfig,
    callback: CallbackQuery
)
    -> StdResult<ResTriplet, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[exec_search_helper] :: [Started]");

    let err_log_prefix = " [:: LOG ::]  :  @fn:[dialogue::callback_handling]  ->  error: ";

    let (requestable, search_for, res_limit, search_in) =
        (search_config.target, search_config.text_to_search, search_config.result_limit, search_config.search_in);

    let res = execute_search_command(callback.from, &search_for, res_limit, &search_in, requestable).await;
    res.log_err(err_log_prefix)
}

/// Helper function used for `handle_callback_data` handler.
pub(crate) async fn callback_helper_for_list_kb
(
    list_kb: &ListCommandButtons,
    dialogue_data: DialogueData,
    callback: CallbackQuery
)
    -> StdResult<Sendable<String>, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[callback_helper_for_list_kb] :: [Started]");

    let opt_dialogue_data =
        match (list_kb, dialogue_data.state.as_ref())
        {
            (ListCommandButtons::Execute, ListCommandActive(list_setting)) =>
                {
                    let config = list_setting.clone().build_config()?;
                    return Sendable::ExecuteCommand(CommandConfig::ListConfig(config)).in_ok();
                }
            (ListCommandButtons::Target(target), _) =>
                {
                    let mut list_settings = list_settings_update_or_default(dialogue_data.state);
                    list_settings.update_target(target.clone());
                    Some(DialogueData { state: ListCommandActive(list_settings), ..dialogue_data })
                }
            (ListCommandButtons::Sorting(sorting), _) =>
                {
                    let mut list_settings = list_settings_update_or_default(dialogue_data.state);
                    list_settings.update_sorting(sorting.clone());
                    Some(DialogueData { state: ListCommandActive(list_settings), ..dialogue_data })
                }
            (ListCommandButtons::ResultLimit, _) =>
                {
                    let list_settings = list_settings_update_or_default(dialogue_data.state);
                    let last_callback = callback.into();
                    Some(DialogueData { state: ListCommandActive(list_settings), last_callback, ..dialogue_data })
                }
            _ => dialogue_data.into()
        };
    
    let (text, opt_kb) = (list_kb.kb_text(), list_kb.create_kb());
    let opt_msg_with_kb = opt_dialogue_data.as_ref().and_then(|d| d.message_with_kb.opt_message.clone());
    
    match (opt_kb, opt_dialogue_data, opt_msg_with_kb)
    {
        (Some(kb), Some(d), Some(m)) => Sendable::EditKeyboard(text.into(), kb, m, d.into()).in_ok(),
        (_, Some(d), _) => Sendable::SendOrEditMessage(text, None, d.into()).in_ok(),
        _ => Sendable::SendOrEditMessage(text, None, None).in_ok()
    }
}

pub(crate) async fn exec_list_helper
(
    list_config: ListConfig,
    callback: CallbackQuery
)
    -> StdResult<ResTriplet, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[exec_list_helper] :: [Started]");

    let err_log_prefix = " [:: LOG ::]  :  @fn:[dialogue::callback_handling]  ->  error: ";

    let (requestable, res_limit, sorting) = (list_config.target, list_config.result_limit, list_config.sorting);

    let res = execute_list_command(callback.from, res_limit, &sorting, requestable).await;
    res.log_err(err_log_prefix)
}

/// Helper function used for `handle_text` handler.
/// Final func that does searching when everything is ready.
pub(crate) async fn execute_search_command
(
    user_id: User,
    search_for: &str,
    res_limit: u32,
    search_in: &SearchIn,
    requestable: Requestable,
)
    -> StdResult<ResTriplet, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[execute_search_command] :: [Started]");

    let err = || "⚠ Internal error ⚠";

    let user_id = user_id.id.0.to_string();
    let db_url = env!("REDIS_URL");

    let Ok(token) =
        get_access_token(&user_id, db_url)
        else
        {
            let mut auth_url = default_auth_url(&user_id).await.map_err_by(err)?.to_link("Log In");
            auth_url = format!("Use this link to log in {auth_url} \nPlease, log in and send your text again");
            return auth_url.in_err()
        };
    
    let secret = read_application_secret(env!("PATH_TO_GOOGLE_OAUTH_SECRET")).await.map_err_by(err)?;
    let token_req = build_refresh_access_token_req(secret, &token).map_err_by(err)?;
    let access_token = refresh_access_token(&user_id, token, db_url, token_req).await.map_err_by(err)?.access_token;
    
    let results =
        match requestable
        {
            Requestable::Subscription(s) => search_items(search_in, s, search_for, &access_token, res_limit).await,
            Requestable::Playlist(p) => search_items(search_in, p, search_for, &access_token, res_limit).await
        };
    let result_count = results.len();
    let (prefix, postfix) = (None::<String>, format!("Finished! ✔ \nFound {result_count} results").into());
    Ok((prefix, results, postfix))
}

/// Helper function used for `handle_text` handler.
/// Final func that does searching when everything is ready.
pub(crate) async fn execute_list_command
(
    user_id: User,
    res_limit: u32,
    sorting: &Sorting,
    requestable: Requestable,
)
    -> StdResult<ResTriplet, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[execute_list_command] :: [Started]");
    
    let err = || "⚠ Internal error ⚠";

    let user_id = user_id.id.0.to_string();
    let db_url = env!("REDIS_URL");
    let Ok(token) =
        get_access_token(&user_id, db_url)
        else
        {
            let mut auth_url = default_auth_url(&user_id).await.map_err_by(err)?.to_link("Log In");
            auth_url = format!("Use this link to log in {auth_url} \nPlease, log in and send your text again");
            return auth_url.in_err()
        };
    
    let secret = read_application_secret(env!("PATH_TO_GOOGLE_OAUTH_SECRET")).await.map_err_by(err)?;
    let token_req = build_refresh_access_token_req(secret, &token).map_err_by(err)?;
    let access_token = refresh_access_token(&user_id, token, db_url, token_req).await.map_err_by(err)?.access_token;

    let results =
        match requestable
        {
            Requestable::Subscription(s) => list_items(s, &access_token, sorting, res_limit).await,
            Requestable::Playlist(p) => list_items(p, &access_token, sorting, res_limit).await,
        };
    let result_count = results.len();
    let (prefix, postfix) = (None::<String>, format!("Finished! ✔ \nFound {result_count} results").into());
    Ok((prefix, results, postfix))
}


#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use std::default::Default;
    use teloxide::types::UserId;
    use crate::model::dialogue::types::{SearchCommandSettings, State};
    use crate::model::net::types::SubscriptionRequester;
    use super::*;
    
    fn create_user() -> User
    {
        let (id, first_name) = (UserId(8587687687), "hgjggo".to_owned());
        let (is_bot, last_name, language_code) = Default::default();
        let (username, is_premium, added_to_attachment_menu) = Default::default();
        User { id, is_bot, first_name, last_name, username, language_code, is_premium, added_to_attachment_menu }
    }
    
    fn create_callback() -> CallbackQuery
    {
        let (id, chat_instance, from) = (87568758.to_string(), 76876959.to_string(), create_user());
        let data = serde_json::to_string(&SearchCommandButtons::ResultLimit).unwrap().into();
        let (message, game_short_name, inline_message_id) = Default::default();
        CallbackQuery { id, from, message, inline_message_id, chat_instance, data, game_short_name }
    }
    
    #[tokio::test]
    async fn save_result_limit_test()
    {
        let search_kb = SearchCommandButtons::ResultLimit;
        
        let target = Some(Requestable::Subscription(SubscriptionRequester));
        let search_in = Some(SearchIn::Title);
        let search_settings = SearchCommandSettings { target, search_in, ..Default::default() };
        let state: State = SearchCommandActive(search_settings);
        
        let d_data = DialogueData { state, ..Default::default() };
        
        let callback: CallbackQuery = create_callback();
        
        let r: Option<DialogueData> =
            match &search_kb
            {
                SearchCommandButtons::ResultLimit =>
                    {
                        let state = SearchCommandActive(SearchCommandSettings { ..search_settings_update_or_default(d_data.state.clone()) });
                        let last_callback = callback.clone().into();
                        Some(DialogueData { state, last_callback, ..d_data.clone() })
                    }
                _ => panic!("hehe")
            };
        
        assert!(matches!(r, Some(..)));
        
        let res: Sendable<String> =
            callback_helper_for_search_kb(&search_kb, d_data.clone(), callback.clone()).await.unwrap();
        if let Sendable::EditKeyboard(Some(_), _, _, Some(d)) = res.clone()
        {
            assert!(matches!(d.state, State::SearchCommandActive(SearchCommandSettings { result_limit: Some(..), .. })));
        }
        else if let Sendable::SendOrEditMessage(_, None, Some(d)) = res
        {
            assert!(matches!(d.last_callback, Some(..)));
            let callback_data = d.last_callback.unwrap().data.unwrap();
            let last_callback = serde_json::from_str::<SearchCommandButtons>(&callback_data).unwrap();
            assert!(matches!(last_callback, SearchCommandButtons::ResultLimit));
        }
        else
        {
            dbg!(res);
            panic!("it's a panic!")
        }
    }
}


