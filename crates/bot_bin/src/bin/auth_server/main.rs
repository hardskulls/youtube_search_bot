use std::{collections::HashMap};
use axum::
{
    body::Body,
    http::Request,
    Json,
    Router,
    extract::{Path, Query},
    headers::HeaderMap,
    routing::{get, post}
};
use redis::Commands;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use yup_oauth2::read_application_secret;

use bot_lib::net::url::find_by_key;

#[tokio::main]
async fn main() -> eyre::Result<()>
{
    // !! All `logs::info!` work only after this line !!
    simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info))?;

    // build our application with a single route
    let app =
        Router::new()
            .route("/google_callback_auth_code", get(handle_auth_code))
            .route("/google_callback_access_token", post(handle_access_token))
            .route("/bot_access_token_req", post(handle_bot_access_token_req));

    // run it with hyper on localhost:8443
    axum::Server::bind(&"0.0.0.0:8443".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn params(state: &str, for_user: &str, auth_code: &str) -> Vec<(String, String)>
{
    let secret = read_application_secret("client_secret_web_client_for_youtube_search_bot").await.unwrap();
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
    log::info!("'handle_auth_code' started");
    log::info!("{:#?}", req);
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
            .header("POST", "/token HTTP/1.1")
            .header("Host:", "oauth2.googleapis.com")
            .header("Content-Type:", "application/x-www-form-urlencoded")
            .body(Body::empty())
            .unwrap();
    let hyper_client: hyper::Client<_> = hyper::Client::new();
    let r = hyper_client.request(request).await;
    log::info!("{:#?}", r);
    log::info!("'handle_auth_code' finished");
}

async fn handle_bot_access_token_req()
{

}

// Something like `async fn handle_access_token(Json(payload): Json<serde_json::Value>) { ... }`
async fn handle_access_token
(
    Path(user_id): Path<u32>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(access_token): Json<YouTubeAccessToken>
)
{
    log::info!("'handle_access_token' started");
    log::info!("{:#?}", user_id);
    log::info!("{:#?}", headers);
    let Some(state) = params.get("state") else { return };
    if !state.contains("! insert state code here") { return }
    let Some(for_user) = params.get("for_user") else { return };
    
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();
    let _: () = con.set(for_user, access_token.access_token.as_ref().unwrap()).unwrap();
    log::info!("'handle_access_token' finished");
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

    mod serialization
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
            let path = "C:/Users/Bender/Downloads/test_access_token_deserialization.json";
            let contents = std::fs::read_to_string(path).unwrap();
            let deserialized_2 = serde_json::from_str::<YouTubeAccessToken>(&contents);
            assert!(matches!(deserialized_2, Ok(_)));
        }
    }
}


