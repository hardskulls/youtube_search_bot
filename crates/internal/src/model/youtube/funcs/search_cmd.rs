
use std::fmt::Debug;
use crate::model::keyboards::types::SearchIn;
use crate::model::net::traits::{YouTubeApiRequestBuilder, YouTubeApiResponsePage};
use crate::model::youtube::funcs::common::pagination;
use crate::model::youtube::traits::{IntoSearchableItem, Searchable};
use crate::model::youtube::types::SearchableItem;


#[allow(clippy::unwrap_used)]
/// Search and filter items (subscriptions, playlists, etc).
pub(crate) async fn search_items<T>
(
    search_in: &SearchIn,
    req_builder: T,
    search_for: &str,
    access_token: &str,
    res_limit: u32
)
    -> Vec<SearchableItem>
    where
        T: YouTubeApiRequestBuilder,
        T::Target: Default + Debug + YouTubeApiResponsePage
{
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] started )");
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] INPUT is [ '{:?}' ] )", (&search_in, &search_for, &res_limit));
    
    let mut store_in = vec![];
    let current_cap = std::sync::Arc::new(std::sync::Mutex::new(store_in.len()));
    
    let stop_if = |_: &T::Target| *current_cap.lock().unwrap() > res_limit as usize;
    let f =
        |search_target: T::Target|
            {
                if let Some(items) = search_target.items()
                { find_matches(search_for, search_in, res_limit, items, &mut store_in) }
                *current_cap.lock().unwrap() = store_in.len()
            };
    pagination(req_builder, access_token, stop_if, f).await;
    
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] 'current_cap' is [ '{:?}' ] )", current_cap);
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] ended )");
    
    store_in.into_iter()
        .map(IntoSearchableItem::into_item)
        .collect()
}

/// Find matches in a list of subscriptions.
fn find_matches<S>(search_for: &str, search_in: &SearchIn, res_limit: u32, items: Vec<S>, store_in: &mut Vec<S>)
    where
        S: Searchable
{
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] started )");
    
    let text_to_search = search_for.to_lowercase();
    
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'text_to_search' is [| '{:#?}' |] )", (&text_to_search));
    
    for item in items
    {
        let compare_by =
            match *search_in
            {
                SearchIn::Title => item.title(),
                SearchIn::Description => item.description()
            };
        
        log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'compare_by' is [| '{:#?}' |] )", (&compare_by));
        
        if let Some(title_or_descr) = compare_by
        {
            if store_in.len() < res_limit as usize
                && title_or_descr
                .to_lowercase()
                .contains(&text_to_search)
            { store_in.push(item) }
        }
    }
    
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'store_in.len()' is [| '{:#?}' |] )", (&store_in.len()));
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] ended )");
}


