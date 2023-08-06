
use error_traits::{PassErrWith, WrapInRes};
use teloxide::prelude::CallbackQuery;
use teloxide::types::User;
use crate::dialogue::DialogueData;
use crate::model::db::get_access_token;
use crate::model::dialogue::funcs::search_settings_update_or_default;
use crate::model::dialogue::types::{CommandConfig, SearchConfig};
use crate::model::dialogue::types::State::SearchCommandActive;
use crate::model::handlers::callback::common::{construct_login_url, ResTriplet, update_and_return_access_token};
use crate::model::keyboards::traits::{CreateKB, KeyboardText};
use crate::model::keyboards::types::{Requestable, SearchCommandButtons, SearchIn};
use crate::model::youtube::funcs::search_cmd::search_items;
use crate::StdResult;
use crate::view::types::Sendable;


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

pub(crate) async fn execute_search_command
(
    search_config: SearchConfig,
    callback: CallbackQuery
)
    -> StdResult<ResTriplet, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[exec_search_helper] :: [Started]");

    let err = |_| "⚠ Internal error ⚠".to_owned();
    let log_prefix = " [:: LOG ::]  :  @fn:[dialogue::callback_handling]  ->  error: ";

    let (requestable, search_for, res_limit, search_in) =
        (search_config.target, search_config.text_to_search, search_config.result_limit, search_config.search_in);

    let res = exec_search_helper(callback.from, &search_for, res_limit, &search_in, requestable).await;
    res.pass_err_with(|e| log::error!("{log_prefix}{e}")).map_err(err)
}


/// Helper function used for `handle_text` handler.
/// Final func that does searching when everything is ready.
pub(crate) async fn exec_search_helper
(
    user_id: User,
    search_for: &str,
    res_limit: u32,
    search_in: &SearchIn,
    requestable: Requestable,
)
    -> eyre::Result<ResTriplet>
{
    log::info!(" [:: LOG ::]     @[fn]:[execute_search_command] :: [Started]");

    let user_id = user_id.id.0.to_string();
    let db_url = env!("REDIS_YOUTUBE_ACCESS_TOKEN_STORAGE");

    let Ok(token) =
        get_access_token(&user_id, db_url)
        else
        { return (construct_login_url(&user_id).await?.into(), vec![], None).in_ok() };

    let access_token = update_and_return_access_token(&user_id, token, db_url).await?;

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

        let res: Option<DialogueData> =
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

        assert!(matches!(res, Some(..)));

        let res: Sendable<String> =
            callback_helper_for_search_kb(&search_kb, d_data.clone(), callback.clone()).await.unwrap();
        if let Sendable::EditKeyboard(Some(_), .., Some(d)) = res.clone()
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


