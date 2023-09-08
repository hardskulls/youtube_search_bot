
use std::fmt::Debug;

use error_traits::WrapInRes;

use crate::model::net::funcs::join;
use crate::model::net::traits::{YouTubeApiRequestBuilder, YouTubeApiResponsePage};
use crate::model::youtube::types::{AUTH_URL_BASE, RequiredAuthURLParams};
use crate::StdResult;


/// Makes a single call to `YouTube API` go get one page of items.
pub(crate) async fn items_request<T>
(
    client: &reqwest::Client,
    access_token: &str,
    req_builder: &T,
    page_token: Option<String>
)
    -> eyre::Result<T::Target>
    where
        T: YouTubeApiRequestBuilder
{
    let resp =
        req_builder.build_req(client, access_token, page_token)?
            .send()
            .await?;
    
    let f = |s| format!(" [:: LOG ::]    ( @:[fn::items_request] 'resp' is [| '{s:#?}' |] )");
    log::debug!("{}", f((&resp.headers(), &resp.status())));
    
    let status_is_failure = !resp.status().is_success();
    let body = resp.text().await;
    
    log::debug!("@:[fn::items_request] <body> is: {body:#?}");
    
    if status_is_failure
    { return eyre::eyre!("status code is not a success").in_err() }

    let body = body?;
    serde_json::from_str::<T::Target>(&body)?.in_ok()
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
        
        log::debug!("@:[fn::pagination] <resp> is: {resp:?}");
        
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
    use std::any::type_name;
    use error_traits::PassErrWith;
    use reqwest::{Client, RequestBuilder};
    use serde::Deserialize;
    use crate::model::net::types::YOUTUBE_PLAYLIST_ITEMS_API;
    use std::io::Write;
    use env_logger::fmt::Formatter;
    use log::Record;
    use super::*;
    
    
    fn format_logs(buf: &mut Formatter, record: &Record) -> std::io::Result<()>
    {
        let file = record.file().unwrap_or("unknown file");
        let line = record.line().unwrap_or(0);
        let utc = chrono::Utc::now().format("DATE ~ %Y/%m/%d || TIME ~ %H:%M:%S");
        let local = chrono::Local::now().format("DATE ~ %Y/%m/%d || TIME ~ %H:%M:%S");
        let (level, args) = (record.level(), record.args());
        let separator = "===================================================================";
        
        writeln!
        (
            buf,
            "\
                \n{separator} \
                \n\nLOG : {level} \
                \n   ->   LOGGED AT ~ {file}:{line} \
                \n   ->   Local({local}) \
                \n   ->   Utc({utc}) \
                \n\n\n{args} \
                \n\n{separator}\n\n\n \
                "
        )
    }
    
    #[test]
    fn option_iter_flatten_test()
    {
        let op: Option<_> = vec![89, 78, 553, 89, 54, 1, 7].into();
        for i in op.iter().flatten()
        {
            println!("it's {i}")
        }
    }
    
    #[test]
    fn test_build_req_with_problematic_pair_test()
    {
        struct TestRequester<'a> { pub playlist_id: &'a str }
        #[derive(Deserialize)]
        struct TestRequesterTarget;
        impl<'a> YouTubeApiRequestBuilder for TestRequester<'a>
        {
            type Target = TestRequesterTarget;
            
            fn build_req(&self, client: &Client, access_token: &str, page_token: Option<String>)
                -> eyre::Result<RequestBuilder>
            {
                log::info!("@:[fn::build_req {}]", type_name::<Self::Target>());
                let mut req =
                    client.get(reqwest::Url::parse(YOUTUBE_PLAYLIST_ITEMS_API)?)
                        .query(&[("part", "contentDetails,id,snippet,status"), ("maxResults", "50")])
                        .query(&[("playlistId", self.playlist_id)])
                        .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                        .header(reqwest::header::ACCEPT, "application/json");
                if let Some(page) = page_token
                { req = req.query(&[("pageToken", &page)]) }
                log::info!("@:[fn::build_req] <req> is: {:?}", req);
                req.in_ok()
            }
        }
        
        // Logger is initiated globally, ..
        // ..so in tests it should be initialized with `.try_init().ok()`
        env_logger::Builder::from_default_env()
            .format(format_logs)
            .try_init()
            .ok();
        
        let client = Client::new();
        let playlist_id = "PLtPKOVLaGpwTaogl68CjhmZmCSBOMWOKc";
        let (access_token, req_builder, page_token) =
            ("gfTHy^75$367Frt%4dHHJytE$#A", TestRequester { playlist_id }, None);
        let _build =
            req_builder.build_req(&client, access_token, page_token)
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap()
                .build()
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap();
    }
}


