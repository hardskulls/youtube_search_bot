use google_youtube3::oauth2::read_application_secret;
use maptypings::WrapInRes;
use teloxide::types::{CallbackQuery, Message};
use url::Url;

use crate::model::dialogue::types::{
    DialogueData, Either, ListCommandSettings, SearchCommandSettings,
    SearchVideosInPlaylistsCommandSettings, State, TheDialogue,
};
use crate::model::errors::{DialogueStateStorageError, NoCallbackDataError, NoTextError};
use crate::model::net::types::{QUERY_SEPARATOR, STATE_CODE};
use crate::model::youtube::funcs::common::make_auth_url;
use crate::model::youtube::types::{ACCESS_TYPE, RESPONSE_TYPE, SCOPE_YOUTUBE_READONLY};
use crate::StdRes;

pub(crate) fn search_settings_update_or_default(d_state: State) -> SearchCommandSettings {
    if let State::SearchCommandActive(search_settings) = d_state {
        search_settings
    } else {
        SearchCommandSettings::default()
    }
}

pub(crate) fn list_settings_update_or_default(d_state: State) -> ListCommandSettings {
    if let State::ListCommandActive(list_settings) = d_state {
        list_settings
    } else {
        ListCommandSettings::default()
    }
}

pub(crate) fn search_videos_in_playlists_update_or_default(
    d_state: State,
) -> SearchVideosInPlaylistsCommandSettings {
    if let State::SearchVideosInPlaylistsCommandActive(search_videos_in_playlists_settings) =
        d_state
    {
        search_videos_in_playlists_settings
    } else {
        SearchVideosInPlaylistsCommandSettings::default()
    }
}

/// Get `state` from dialogue.
#[inline]
pub(crate) async fn get_dialogue_data(
    dialogue: &TheDialogue,
) -> Result<DialogueData, DialogueStateStorageError> {
    dialogue
        .get()
        .await
        .map_err(|_| DialogueStateStorageError)?
        .ok_or(DialogueStateStorageError)
}

/// Get `text` from message.
#[inline]
pub(crate) async fn get_text(msg: &Message) -> StdRes<&str, NoTextError> {
    msg.text().ok_or(NoTextError)
}

/// Get `callback` data as a `String`.
#[inline]
pub(crate) async fn get_callback_data(
    callback: &CallbackQuery,
) -> StdRes<String, NoCallbackDataError> {
    callback.data.clone().ok_or(NoCallbackDataError)
}

/// Construct authorization url.
pub(crate) async fn default_auth_url(user_id: &str) -> eyre::Result<Url> {
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
    let secret = read_application_secret(secret_path).await?;

    let (client_id, redirect_uri) = (secret.client_id.as_str(), secret.redirect_uris[0].as_str());
    let (scope, response_type) = (&[SCOPE_YOUTUBE_READONLY], RESPONSE_TYPE);
    let state = format!("for_user={user_id}{QUERY_SEPARATOR}state_code={STATE_CODE}");
    let optional_params = &[
        ("ACCESS_TYPE".to_owned().to_lowercase(), ACCESS_TYPE),
        ("state".to_owned(), state.as_str()),
    ];

    make_auth_url(
        client_id,
        redirect_uri,
        response_type,
        scope,
        optional_params,
    )?
    .in_ok()
}

/// Helper function used for `handle_text` handler.
/// Parses user input as number in order to set it as `result limit` setting.
pub(crate) fn parse_number(
    text: &str,
    configs: Either<&SearchCommandSettings, &ListCommandSettings>,
    dialogue_data: &DialogueData,
) -> (&'static str, Option<DialogueData>) {
    log::info!(" [:: LOG ::]     @[fn]:[parse_number] :: [Started]");
    match text.parse::<u16>() {
        Ok(num) if num >= 1 => (
            "Accepted! ✅",
            Some(DialogueData {
                state: save_res_limit(configs, num),
                ..dialogue_data.clone()
            }),
        ),
        _ => ("Send a number greater than 0", None),
    }
}

fn save_res_limit(
    configs: Either<&SearchCommandSettings, &ListCommandSettings>,
    num: u16,
) -> State {
    use crate::model::dialogue::types::State::{ListCommandActive, SearchCommandActive};
    let result_limit = Some(u32::from(num));
    match configs {
        Either::First(search_settings) => SearchCommandActive(SearchCommandSettings {
            result_limit,
            ..search_settings.clone()
        }),
        Either::Last(list_settings) => ListCommandActive(ListCommandSettings {
            result_limit,
            ..list_settings.clone()
        }),
    }
}

/// Save text to search.
pub(crate) fn save_text(
    text: &str,
    search_settings: SearchCommandSettings,
    dialogue_data: &DialogueData,
) -> (&'static str, Option<DialogueData>) {
    log::info!(" [:: LOG ::]     @[fn]:[save_text] :: [Started]");
    let state = State::SearchCommandActive(SearchCommandSettings {
        text_to_search: text.to_owned().into(),
        ..search_settings
    });
    (
        "Accepted! ✅",
        Some(DialogueData {
            state,
            ..dialogue_data.clone()
        }),
    )
}
