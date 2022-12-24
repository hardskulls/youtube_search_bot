use axum::http::Request;
use google_youtube3::oauth2::read_application_secret;
use hyper::Body;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::mods::db::set_access_token;
use crate::net::url::find_by_key;
use crate::StdResult;

async fn params(auth_code: &str) -> [(String, String); 5]
{
    let secret_path = std::env::var("OAUTH_SECRET_PATH").unwrap();
    let secret = read_application_secret(secret_path).await.unwrap();
    [
        ("client_id".to_owned(), secret.client_id),
        ("client_secret".to_owned(), secret.client_secret),
        ("code".to_owned(), auth_code.to_owned()),
        ("grant_type".to_owned(), "authorization_code".to_owned()),
        ("redirect_uri".to_owned(), secret.redirect_uris[0].clone())
    ]
}

async fn access_token_req(auth_code: &str) -> RequestBuilder
{
    let params = params(auth_code).await;
    let uri = reqwest::Url::parse_with_params("https://oauth2.googleapis.com/token", &params).unwrap();
    reqwest::Client::new()
        .post(reqwest::Url::parse("https://oauth2.googleapis.com/token").unwrap())
        .header(hyper::header::LOCATION, "https://t.me/test_echo_123_456_bot")
        .header(hyper::header::HOST, "oauth2.googleapis.com")
        .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(uri.query().unwrap().to_owned())
}

pub async fn handle_auth_code(req: Request<Body>) -> StdResult<&'static str, &'static str>
{
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] started [ OK ] )");
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'req' is [| '{:#?}' |]", format!("{:#?}", &req));
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'req.body()' is [| '{:#?}' |]", format!("{:#?}", &req.body()));
    
    let url_encoded_query = req.uri().query().unwrap_or("");
    let decoded_query: String =
        form_urlencoded::parse(url_encoded_query.as_bytes())
            .map(|(k, v)| [&k, "=", &v, "&"].concat())
            .collect();
    let state = find_by_key(&decoded_query, "&", "state").map_err(|_| "state not found")?;
    
    let state_code = find_by_key(state, "xplusx", "state_code").map_err(|_| "state code not found")?;
    if !state_code.contains("liuhw9p38y08q302q02h0gp9g0p2923924u0s") { return Err("state codes don't match") }
    let for_user = find_by_key(state, "xplusx", "for_user").map_err(|_| "for_user not found")?;
    
    let auth_code = find_by_key(&decoded_query, "&", "code").map_err(|_| "auth_code not found")?;
    
    let tok_req = access_token_req(auth_code).await;
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'tok_req' is [| '{:#?}' |]", format!("{:#?}", &tok_req));
    let resp = tok_req.send().await.map_err(|_| "access token request failed")?;
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'resp' is [| '{:#?}' |]", format!("{:#?}", &resp));
    
    let access_token = resp.json::<YouTubeAccessToken>().await.map_err(|_| "couldn't deserialize access token")?;
    set_access_token(for_user, &access_token.access_token.unwrap()).map_err(|_| "db error")?;
    
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] finished [ OK ] )");
    Ok("success")
}

pub async fn serve_all(req: Request<Body>) -> &'static str
{
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] started [ OK ] )");
    let (p, b) = req.into_parts();
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] 'p' is [| '{:#?}' |]", format!("{:#?}", &p));
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] 'b' is [| '{:#?}' |]", format!("{:#?}", &b));
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] finished [ OK ] )");
    "server is up"
}

/// Represents a `token` as returned by `OAuth2` servers.
///
/// It is produced by all authentication flows.
/// It authenticates certain operations, and must be refreshed once it reached it's expiry date.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct YouTubeAccessToken
{
    /// used when authorizing calls to `oauth2` enabled services.
    pub access_token: Option<String>,
    /// used to refresh an expired `access_token`.
    pub refresh_token: Option<String>,
    /// The time when the `token` expires.
    pub expires_at: Option<OffsetDateTime>,
    /// Optionally included by the `OAuth2` server and may contain information to verify the identity
    /// used to obtain the `access token`.
    /// Specifically `Google API:s` include this if the additional scopes `email` and/or `profile`
    /// are used. In that case the content is an `JWT token`.
    pub id_token: Option<String>,
}

#[cfg(test)]
mod tests
{
    use super::*;
    
    mod serialization_testing
    {
        use super::*;
        
        #[test]
        fn serialize_deserialize_string_test()
        {
            let (access_token, refresh_token, expires_at, id_token) =
                (Some("access_token".to_owned()), Some("refresh_token".to_owned()), Some(OffsetDateTime::now_utc()), Some("id_token".to_owned()));
            let token = YouTubeAccessToken { access_token, refresh_token, expires_at, id_token };
            let serialized = serde_json::to_string(&token).unwrap();
            dbg!(&serialized);
            let deserialized = serde_json::from_str::<YouTubeAccessToken>(&serialized).unwrap();
            assert_eq!(token, deserialized);
        }
        
        #[test]
        fn deserialize_from_json_test()
        {
            let path = "test_access_token_deserialization.json";
            let contents = std::fs::read_to_string(path).unwrap();
            let deserialized_2 = serde_json::from_str::<YouTubeAccessToken>(&contents);
            assert!(matches!(deserialized_2, Ok(_)));
        }
    }
    
    mod requests_testing
    {
        use std::net::SocketAddr;
        use super::*;
    
        #[tokio::test]
        async fn print_request_contents_test()
        {
            let (_state, _for_user, auth_code) = ("this_is_state", "this_is_for_user", "this_is_auth_code");
            let params = params(auth_code).await;
            let uri = reqwest::Url::parse_with_params("https://oauth2.googleapis.com/token", &params).unwrap();
            let request =
                hyper::Request::builder()
                    .uri(uri.as_str())
                    .method(hyper::Method::POST)
                    .header(hyper::header::LOCATION, "https://t.me/test_echo_123_456_bot")
                    .header(hyper::header::HOST, "oauth2.googleapis.com")
                    .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::empty())
                    .unwrap();
            println!(" [:: LOG ::] ... : ( 'request' of type '{}' is [< {:#?} >]", std::any::type_name::<Request<Body>>(), request);
            assert_eq!(request.method(), hyper::Method::POST)
        }
    
        #[test]
        fn create_port_test()
        {
            let socket_addr: Result<SocketAddr, _> = "0.0.0.0:443".parse();
            assert!(matches!(socket_addr, Ok(_)));
            let socket_addr: Result<SocketAddr, _> = "0.0.0.0".parse();
            assert!(matches!(socket_addr, Err(_)));
        }
        
        #[tokio::test]
        async fn access_token_request_test()
        {
            let auth_code = "4/tfi76r7r7uruydyt";
            let params = params(auth_code).await;
            dbg!(&params);
            let uri = reqwest::Url::parse_with_params("https://oauth2.googleapis.com/token", &params).unwrap();
            dbg!(&uri);
            let r =
                reqwest::Client::new()
                    .post(uri)
                    .header(hyper::header::LOCATION, "https://t.me/test_echo_123_456_bot")
                    .header(hyper::header::HOST, "oauth2.googleapis.com")
                    .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded");
            dbg!(&r);
            log::info!(" [:: LOG ::] ... : ( 'r' of type '{}' is [< {:#?} >]", std::any::type_name::<Request<Body>>(), r);
            assert!(!auth_code.contains("hjgjgjg"));
        }
    }
}


