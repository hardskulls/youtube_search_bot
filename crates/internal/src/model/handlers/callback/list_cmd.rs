
use error_traits::{PassErrWith, WrapInRes};
use teloxide::prelude::CallbackQuery;
use teloxide::types::User;
use crate::dialogue::DialogueData;
use crate::model::db::get_access_token;
use crate::model::dialogue::funcs::list_settings_update_or_default;
use crate::model::dialogue::types::{CommandConfig, ListConfig};
use crate::model::dialogue::types::State::ListCommandActive;
use crate::model::handlers::callback::common::{construct_login_url, ResTriplet, update_and_return_access_token};
use crate::model::keyboards::traits::{CreateKB, KeyboardText};
use crate::model::keyboards::types::{ListCommandButtons, Requestable, Sorting};
use crate::model::youtube::funcs::list_items;
use crate::StdResult;
use crate::view::types::Sendable;


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
    log::info!(" [:: LOG ::]     @[fn]:[exec_search_helper] :: [Started]");

    let err = |_| "⚠ Internal error ⚠".to_owned();
    let log_prefix = " [:: LOG ::]  :  @fn:[dialogue::callback_handling]  ->  error: ";

    let (requestable, res_limit, sorting) = (list_config.target, list_config.result_limit, list_config.sorting);

    let res = execute_list_command(callback.from, res_limit, &sorting, requestable).await;
    res.pass_err_with(|e| log::error!("{log_prefix}{e}")).map_err(err)
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
    -> eyre::Result<ResTriplet>
{
    log::info!(" [:: LOG ::]     @[fn]:[execute_search_command] :: [Started]");

    let user_id = user_id.id.0.to_string();
    let db_url = env!("REDIS_URL");

    let Ok(token) =
        get_access_token(&user_id, db_url)
        else
        { return (construct_login_url(&user_id).await?.into(), vec![], None).in_ok() };

    let access_token = update_and_return_access_token(&user_id, token, db_url).await?;

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


