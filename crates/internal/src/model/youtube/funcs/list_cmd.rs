use crate::model::keyboards::types::Sorting;
use crate::model::net::traits::{YouTubeApiRequestBuilder, YouTubeApiResponsePage};
use crate::model::youtube::funcs::common::{items_request, pagination};
use crate::model::youtube::traits::{IntoSearchableItem, Searchable};
use crate::model::youtube::types::SearchableItem;
use std::fmt::Debug;

#[allow(clippy::unwrap_used)]
/// Returns all items on user's channel.
pub(crate) async fn list_items<T>(
    req_builder: T,
    access_token: &str,
    sorting: &Sorting,
    res_limit: u32,
) -> Vec<SearchableItem>
where
    T: YouTubeApiRequestBuilder,
    T::Target: Default + Debug + YouTubeApiResponsePage,
{
    log::info!(" [:: LOG ::]    ( @:[fn::list_items] started )");

    let client = reqwest::Client::new();
    let resp = items_request(&client, access_token, &req_builder, None).await;
    let search_res = resp.unwrap_or_default();

    let cap = (search_res.total_results().unwrap()).min(res_limit) as usize;
    let mut store_in = Vec::with_capacity(cap);
    let current_cap = std::sync::Arc::new(std::sync::Mutex::new(store_in.len()));

    let stop_if = |_: &T::Target| *current_cap.lock().unwrap() > res_limit as usize;
    let f = |item: T::Target| {
        if let Some(mut i) = item.items() {
            store_in.append(&mut i)
        }
        *current_cap.lock().unwrap() = store_in.len()
    };

    pagination(req_builder, access_token, stop_if, f).await;

    log::info!(" [:: LOG ::]    ( @:[fn::list_items] ended )");

    match *sorting {
        Sorting::Alphabetical => store_in.sort_by(|a, b| Ord::cmp(&a.title(), &b.title())),
        Sorting::Date => store_in.sort_by(|a, b| Ord::cmp(&a.date(), &b.date())),
    }
    store_in
        .into_iter()
        .take(res_limit as usize)
        .map(IntoSearchableItem::into_item)
        .collect()
}
