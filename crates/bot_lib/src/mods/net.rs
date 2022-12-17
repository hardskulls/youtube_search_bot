use std::collections::HashMap;
use axum::{headers::HeaderMap, Json, Router, http::Request};
use axum::extract::{Path, Query};
use axum::routing::{get, post};
use google_youtube3::oauth2::read_application_secret;
use hyper::Body;
use redis::Commands;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::net::url::find_by_key;

pub async fn start_auth_server() -> eyre::Result<()>
{
    log::info!(" [:: LOG ::] ... : ( ⚙ <| Building 'auth_server'... |> ⚙ )");
    // build our application with a single route
    let app =
        Router::new()
            .route("/google_callback_auth_code", get(handle_auth_code))
            .route("/google_callback_access_token", post(handle_access_token))
            .route("/bot_access_token_req", post(handle_bot_access_token_req));
    
    // run it with hyper on localhost:8443
    //let port = std::env::var("PORT_AUTH_SERVER")?.parse::<u16>()?;
    let addr = std::env::var("LOCAL_ADDR")?;
    let ports = [80, 443, 88, 8181, 8080];
    for p in ports.into_iter()
    {
        let res =
            axum::Server::bind(&format!("{addr}:{p}").parse()?)
                .serve(app.clone().into_make_service())
                .await;
        
        if res.is_err()
        { log::info!(" [:: LOG ::] ... : ( 🚧 <| 'res' is {res:#?} |> 🚧 )"); }
        else
        {
            log::info!(" [:: LOG ::] ... : ( 🚀 'auth_server' started 🚀 )");
            break
        }
    }
    
    log::info!(" [:: LOG ::] ... : ( <| 'auth_server' finished |> ❌ )");
    Ok(())
}

async fn params(state: &str, for_user: &str, auth_code: &str) -> Vec<(String, String)>
{
    let secret_path = std::env::var("OAUTH_SECRET_PATH").unwrap();
    let secret = read_application_secret(secret_path).await.unwrap();
    let arr: [(&str, &str); 7] =
        [
            ("client_id", &secret.client_id),
            ("client_secret", &secret.client_secret),
            ("code", auth_code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", secret.redirect_uris[1].as_str()),
            ("state", state),
            ("for_user", for_user)
        ];
    arr.into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}

async fn handle_auth_code(req: Request<Body>)
{
    log::info!(" [:: LOG ::] ... : ( 'handle_auth_code' started )");
    log::info!(" [:: LOG ::] ... : ( 'req' of type '{}' is [< {:#?} >]", std::any::type_name::<Request<Body>>(), req);
    let query_as_str = req.uri().query().unwrap_or("");
    let Ok(state) = find_by_key(query_as_str, "state") else { return };
    if !state.contains("! insert state code here") { return }
    let Ok(for_user) = find_by_key(state, "for_user") else { return };
    let Ok(auth_code) = find_by_key(query_as_str, "code") else { return };
    
    let params = params(state, for_user, auth_code).await;
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
    log::info!(" [:: LOG ::] ... : ( 'request' of type '{}' is [< {:#?} >]", std::any::type_name::<Request<Body>>(), request);
    let hyper_client: hyper::Client<_> = hyper::Client::new();
    let r = hyper_client.request(request).await;
    log::info!(" [:: LOG ::] ... : ( 'r' of type '{}' is [< {:#?} >]", std::any::type_name::<hyper::Result<hyper::Response<Body>>>(), r);
    log::info!(" [:: LOG ::] ... : ( 'handle_auth_code' finished )");
}

async fn handle_access_token
(
    Path(user_id): Path<u32>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(access_token): Json<YouTubeAccessToken>
)
{
    log::info!(" [:: LOG ::] ... : ( 'handle_access_token' started )");
    log::info!(" [:: LOG ::] ... : ( 'user_id' of type '{}' is [< {:#?} >]", std::any::type_name::<Path<u32>>(), user_id);
    log::info!(" [:: LOG ::] ... : ( 'params' of type '{}' is [< {:#?} >]", std::any::type_name::<Query<HashMap<String, String>>>(), params);
    log::info!(" [:: LOG ::] ... : ( 'headers' of type '{}' is [< {:#?} >]", std::any::type_name::<HeaderMap>(), headers);
    let Some(state) = params.get("state") else { return };
    if !state.contains("! insert state code here") { return }
    let Some(for_user) = params.get("for_user") else { return };
    
    let client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
    let mut con = client.get_connection().unwrap();
    let _: () = con.set(for_user, access_token.access_token.as_ref().unwrap()).unwrap();
    log::info!(" [:: LOG ::] ... : ( 'handle_access_token' finished )");
}

async fn handle_bot_access_token_req()
{

}

/// Represents a `token` as returned by `OAuth2` servers.
///
/// It is produced by all authentication flows.
/// It authenticates certain operations, and must be refreshed once it reached it's expiry date.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct YouTubeAccessToken {
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
            let (state, for_user, auth_code) = ("this_is_state", "this_is_for_user", "this_is_auth_code");
            let params = params(state, for_user, auth_code).await;
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
    }
}


