use google_youtube3::api::SubscriptionListResponse;

use crate::mods::net::join;
use crate::mods::youtube::types::{AUTH_URL_BASE, RequiredAuthURLParams};
use crate::StdResult;

pub(crate) mod types;

pub async fn list_subscriptions(client: &reqwest::Client, next_page_tok: Option<String>, access_token: &str)
    -> eyre::Result<SubscriptionListResponse>
{
    let mut req =
        client
            .get(reqwest::Url::parse("https://www.googleapis.com/youtube/v3/subscriptions")?)
            .query(&[("part", "snippet,contentDetails"), ("maxResults", "50"), ("mine", "true")])
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", access_token))
            .header(reqwest::header::ACCEPT, "application/json");
    if let Some(page) = next_page_tok
    { req = req.query(&[("pageToken", &page)]) }
    
    let resp = req.send().await?;
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'resp' is [| '{:#?}' |] )", (&resp.headers(), &resp.status()));
    if !resp.status().is_success()
    { return Err(eyre::eyre!("status code is not a success")) }
    
    let subscr_list_resp = resp.json().await?;
    log::info!(" [:: LOG ::]    ( @:[fn::list_subscriptions] 'subscr_list_resp' is [| '{:#?}' |] )", subscr_list_resp);
    Ok(subscr_list_resp)
}

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

#[cfg(test)]
mod tests
{
    use google_youtube3::oauth2::ApplicationSecret;
    use google_youtube3::oauth2::read_application_secret;
    
    use crate::mods::net::query_pairs;
    use crate::mods::youtube::make_auth_url;
    
    const URL_1 : &str =
        "\
        https://accounts.google.com/o/oauth2/auth?\
        scope=https://www.googleapis.com/auth/youtube%20https://www.googleapis.com/auth/youtube.readonly&\
        access_type=offline&\
        redirect_uri=http://127.0.0.1:62320&\
        response_type=code&\
        client_id=799749940076-oktc5l1861j0ilnp3jndb9elrk38krus.apps.googleusercontent.com\
        ";
    
    #[tokio::test]
    async fn make_auth_url_test() -> eyre::Result<()>
    {
        let secret =
            read_application_secret("C:/Users/Bender/Documents/Code/MyCode/Current/youtube_search_bot/crates/secret.json").await.unwrap();
        let (client_id, redirect_uri, response_type) = (secret.client_id, secret.redirect_uris[0].clone(), "code".into());
        let scopes = ["https://www.googleapis.com/auth/youtube".to_owned(), "https://www.googleapis.com/auth/youtube.readonly".to_owned()];
        let opt_params = [("access_type".to_owned(), "offline".to_owned())];
        let url = make_auth_url(client_id, redirect_uri, response_type, &scopes, &opt_params).unwrap();
        let uri_1 = url.as_str().parse::<axum::http::Uri>().unwrap();
        let uri_2 = URL_1.parse::<axum::http::Uri>().unwrap();
        let mut val_1: Vec<_> = query_pairs(uri_1.query().unwrap_or(""), "&")?.collect();
        val_1.sort();
        let mut val_2: Vec<_> = query_pairs(uri_2.query().unwrap_or(""), "&")?.collect();
        val_2.sort();
        for (i, x) in val_1.iter().enumerate()
        {
            assert_eq!(x, &val_2[i]);
        }
        Ok(())
    }
    
    #[tokio::test]
    async fn test_make_url()
    {
        let secret: ApplicationSecret = read_application_secret("client_secret_web_app.json").await.unwrap();
        let (response_type, scope) = ("code".to_owned(), ["https://www.googleapis.com/auth/youtube".to_owned()]);
        let url = make_auth_url(secret.client_id, secret.redirect_uris[0].clone(), response_type, &scope, &[]);
        assert!(matches!(url, Ok(_))) ;
        dbg!(&url) ;
        println!("{}", url.as_ref().unwrap()) ;
    }
}


