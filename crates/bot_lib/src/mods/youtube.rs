use std::fmt::Debug;
use error_traits::InOk;
use crate::mods::inline_keyboards::types::SearchIn;

use crate::mods::net::join;
use crate::mods::net::traits::{ItemsListRequestBuilder, ItemsResponsePage};
use crate::mods::youtube::traits::Searchable;
use crate::mods::youtube::types::{AUTH_URL_BASE, RequiredAuthURLParams};
use crate::StdResult;

pub(crate) mod types;
pub(crate) mod traits;

/// Makes a single call to `YouTube API` go get one page of items.
pub(crate) async fn req_items<T>(client: &reqwest::Client, access_token: &str, item_to_search: &T, page_token: Option<String>)
    -> eyre::Result<T::Target>
    where
        T: ItemsListRequestBuilder
{
    let req_builder = item_to_search.build_req(client, access_token, page_token)?;
    let resp = req_builder.send().await?;
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'resp' is [| '{:#?}' |] )", (&resp.headers(), &resp.status()));
    if !resp.status().is_success()
    { return Err(eyre::eyre!("status code is not a success")) }
    
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'resp' is [| '{:#?}' |] )", (&resp.headers(), &resp.status()));
    resp.json::<T::Target>().await?.in_ok()
}

/// Authorization url constructor.
pub(crate) fn make_auth_url<V>(client_id: V, redirect_uri: V, response_type: V, scope: &[V], optional_params: &[(String, V)])
    -> StdResult<url::Url, url::ParseError>
    where
        V: AsRef<str> + Clone, /* K: AsRef<str>, I: IntoIterator, I::Item: std::borrow::Borrow<(K, V)> */
{
    let keys = (RequiredAuthURLParams::ClientId, RequiredAuthURLParams::RedirectUri, RequiredAuthURLParams::ResponseType);
    let required_params = [(keys.0.to_string(), client_id), (keys.1.to_string(), redirect_uri), (keys.2.to_string(), response_type)];
    let params = [&required_params[..], optional_params].concat();
    let mut url: url::Url = url::Url::parse_with_params(AUTH_URL_BASE, &params)?;
    let (scope_key, scope_list) = (RequiredAuthURLParams::Scope.to_string(), join(scope, ","));
    url.query_pairs_mut().append_pair(&scope_key, &scope_list);
    Ok(url)
}

/// Search and filter items (subscriptions, playlists, etc).
pub(crate) async fn search_items<I>
(
    search_mode: &SearchIn,
    request_builder: I,
    text_to_look_for: &str,
    access_token: &str,
    max_res: u32
)
    -> Vec<<I::Target as ItemsResponsePage>::Item>
    where
        I: ItemsListRequestBuilder,
        I::Target: Default + Debug + ItemsResponsePage
{
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] started )");
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] INPUT is [ '{:?}' ] )", (&search_mode, &text_to_look_for, &max_res));
    let mut store_in = vec![];
    let mut current_cap = store_in.len();
    
    let stop_if = move |_: &_| current_cap > max_res as usize;
    let f =
        |item: I::Target|
            {
                if let Some(i) = item.items()
                { find_matches(search_mode, &mut store_in, i, text_to_look_for) }
                current_cap = store_in.len()
            };
    pagination(request_builder, access_token, stop_if, f).await;
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] 'current_cap' is [ '{:?}' ] )", current_cap);
    log::info!(" [:: LOG ::]    ( @:[fn::search_items] ended )");
    store_in
}

/// Returns all items on user's channel.
pub(crate) async fn list_items<I>
(
    request_builder: I,
    access_token: &str,
) 
    -> Vec<<I::Target as ItemsResponsePage>::Item>
    where
        I: ItemsListRequestBuilder,
        I::Target: Default + Debug + ItemsResponsePage,
{
    log::info!(" [:: LOG ::]    ( @:[fn::list_items] started )");
    
    let client = reqwest::Client::new();
    let resp = req_items(&client, access_token, &request_builder, None).await;
    let search_res = resp.unwrap_or_default();
    let cap = search_res.total_results().unwrap_or(50) as usize;
    
    let mut store_in = Vec::<<I::Target as ItemsResponsePage>::Item>::with_capacity(cap);
    let stop_if = |_: &_| false;
    let f =
        |item: I::Target|
            {
                if let Some(mut i) = item.items()
                { store_in.append(&mut i) }
            };
    
    pagination(request_builder, access_token, stop_if, f).await;
    log::info!(" [:: LOG ::]    ( @:[fn::list_items] ended )");
    store_in
}

/// Gives full access to each page of request.
/// Stop condition can be set using `stop_if`.
pub(crate) async fn pagination<I, F, S>(request_builder: I, access_token: &str, stop_if: S, f: F)
    where
        I: ItemsListRequestBuilder,
        I::Target: Default + Debug + ItemsResponsePage,
        F: FnMut(I::Target),
        S: Fn(&I::Target) -> bool,
{
    log::info!(" [:: LOG ::]    ( @:[fn::pagination] started )");
    let mut f = f;
    let client = reqwest::Client::new();
    
    let mut next_page_token = None;
    loop
    {
        let resp = req_items(&client, access_token, &request_builder, next_page_token).await;
        let search_res = resp.unwrap_or_default();
        next_page_token = search_res.next_page_token();
    
        if stop_if(&search_res)
        { break }
        
        f(search_res);
        
        if next_page_token.is_none()
        { break }
    }
    log::info!(" [:: LOG ::]    ( @:[fn::pagination] ended )");
}

/// Find matches in a list of subscriptions.
fn find_matches<S>(search_mode: &SearchIn, store_in: &mut Vec<S>, search_in: Vec<S>, text_to_look_for: &str)
    where
        S: Searchable
{
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] started )");
    let text_to_search = text_to_look_for.to_lowercase();
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'text_to_search' is [| '{:#?}' |] )", (&text_to_search));
    for item in search_in
    {
        let compare_by = match *search_mode { SearchIn::Title => item.title(), SearchIn::Description => item.description() };
        log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'compare_by' is [| '{:#?}' |] )", (&compare_by));
        if let Some(title_or_descr) = compare_by
        { if title_or_descr.to_lowercase().contains(&text_to_search) { store_in.push(item) } }
    }
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'store_in.len()' is [| '{:#?}' |] )", (&store_in.len()));
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] ended )");
}

#[cfg(test)]
mod tests
{
    use crate::mods::net::traits::ListSubscriptions;
    use super::*;
    
    #[tokio::test]
    async fn trythat()
    {
        let access_token = "kmkpmpmp";
        //let mut v = vec![];
        let f = |x| drop(x);
        pagination(ListSubscriptions, access_token, |_| false, f).await;
    }
    
}


