
use error_traits::{PassErrWith, WrapInRes};
use google_youtube3::api::PlaylistItemListResponse;

use crate::model::keyboards::types::SearchIn;
use crate::model::net::traits::YouTubeApiResponsePage;
use crate::model::net::types::{PlaylistItemRequester, PlaylistRequester};
use crate::model::youtube::funcs::common::{items_request, pagination};
use crate::model::youtube::traits::{IntoSearchableItem, Searchable};
use crate::model::youtube::types::SearchableItem;


#[allow(clippy::unwrap_used)]
/// Returns all items on user's channel.
pub(crate) async fn search_videos_in_playlists
(
    search_in: &SearchIn,
    search_for: &str,
    access_token: &str,
    res_limit: u32
)
    -> Vec<SearchableItem>
{
    log::info!(" [:: LOG ::]    ( @:[fn::search_videos_in_playlists] started )");
    log::info!(" [:: LOG ::]    ( @:[fn::search_videos_in_playlists] INPUT is [ '{:?}' ] )", (&search_in, &search_for, &res_limit));
    
    let mut store_in: Vec<SearchableItem> = vec![];
    
    let client = reqwest::Client::new();
    
    let mut next_page_token = None;
    loop
    {
        let resp = items_request(&client, access_token, &PlaylistRequester, next_page_token).await;
        let search_res = resp.unwrap_or_default();
        next_page_token = search_res.next_page_token();
        
        log::info!
        (
            "@:[fn::search_videos_in_playlists] <search_res.items.len()> is: {:#?}",
            search_res.items.as_ref().map(|i| i.len()).unwrap_or(0)
        );
        
        for playlist in search_res.items.into_iter().flatten()
        {
            log::info!("@:[fn::search_videos_in_playlists] <playlist> is: {playlist:#?}");
            
            let pl_title = playlist.title().unwrap_or("");
            let pl_id = playlist.id.as_deref().unwrap_or("");
            let search_in = search_in.clone();
            let mut res = find_videos_in_playlist(pl_title, pl_id, search_in, search_for, access_token).await;
            store_in.append(&mut res);
        }
        
        if next_page_token.is_none()
        { break }
    }
    
    log::info!(" [:: LOG ::]    ( @:[fn::search_videos_in_playlists] ended )");
    
    store_in.into_iter()
        .take(res_limit as usize)
        .collect()
}

async fn find_videos_in_playlist(pl_title: &str, pl_id: &str, search_in: SearchIn, search_for: &str, access_token: &str)
    -> Vec<SearchableItem>
{
    log::info!(" [:: LOG ::]    ( @:[fn::find_videos_in_playlist] started )");
    
    find_videos_in_playlist_helper(pl_title, pl_id, search_in, search_for, access_token)
        .await
        .pass_err_with(|e| log::error!("{e:?}"))
        .unwrap_or_default()
}

async fn find_videos_in_playlist_helper
(
    pl_title: &str,
    pl_id: &str,
    search_in: SearchIn,
    search_for: &str,
    access_token: &str
)
    -> eyre::Result<Vec<SearchableItem>>
{
    log::info!(" [:: LOG ::]    ( @:[fn::find_videos_in_playlist_helper] started )");
    
    let mut store_in: Vec<SearchableItem> = vec![];
    let text_to_search = search_for.to_lowercase();
    
    let stop_if = |_: &_| false;
    let f =
        |search_target: PlaylistItemListResponse|
            for i in search_target.items().into_iter().flatten()
            {
                let compare_by =
                    match search_in
                    {
                        SearchIn::Title => i.title(),
                        SearchIn::Description => i.description()
                    };
                if let Some(compare_by) = compare_by
                {
                    if compare_by.to_lowercase().contains(&text_to_search)
                    { store_in.push(i.into_item()) }
                }
            };
    pagination(PlaylistItemRequester { playlist_id: pl_id }, access_token, stop_if, f).await;
    
    for i in store_in.iter_mut()
    {
        let opt_about = i.about.as_mut();
        if let Some(about) = opt_about
        { *about = about.replace("Playlist: \n\n", &format!("Playlist: \n\ntitle: {pl_title} \n\n")); }
    }
    
    store_in.in_ok()
}


