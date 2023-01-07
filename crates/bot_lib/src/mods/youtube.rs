use std::fmt::Debug;
use error_traits::InOk;
use crate::mods::inline_keyboards::types::SearchMode;

use crate::mods::net::join;
use crate::mods::net::traits::{ItemsListRequestBuilder, ItemsResponsePage};
use crate::mods::youtube::traits::Searchable;
use crate::mods::youtube::types::{AUTH_URL_BASE, RequiredAuthURLParams};
use crate::StdResult;

pub(crate) mod types;
pub(crate) mod traits;

/// Request items (subscriptions, playlists, etc).
pub async fn request_items<T>(client: &reqwest::Client, access_token: &str, item_to_search: &T)
    -> eyre::Result<T::Target>
    where
        T: ItemsListRequestBuilder
{
    let req_builder = item_to_search.build_req(client, access_token, None)?;
    let resp = req_builder.send().await?;
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'resp' is [| '{:#?}' |] )", (&resp.headers(), &resp.status()));
    if !resp.status().is_success()
    { return Err(eyre::eyre!("status code is not a success")) }
    
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'resp' is [| '{:#?}' |] )", (&resp.headers(), &resp.status()));
    resp.json::<T::Target>().await?.in_ok()
}

/// Request items (subscriptions, playlists, etc).
pub async fn request_items_page<T>(client: &reqwest::Client, access_token: &str, item_to_search: &T, next_page_token: String)
    -> eyre::Result<T::Target>
    where
        T: ItemsListRequestBuilder
{
    let req_builder = item_to_search.build_req(client, access_token, next_page_token.into())?;
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

/// Search and filter subscriptions.
pub(crate) async fn search_items<I>
(
    search_mode: &SearchMode,
    request_builder: I,
    text_to_look_for: &str,
    access_token: &str,
    max_res: u32
)
    -> eyre::Result<Vec<<I::Target as ItemsResponsePage>::Item>>
    where
        I: ItemsListRequestBuilder,
        I::Target: Default + Debug + ItemsResponsePage
{
    log::info!(" [:: LOG ::]    ( @:[fn::get_subs_list] started )");
    log::info!(" [:: LOG ::]    ( @:[fn::get_subs_list] INPUT is [| '{:?}' |] )", (&search_mode, &text_to_look_for, &max_res));
    let client = reqwest::Client::new();
    let initial_response =
        request_items(&client, access_token, &request_builder).await.unwrap_or_default();
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] FIRST 'subs_list_resp' is [| '{:?}' |] )", (&initial_response));
    let items_and_pg_token = initial_response.items_search_res();
    
    let mut store_in = vec![];
    
    if let Some(items) = items_and_pg_token.items
    { find_matches(search_mode, &mut store_in, items, text_to_look_for); }
    
    let mut next_page_token = items_and_pg_token.next_page_token;
    while let Some(page) = next_page_token
    {
        let pagination_resp =
            request_items_page(&client, access_token, &request_builder, page).await.unwrap_or_default();
        
        let items_search_res = pagination_resp.items_search_res();
        next_page_token = items_search_res.next_page_token;
        
        if let Some(items) = items_search_res.items
        { find_matches(search_mode, &mut store_in, items, text_to_look_for); }
        
        if store_in.len() >= max_res as usize
        { next_page_token = None }
    }
    log::info!(" [:: LOG ::]    ( @:[fn::get_subs_list] FINAL 'store_in.len()' is [| '{:#?}' |] )", (&store_in.len()));
    log::info!(" [:: LOG ::]    ( @:[fn::get_subs_list] ended )");
    Ok(store_in)
}

/// Find matches in a list of subscriptions.
fn find_matches<S>(search_mode: &SearchMode, store_in: &mut Vec<S>, search_in: Vec<S>, text_to_look_for: &str)
    where
        S: Searchable
{
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] started )");
    let text_to_search = text_to_look_for.to_lowercase();
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'text_to_search' is [| '{:#?}' |] )", (&text_to_search));
    for item in search_in
    {
        let compare_by = match *search_mode { SearchMode::Title => item.title(), SearchMode::Description => item.description() };
        log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'compare_by' is [| '{:#?}' |] )", (&compare_by));
        if let Some(title_or_descr) = compare_by
        { if title_or_descr.to_lowercase().contains(&text_to_search) { store_in.push(item) } }
    }
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] 'store_in.len()' is [| '{:#?}' |] )", (&store_in.len()));
    log::info!(" [:: LOG ::]    ( @:[fn::find_matches] ended )");
}


