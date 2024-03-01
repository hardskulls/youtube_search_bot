use axum::{body::Body, body::BoxBody, http::Request};
use error_traits::PassErrWith;
use google_youtube3::oauth2::read_application_secret;
use maptypings::{ErrIf, MapType, WrapInRes};
use reqwest::RequestBuilder;

use thiserror::Error;

use crate::model::db::{combine_old_new_tokens, set_access_token};
use crate::model::net::funcs::{build_post_request, find_by_key};
use crate::model::net::types::{
    GET_ACCESS_TOKEN_URL, KV_SEP, PAIR_SEP, QUERY_SEPARATOR, STATE_CODE,
};
use crate::model::utils::PassWith;
use crate::model::youtube::types::YouTubeAccessToken;
use crate::Res;

mod keys {
    pub(super) const STATE: &str = "state";
    pub(super) const STATE_CODE: &str = "state_code";
    pub(super) const FOR_USER: &str = "for_user";
    pub(super) const CODE: &str = "code";
}

struct User<T>(T);

struct AuthCode<T>(T);

#[derive(Debug, Error)]
#[error("internal error")]
struct InternalError;

#[derive(Debug, Error)]
enum StateCodeError {
    #[error("state codes don't match")]
    Mismatch,
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

pub async fn handle_auth_code(
    req: Request<Body>,
) -> axum::response::Result<axum::response::Response> {
    handle_auth_code_internal(req)
        .await
        .pass_err_with(|e| log::error!("error: {e}"))
        .map_err(|_| InternalError.to_string())?
        .in_ok()
}

async fn handle_auth_code_internal(req: Request<Body>) -> Res<axum::response::Response> {
    let op = "crates/internal/src/model/auth_server.rs:handle_auth_code";

    log::info!("[LOG]  op: '{op}'  ( started [ OK ] )");
    log::info!("req is: {req:?}");

    let decoded_query = get_query(&req);
    let (for_user, auth_code) = get_params_from_query(&decoded_query)?;

    let db_url = env!("REDIS_YOUTUBE_ACCESS_TOKEN_STORAGE");

    let new_token = get_youtube_token(auth_code).await?;
    let updated_token = combine_old_new_tokens(for_user.0, new_token, db_url);
    let serialized_token = serde_json::to_string(&updated_token)?;

    set_access_token(for_user.0, &serialized_token, db_url)?;

    redirect_user(env!("BOT_REDIRECT_URL"))?
        .pass_with(|| log::info!("[LOG]  op: '{op}'  ( finished [ OK ] )"))
        .in_ok()
}

/// Build request access token using `auth code`.
async fn request_access_token<S: AsRef<str>>(
    auth_code: AuthCode<S>,
) -> eyre::Result<RequestBuilder> {
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
    let secret = read_application_secret(secret_path).await?;
    let params = [
        ("client_id", secret.client_id.as_str()),
        ("client_secret", secret.client_secret.as_str()),
        ("code", auth_code.0.as_ref()),
        ("grant_type", "authorization_code"),
        ("redirect_uri", secret.redirect_uris[0].as_str()),
    ];
    build_post_request(GET_ACCESS_TOKEN_URL, params)
}

fn get_query(req: &Request<Body>) -> String {
    let url_encoded_query = req.uri().query().unwrap_or_default();
    let decoded_query: String = url::form_urlencoded::parse(url_encoded_query.as_bytes())
        .map(|(k, v)| [&k, KV_SEP, &v, PAIR_SEP].concat())
        .collect();
    decoded_query
}

fn redirect_user(redirect_to: &str) -> Res<axum::response::Response<BoxBody>> {
    axum::response::Response::builder()
        .header(axum::http::header::LOCATION, redirect_to)
        .status(axum::http::status::StatusCode::PERMANENT_REDIRECT)
        .body(BoxBody::default())
        .map_err(<_>::into)
}

fn get_params_from_query(decoded_query: &str) -> Res<(User<&str>, AuthCode<&str>)> {
    let state = find_by_key(decoded_query, PAIR_SEP, keys::STATE)?;
    let auth_code = find_by_key(decoded_query, PAIR_SEP, keys::CODE)?.map_type(AuthCode);

    find_by_key(state, QUERY_SEPARATOR, keys::STATE_CODE)?
        .err_if(|s| !s.contains(STATE_CODE), StateCodeError::Mismatch)?;

    let for_user = find_by_key(state, QUERY_SEPARATOR, keys::FOR_USER)?.map_type(User);

    (for_user, auth_code).in_ok()
}

async fn get_youtube_token<S: AsRef<str>>(auth_code: AuthCode<S>) -> Res<YouTubeAccessToken> {
    let req_builder = request_access_token(auth_code).await?;
    let resp = req_builder.send().await?;
    resp.json::<YouTubeAccessToken>().await?.in_ok()
}
