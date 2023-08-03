
use std::fmt::Debug;


use error_traits::{WrapInRes};

use crate::model::net::funcs::join;
use crate::model::net::traits::{YouTubeApiRequestBuilder, YouTubeApiResponsePage};

use crate::model::youtube::types::{AUTH_URL_BASE, RequiredAuthURLParams};
use crate::StdResult;


/// Makes a single call to `YouTube API` go get one page of items.
pub(crate) async fn items_request<T>(client: &reqwest::Client, access_token: &str, req_builder: &T, page_token: Option<String>)
    -> eyre::Result<T::Target>
    where
        T: YouTubeApiRequestBuilder
{
    let resp = req_builder.build_req(client, access_token, page_token)?.send().await?;
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'resp' is [| '{:#?}' |] )", (&resp.headers(), &resp.status()));
    if !resp.status().is_success()
    { return eyre::eyre!("status code is not a success").in_err() }

    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'resp' is [| '{:#?}' |] )", (&resp.headers(), &resp.status()));
    resp.json::<T::Target>().await?.in_ok()
}

/// Authorization url constructor.
pub(crate) fn make_auth_url<V>(client_id: V, redirect_uri: V, response_type: V, scope: &[V], optional_params: &[(String, V)])
    -> StdResult<url::Url, url::ParseError>
    where
        V: AsRef<str> + Clone, /* K : AsRef<str>, I : IntoIterator, I::Item : std::borrow::Borrow<(K, V)> */
{
    let keys = (RequiredAuthURLParams::ClientId, RequiredAuthURLParams::RedirectUri, RequiredAuthURLParams::ResponseType);
    let required_params = [(keys.0.to_string(), client_id), (keys.1.to_string(), redirect_uri), (keys.2.to_string(), response_type)];
    let params = [&required_params[..], optional_params].concat();
    let mut url: url::Url = url::Url::parse_with_params(AUTH_URL_BASE, &params)?;
    let (scope_key, scope_list) = (RequiredAuthURLParams::Scope.to_string(), join(scope, ","));
    url.query_pairs_mut().append_pair(&scope_key, &scope_list);
    url.in_ok()
}

/// Gives full access all pages of request, applying 'f' to each page.
/// Stop condition can be set using `stop_if`.
pub(crate) async fn pagination<I, F, S>(req_builder: I, access_token: &str, stop_if: S, f: F)
    where
        I: YouTubeApiRequestBuilder,
        I::Target: Default + Debug + YouTubeApiResponsePage,
        F: FnMut(I::Target),
        S: Fn(&I::Target) -> bool,
{
    log::info!(" [:: LOG ::]    ( @:[fn::pagination] started )");

    let mut f = f;
    let client = reqwest::Client::new();

    let mut next_page_token = None;
    loop
    {
        let resp = items_request(&client, access_token, &req_builder, next_page_token).await;
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

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    
    
    #[test]
    fn option_iter_flatten_test()
    {
        let op: Option<_> = vec![89, 78, 553, 89, 54, 1, 7].into();
        for i in op.iter().flatten()
        {
            println!("it's {i}")
        }
    }
}


