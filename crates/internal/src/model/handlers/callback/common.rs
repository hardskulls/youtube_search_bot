
use error_traits::{MergeOkErr, PassErrWith, WrapInRes};
use google_youtube3::oauth2::read_application_secret;
use teloxide::prelude::CallbackQuery;
use crate::dialogue::DialogueData;
use crate::model::db::{build_refresh_access_token_req, refresh_access_token};
use crate::model::dialogue::funcs::{default_auth_url, get_callback_data, get_dialogue_data};
use crate::model::dialogue::types::TheDialogue;
use crate::model::handlers::callback::list_cmd::callback_helper_for_list_kb;
use crate::model::handlers::callback::search_cmd::callback_helper_for_search_kb;
use crate::model::handlers::callback::search_videos_in_playlits::callback_helper_for_search_videos_in_playlists_kb;
use crate::model::keyboards::types::Buttons;
use crate::model::keyboards::types::Buttons::*;
use crate::model::utils::HTMLise;
use crate::model::youtube::types::{SearchableItem, YouTubeAccessToken};
use crate::view::types::Sendable;


pub(crate) type ResTriplet = (Option<String>, Vec<SearchableItem>, Option<String>);


pub(crate) async fn get_required_callback_data(callback: &CallbackQuery, dialogue: TheDialogue) 
    -> eyre::Result<(DialogueData, Buttons)>
{
    log::info!(" [:: LOG ::]     @[fn]:[get_required_callback_data] :: [Started]");

    let dialogue_data = get_dialogue_data(&dialogue).await?;
    let callback_data = get_callback_data(callback).await?;
    let keyboard: Buttons = serde_json::from_str(&callback_data)?;

    (dialogue_data, keyboard).in_ok()
}

/// Main `text` handler.
pub(crate) async fn handle_callback(callback: CallbackQuery, dialogue: TheDialogue)
   -> Sendable<String>
{
    log::info!(" [:: LOG ::]     @[fn]:[handlers::handle_callback] :: [Started]");

    let log_prefix = " [:: LOG ::]     @[fn]:[handlers::handle_callback]";

    let res = get_required_callback_data(&callback, dialogue).await;
    let (d_data, buttons) =
        match res.pass_err_with(|e| log::error!("{log_prefix}{e:?}"))
        {
            Ok(ok) => ok,
            Err(_) => return Sendable::SendError("⚠ Internal error ⚠".to_owned())
        };
    let res =
        match &buttons
        {
            SearchButtons(search_kb) => callback_helper_for_search_kb(search_kb, d_data, callback).await,
            ListButtons(list_kb) => callback_helper_for_list_kb(list_kb, d_data, callback).await,
            SearchVideosInMyPlaylistsButtons(search_video_oin_pls_kb) =>
                callback_helper_for_search_videos_in_playlists_kb(search_video_oin_pls_kb, d_data, callback).await
        };
    res.map_err(Sendable::SendError).merge_ok_err()
}

pub(crate) async fn construct_login_url(user_id: &str) -> eyre::Result<String>
{
    let auth_url = default_auth_url(user_id).await?.to_link("Log In");
    format!("Use this link to log in {auth_url} \nPlease, log in and send your text again").in_ok()
}

pub(crate) async fn update_and_return_access_token(user_id: &str, token: YouTubeAccessToken, db_url: &str) 
    -> eyre::Result<String>
{
    let secret = read_application_secret(env!("PATH_TO_GOOGLE_OAUTH_SECRET")).await?;
    let token_req = build_refresh_access_token_req(secret, &token)?;
    refresh_access_token(user_id, token, db_url, token_req).await?.access_token.in_ok()
}


