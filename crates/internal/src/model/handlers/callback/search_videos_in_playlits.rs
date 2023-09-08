
use error_traits::{PassErrWith, WrapInRes};
use teloxide::prelude::CallbackQuery;
use teloxide::types::User;
use crate::dialogue::DialogueData;
use crate::model::db::get_access_token;
use crate::model::dialogue::funcs::search_videos_in_playlists_update_or_default;
use crate::model::dialogue::types::{CommandConfig, SearchVideosInPlaylistsConfig};
use crate::model::dialogue::types::State::SearchVideosInPlaylistsCommandActive;
use crate::model::handlers::callback::common::{construct_login_url, ResTriplet, update_and_return_access_token};
use crate::model::keyboards::traits::{CreateKB, KeyboardText};
use crate::model::keyboards::types::{SearchIn, SearchVideoInPlaylistsCommandButtons};
use crate::model::youtube::funcs::search_videos_in_playlists::search_videos_in_playlists;
use crate::StdResult;
use crate::view::types::Sendable;


/// Helper function used for `handle_callback_data` handler.
pub(crate) async fn callback_helper_for_search_videos_in_playlists_kb
(
    search_kb: &SearchVideoInPlaylistsCommandButtons,
    dialogue_data: DialogueData,
    callback: CallbackQuery,
)
    -> StdResult<Sendable<String>, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[callback_helper_for_search_kb] :: [Started]");
    
    use SearchVideoInPlaylistsCommandButtons::{TextToSearch, Execute, ResultLimit};
    let opt_dialogue_data =
        match (search_kb, dialogue_data.state.as_ref())
        {
            (Execute, SearchVideosInPlaylistsCommandActive(search_settings)) =>
                {
                    let config = search_settings.clone().build_config()?;
                    return Sendable::ExecuteCommand(CommandConfig::SearchVideosInPlaylistsConfig(config)).in_ok()
                }
            (SearchVideoInPlaylistsCommandButtons::SearchIn(search_in), _) =>
                {
                    let mut search_videos_in_playlists_settings = search_videos_in_playlists_update_or_default(dialogue_data.state);
                    search_videos_in_playlists_settings.update_search_in(search_in.clone());
                    Some(DialogueData { state: SearchVideosInPlaylistsCommandActive(search_videos_in_playlists_settings), ..dialogue_data })
                }
            (ResultLimit, _) =>
                {
                    let search_videos_in_playlists_settings = search_videos_in_playlists_update_or_default(dialogue_data.state);
                    let last_callback = callback.into();
                    Some(DialogueData { state: SearchVideosInPlaylistsCommandActive(search_videos_in_playlists_settings), last_callback, ..dialogue_data })
                }
            (TextToSearch, _) =>
                {
                    let search_videos_in_playlists_settings = search_videos_in_playlists_update_or_default(dialogue_data.state);
                    let last_callback = callback.into();
                    Some(DialogueData { state: SearchVideosInPlaylistsCommandActive(search_videos_in_playlists_settings), last_callback, ..dialogue_data })
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

pub(crate) async fn execute_search_videos_in_playlists_command
(
    search_config: SearchVideosInPlaylistsConfig,
    callback: CallbackQuery
)
    -> StdResult<ResTriplet, String>
{
    log::info!(" [:: LOG ::]     @[fn]:[exec_search_helper] :: [Started]");
    
    let err = |_| "⚠ Internal error ⚠".to_owned();
    let log_prefix = " [:: LOG ::]  :  @fn:[dialogue::callback_handling]  ->  error: ";
    
    let (search_for, res_limit, search_in) =
        (search_config.text_to_search, search_config.result_limit, search_config.search_in);
    
    let res = exec_search_videos_in_playlists_helper(callback.from, &search_for, res_limit, &search_in).await;
    res.pass_err_with(|e| log::error!("{log_prefix}{e:?}")).map_err(err)
}

/// Helper function used for `handle_text` handler.
/// Final func that does searching when everything is ready.
pub(crate) async fn exec_search_videos_in_playlists_helper
(
    user_id: User,
    search_for: &str,
    res_limit: u32,
    search_in: &SearchIn
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
    
    // let access_token = token.access_token;
    let access_token = update_and_return_access_token(&user_id, token, db_url).await?;
    
    let results = search_videos_in_playlists(search_in, search_for, &access_token, res_limit).await;
    let result_count = results.len();
    let (prefix, postfix) = (None::<String>, format!("Finished! ✔ \nFound {result_count} results").into());
    Ok((prefix, results, postfix))
}


