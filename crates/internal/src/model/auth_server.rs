use axum::body::Body;
use axum::body::BoxBody;
use axum::http::Request;
use error_traits::MapErrToString;
use google_youtube3::oauth2::read_application_secret;
use maptypings::{ErrIf, MapType, WrapInRes};
use reqwest::RequestBuilder;

use crate::model::db::{combine_old_new_tokens, set_access_token};
use crate::model::net::funcs::{build_post_request, find_by_key};
use crate::model::net::types::{GET_ACCESS_TOKEN_URL, QUERY_SEPARATOR, STATE_CODE};
use crate::model::youtube::types::YouTubeAccessToken;
use crate::StdResult;

/// Build request access token using `auth code`.
async fn request_access_token(auth_code: &str) -> eyre::Result<RequestBuilder> {
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
    let secret = read_application_secret(secret_path).await?;
    let params = [
        ("client_id", secret.client_id.as_str()),
        ("client_secret", secret.client_secret.as_str()),
        ("code", auth_code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", secret.redirect_uris[0].as_str()),
    ];
    build_post_request(GET_ACCESS_TOKEN_URL, params)
}

fn get_query(req: &Request<Body>) -> String {
    let url_encoded_query = req.uri().query().unwrap_or("");
    let decoded_query: String = url::form_urlencoded::parse(url_encoded_query.as_bytes())
        .map(|(k, v)| [&k, "=", &v, "&"].concat())
        .collect();
    decoded_query
}

fn redirect_user(redirect_to: &str) -> StdResult<axum::response::Response<BoxBody>, String> {
    axum::response::Response::builder()
        .header(axum::http::header::LOCATION, redirect_to)
        .status(axum::http::status::StatusCode::PERMANENT_REDIRECT)
        .body(BoxBody::default())
        .map_err_to_str()
}

struct CodeAndUserId<'a> {
    for_user: &'a str,
    auth_code: &'a str,
}

fn get_params_from_query(decoded_query: &'_ str) -> StdResult<CodeAndUserId<'_>, &'static str> {
    let state = find_by_key(decoded_query, "&", "state").map_err(|_| "state not found")?;

    find_by_key(state, QUERY_SEPARATOR, "state_code")
        .map_err(|_| "state code not found")?
        .err_if(|s| !s.contains(STATE_CODE), "state codes don't match")?;

    let for_user =
        find_by_key(state, QUERY_SEPARATOR, "for_user").map_err(|_| "for_user not found")?;
    let auth_code = find_by_key(decoded_query, "&", "code").map_err(|_| "auth_code not found")?;
    Ok(CodeAndUserId {
        for_user,
        auth_code,
    })
}

pub async fn handle_auth_code(
    req: Request<Body>,
) -> axum::response::Result<axum::response::Response> {
    let op = "crates/internal/src/model/auth_server.rs:handle_auth_code";

    log::info!("[LOG]  op: '{op}'  ( started [ OK ] )");
    log::info!("req is: {req:?}");

    let decoded_query = get_query(&req);
    let CodeAndUserId {
        for_user,
        auth_code,
    } = get_params_from_query(&decoded_query)?;

    let access_token_request = request_access_token(auth_code)
        .await
        .map_err(|_| "building token request failed")?;
    let resp = access_token_request
        .send()
        .await
        .map_err(|_| "access token request failed")?;

    let new_token = resp
        .json::<YouTubeAccessToken>()
        .await
        .map_err(|_| "couldn't deserialize access token")?;
    let db_url = env!("REDIS_YOUTUBE_ACCESS_TOKEN_STORAGE");
    let combined_token = combine_old_new_tokens(for_user, new_token, db_url);
    let serialized_access_token = serde_json::to_string(&combined_token).map_err(|_| "db error")?;
    set_access_token(for_user, &serialized_access_token, db_url).map_err(|_| "db error")?;

    let redirect = redirect_user(env!("BOT_REDIRECT_URL"))?;

    log::info!("[LOG]  op: '{op}'  ( finished [ OK ] )");
    redirect.in_ok()
}

pub async fn serve_all(req: Request<Body>) -> &'static str {
    let op = "crates/internal/src/model/auth_server.rs:serve_all";

    log::info!(" [LOG]  op: '{op}'  (  started [ OK ] )");

    let (parts, body) = req.into_parts();
    log::info!(" [LOG]  op: '{op}'  ( 'parts' is '{:#?}' )", &parts);
    log::info!(" [LOG]  op: '{op}'  ( 'body' is '{:#?}' )", &body);

    log::info!(" [LOG]  op: '{op}'  (  finished [ OK ] )");

    "server is up ✔"
}
