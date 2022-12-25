use axum::http::Request;
use google_youtube3::oauth2::read_application_secret;
use hyper::Body;
use reqwest::RequestBuilder;

use crate::mods::db::set_access_token;
use crate::mods::net::find_by_key;
use crate::mods::youtube::types::{MapErrToString, YouTubeAccessToken};

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
        .header(hyper::header::HOST, "oauth2.googleapis.com")
        .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(uri.query().unwrap().to_owned())
}

pub async fn handle_auth_code(req: Request<Body>) -> axum::response::Result<axum::response::Response>
{
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] started [ OK ] )");
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'req' is [| '{:#?}' |] )", &req);
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'req.body()' is [| '{:#?}' |] )", &req.body());
    
    let url_encoded_query = req.uri().query().unwrap_or("");
    let decoded_query: String =
        form_urlencoded::parse(url_encoded_query.as_bytes())
            .map(|(k, v)| [&k, "=", &v, "&"].concat())
            .collect();
    let state = find_by_key(&decoded_query, "&", "state").map_err(|_| "state not found")?;
    
    let state_code = find_by_key(state, "xplusx", "state_code").map_err(|_| "state code not found")?;
    if !state_code.contains("liuhw9p38y08q302q02h0gp9g0p2923924u0s")
    { return Err("state codes don't match".into()) }
    let for_user = find_by_key(state, "xplusx", "for_user").map_err(|_| "for_user not found")?;
    
    let auth_code = find_by_key(&decoded_query, "&", "code").map_err(|_| "auth_code not found")?;
    
    let tok_req = access_token_req(auth_code).await;
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'tok_req' is [| '{:#?}' |] )", &tok_req);
    let resp = tok_req.send().await.map_err(|_| "access token request failed")?;
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'resp' is [| '{:#?}' |] )", &resp);
    
    let serialized_access_token = resp.text().await.map_err(|_| "couldn't deserialize access token")?;
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'serialized_access_token' is [| '{:#?}' |] )", &serialized_access_token);
    log::info!
    (
        " [:: LOG ::] ... : ( @:[fn::handle_auth_code] 'serde_json::from_str::<YouTubeAccessToken>' is [| '{:#?}' |] )",
        serde_json::from_str::<YouTubeAccessToken>(&serialized_access_token)
    );
    set_access_token(for_user, &serialized_access_token).map_err(|_| "db error")?;
    
    let redirect =
        axum::response::Response::builder()
            .header(hyper::header::LOCATION, "https://t.me/test_echo_123_456_bot")
            .status(axum::http::status::StatusCode::PERMANENT_REDIRECT)
            .body(axum::body::BoxBody::default())
            .map_err_to_str()?;
    
    log::info!(" [:: LOG ::] ... : ( @:[fn::handle_auth_code] finished [ OK ] )");
    Ok(redirect)
}

pub async fn serve_all(req: Request<Body>) -> &'static str
{
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] started [ OK ] )");
    let (p, b) = req.into_parts();
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] 'p' is [| '{:#?}' |] )", &p);
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] 'b' is [| '{:#?}' |] )", &b);
    log::info!(" [:: LOG ::] ... : ( @:[fn::serve_all] finished [ OK ] )");
    "server is up"
}

#[cfg(test)]
mod tests
{
    use super::*;
    
    mod serialization_testing
    {
        use crate::mods::youtube::types::YouTubeAccessToken;
        
        #[test]
        fn serialize_deserialize_string_test()
        {
            let (access_token, refresh_token) =
                ("access_token".to_owned(), Some("refresh_token".to_owned()));
            let (scope, token_type) =
                (vec!["hey".to_owned()], "id_token".to_owned());
            let expires_in = time::OffsetDateTime::now_utc();
            let token = YouTubeAccessToken { access_token, expires_in, refresh_token, scope, token_type };
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


