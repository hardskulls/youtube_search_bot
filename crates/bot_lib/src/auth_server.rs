use axum::body::BoxBody;
use axum::http::Request;
use google_youtube3::oauth2::read_application_secret;
use hyper::Body;
use reqwest::RequestBuilder;

use error_traits::MapErrToString;

use crate::db::{combine_old_new_tokens, set_access_token};
use crate::net::find_by_key;
use crate::StdResult;
use crate::youtube::types::YouTubeAccessToken;

async fn params(auth_code: &str) -> [(String, String); 5]
{
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
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
        .header(hyper::header::HOST, "oauth2.googleapis.com")
        .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(uri.query().unwrap().to_owned())
}

fn get_query(req: &Request<Body>) -> String
{
    let url_encoded_query = req.uri().query().unwrap_or("");
    let decoded_query: String =
        form_urlencoded::parse(url_encoded_query.as_bytes())
            .map(|(k, v)| [&k, "=", &v, "&"].concat())
            .collect();
    decoded_query
}

fn redirect_user(redirect_to: &str) -> StdResult<axum::response::Response<BoxBody>, String>
{
    axum::response::Response::builder()
        .header(hyper::header::LOCATION, redirect_to)
        .status(axum::http::status::StatusCode::PERMANENT_REDIRECT)
        .body(BoxBody::default())
        .map_err_to_str()
}

pub async fn handle_auth_code(req: Request<Body>) -> axum::response::Result<axum::response::Response>
{
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] started [ OK ] )");
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] 'req' is [| '{:#?}' |] )", &req);
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] 'req.body()' is [| '{:#?}' |] )", &req.body());
    
    let decoded_query = get_query(&req);
    let state = find_by_key(&decoded_query, "&", "state").map_err(|_| "state not found")?;
    
    let state_code = find_by_key(state, "xplusx", "state_code").map_err(|_| "state code not found")?;
    if !state_code.contains("liuhw9p38y08q302q02h0gp9g0p2923924u0s")
    { return Err("state codes don't match".into()) }
    let for_user = find_by_key(state, "xplusx", "for_user").map_err(|_| "for_user not found")?;
    
    let auth_code = find_by_key(&decoded_query, "&", "code").map_err(|_| "auth_code not found")?;
    
    let tok_req = access_token_req(auth_code).await;
    let resp = tok_req.send().await.map_err(|_| "access token request failed")?;
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] 'resp' is [| '{:#?}' |] )", &resp);
    
    let new_token = resp.json::<YouTubeAccessToken>().await.map_err(|_| "couldn't deserialize access token")?;
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] 'new_token' is [| '{:#?}' |] )", &new_token);
    let db_url = env!("REDIS_URL");
    let t = combine_old_new_tokens(for_user, new_token, db_url);
    let serialized_access_token = serde_json::to_string(&t).map_err(|_| "db error")?;
    set_access_token(for_user, &serialized_access_token, db_url).map_err(|_| "db error")?;
    
    let redirect = redirect_user("https://t.me/test_echo_123_456_bot")?;
    
    log::info!(" [:: LOG ::]    ( @:[fn::handle_auth_code] finished [ OK ] )");
    Ok(redirect)
}

pub async fn serve_all(req: Request<Body>) -> &'static str
{
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] started [ OK ] )");
    
    let (p, b) = req.into_parts();
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] 'p' is [| '{:#?}' |] )", &p);
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] 'b' is [| '{:#?}' |] )", &b);
    
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] finished [ OK ] )");
    "server is up"
}


