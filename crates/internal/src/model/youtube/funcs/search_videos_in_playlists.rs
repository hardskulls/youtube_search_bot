use error_traits::PassErrWith;
use google_youtube3::api::PlaylistItemListResponse;
use maptypings::WrapInRes;
use std::fmt::Display;
use tokio::task::JoinHandle;

use crate::model::keyboards::types::SearchIn;
use crate::model::net::traits::YouTubeApiResponsePage;
use crate::model::net::types::{PlaylistItemRequester, PlaylistRequester};
use crate::model::youtube::funcs::common::{items_request, pagination};
use crate::model::youtube::traits::{IntoSearchableItem, Searchable};
use crate::model::youtube::types::SearchableItem;

#[allow(clippy::unwrap_used)]
/// Returns all items on user's channel.
pub(crate) async fn search_videos_in_playlists(
    search_in: &SearchIn,
    search_for: &str,
    access_token: &str,
    res_limit: u32,
) -> Vec<SearchableItem> {
    log::info!(" [:: LOG ::]    ( @:[fn::search_videos_in_playlists] started )");

    let mut store_in = vec![];
    let client = reqwest::Client::new();

    let mut next_page_token = None;
    loop {
        let resp = items_request(&client, access_token, &PlaylistRequester, next_page_token).await;
        let search_res = resp.unwrap_or_default();
        next_page_token = search_res.next_page_token();

        let results: Vec<JoinHandle<Vec<SearchableItem>>> = search_res
            .items
            .into_iter()
            .flatten()
            .map(|playlist| {
                let pl_title = playlist.title().unwrap_or_default().to_owned();
                let pl_id = playlist.id.clone().unwrap_or_default();
                let search_in = search_in.clone();
                let (access_token, search_for) = (access_token.to_owned(), search_for.to_owned());
                tokio::spawn(find_videos_in_playlist(
                    pl_title,
                    pl_id,
                    search_in,
                    search_for,
                    access_token,
                ))
            })
            .collect();
        for i in results {
            let log_prefix = "@:[fn::items_request] ";
            let log_err = |e: &_| log::error!("{log_prefix}{e:?}");
            let mut res = i.await.pass_err_with(log_err).unwrap_or_default();
            store_in.append(&mut res);
        }

        if next_page_token.is_none() {
            break;
        }
    }

    log::info!(" [:: LOG ::]    ( @:[fn::search_videos_in_playlists] ended )");

    store_in.into_iter().take(res_limit as usize).collect()
}

async fn find_videos_in_playlist(
    pl_title: impl Display,
    pl_id: impl Display,
    search_in: SearchIn,
    search_for: impl Display,
    access_token: impl Display,
) -> Vec<SearchableItem> {
    log::info!(" [:: LOG ::]    ( @:[fn::find_videos_in_playlist] started )");

    find_videos_in_playlist_helper(pl_title, pl_id, search_in, search_for, access_token)
        .await
        .pass_err_with(|e| log::error!("{e:?}"))
        .unwrap_or_default()
}

async fn find_videos_in_playlist_helper(
    pl_title: impl Display,
    pl_id: impl Display,
    search_in: SearchIn,
    search_for: impl Display,
    access_token: impl Display,
) -> eyre::Result<Vec<SearchableItem>> {
    log::info!(" [:: LOG ::]    ( @:[fn::find_videos_in_playlist_helper] started )");

    let mut store_in: Vec<SearchableItem> = vec![];
    let text_to_search = search_for.to_string().to_lowercase();

    let stop_if = |_: &_| false;
    let f = |search_target: PlaylistItemListResponse| {
        for i in search_target.items.into_iter().flatten() {
            log::info!("@:[fn::find_videos_in_playlist_helper] <playlistItem> is: {i:#?}");

            let compare_by = match search_in {
                SearchIn::Title => i.title(),
                SearchIn::Description => i.description(),
            };
            log::info!("@:[fn::find_videos_in_playlist_helper] <compare_by> is: {compare_by:?}");
            log::info!(
                "@:[fn::find_videos_in_playlist_helper] <text_to_search> is: {text_to_search:?}"
            );
            if let Some(compare_by) = compare_by {
                if compare_by.to_lowercase().contains(&text_to_search) {
                    store_in.push(i.into_item());
                }
            }
        }
    };
    let playlist_id = pl_id.to_string();
    let access_token = access_token.to_string();
    pagination(
        PlaylistItemRequester {
            playlist_id: &playlist_id,
        },
        &access_token,
        stop_if,
        f,
    )
    .await;

    log::info!(
        "@:[fn::find_videos_in_playlist_helper] <store_in.len()> is: {:?}",
        store_in.len()
    );

    for i in &mut store_in {
        log::info!("@:[fn::find_videos_in_playlist_helper] <searchableItem> is: {i:#?}");

        let opt_about = i.about.as_mut();
        if let Some(about) = opt_about {
            *about = about.replace("[Playlist]\n", &format!("[Playlist]\n \n{pl_title}"));
        }
    }

    store_in.in_ok()
}
