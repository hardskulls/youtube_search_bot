use axum::body::BoxBody;
use axum::http::Request;
use error_traits::{MapErrToString, WrapInErr, WrapInOk};
use google_youtube3::oauth2::read_application_secret;
use hyper::Body;
use reqwest::RequestBuilder;
use crate::model::db::{combine_old_new_tokens, set_access_token};
use crate::model::net::funcs::find_by_key;
use crate::model::youtube::types::YouTubeAccessToken;
use crate::StdResult;

/// Parameters for access token request (not code request).
async fn params(auth_code : &str) -> StdResult<[(String, String); 5], std::io::Error>
{
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
    let secret = read_application_secret(secret_path).await?;
    [
        ("client_id".to_owned(), secret.client_id),
        ("client_secret".to_owned(), secret.client_secret),
        ("code".to_owned(), auth_code.to_owned()),
        ("grant_type".to_owned(), "authorization_code".to_owned()),
        ("redirect_uri".to_owned(), secret.redirect_uris[0].clone())
    ]
    .in_ok()
}

async fn access_token_req(auth_code : &str) -> eyre::Result<RequestBuilder>
{
    let params = params(auth_code).await?;
    let uri = reqwest::Url::parse_with_params("https://oauth2.googleapis.com/token", &params)?;
    reqwest::Client::new()
        .post(reqwest::Url::parse("https://oauth2.googleapis.com/token")?)
        .header(hyper::header::HOST, "oauth2.googleapis.com")
        .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(uri.query().ok_or(eyre::eyre!("no query!"))?.to_owned())
        .in_ok()
}

fn get_query(req : &Request<Body>) -> String
{
    let url_encoded_query = req.uri().query().unwrap_or("");
    let decoded_query : String =
        form_urlencoded::parse(url_encoded_query.as_bytes())
            .map(|(k, v)| [&k, "=", &v, "&"].concat())
            .collect();
    decoded_query
}

fn redirect_user(redirect_to : &str) -> StdResult<axum::response::Response<BoxBody>, String>
{
    axum::response::Response::builder()
        .header(hyper::header::LOCATION, redirect_to)
        .status(axum::http::status::StatusCode::PERMANENT_REDIRECT)
        .body(BoxBody::default())
        .map_err_to_str()
}

struct CodeAndUserId<'a>
{
    for_user : &'a str,
    auth_code : &'a str
}

fn get_params_from_query(decoded_query : &'_ str)
    -> StdResult<CodeAndUserId<'_>, &'static str>
{
    let state = find_by_key(decoded_query, "&", "state").map_err(|_| "state not found")?;
    
    let state_code = find_by_key(state, "xplusx", "state_code").map_err(|_| "state code not found")?;
    if !state_code.contains("liuhw9p38y08q302q02h0gp9g0p2923924u0s")
    { return "state codes don't match".in_err() }
    
    let for_user = find_by_key(state, "xplusx", "for_user").map_err(|_| "for_user not found")?;
    let auth_code = find_by_key(decoded_query, "&", "code").map_err(|_| "auth_code not found")?;
    Ok(CodeAndUserId { for_user, auth_code })
}

pub async fn handle_auth_code(req : Request<Body>) -> axum::response::Result<axum::response::Response>
{
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] started [ OK ] )");
    
    let decoded_query = get_query(&req);
    let CodeAndUserId { for_user, auth_code } = get_params_from_query(&decoded_query)?;
    
    let tok_req = access_token_req(auth_code).await.map_err(|_| "building token request failed")?;
    let resp = tok_req.send().await.map_err(|_| "access token request failed")?;
    
    let new_token = resp.json::<YouTubeAccessToken>().await.map_err(|_| "couldn't deserialize access token")?;
    let db_url = env!("REDIS_URL");
    let combined_token = combine_old_new_tokens(for_user, new_token, db_url);
    let serialized_access_token = serde_json::to_string(&combined_token).map_err(|_| "db error")?;
    set_access_token(for_user, &serialized_access_token, db_url).map_err(|_| "db error")?;
    
    let bot_url = env!("BOT_REDIRECT_URL");
    let redirect = redirect_user(bot_url)?;
    
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] finished [ OK ] )");
    Ok(redirect)
}

pub async fn serve_all(req : Request<Body>) -> &'static str
{
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] started [ OK ] )");
    
    let (parts, body) = req.into_parts();
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] 'parts' is [| '{:#?}' |] )", &parts);
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] 'body' is [| '{:#?}' |] )", &body);
    
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] finished [ OK ] )");
    "server is up ✔"
}

