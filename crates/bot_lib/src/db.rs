use std::ops::Sub;
use google_youtube3::oauth2::ApplicationSecret;
use redis::Commands;
use error_traits::WrapInOk;

use crate::youtube::types::YouTubeAccessToken;

/// Required to avoid key collisions.
const TOKEN_PREFIX: &str = "youtube_access_token_rand_fuy6776d75ygku8i7_user_id_";

/// Gets a token by `user id`.
pub(crate) fn get_access_token(user_id: &str, db_url: &str) -> eyre::Result<YouTubeAccessToken>
{
    log::info!("getting access_token from a database | (silent on failure)");
    let mut con = redis::Client::open(db_url)?.get_connection()?;
    let serialized_token = con.get::<_, String>(format!("{TOKEN_PREFIX}{user_id}"))?;
    let token = serde_json::from_str::<YouTubeAccessToken>(&serialized_token)?;
    log::info!("access_token acquired!");
    token.in_ok()
}

/// Sets a token by `user id`.
pub(crate) fn set_access_token(user_id: &str, token: &str, db_url: &str) -> eyre::Result<()>
{
    log::info!("saving access_token to a database | (silent on failure)");
    let mut con = redis::Client::open(db_url)?.get_connection()?;
    con.set(format!("{TOKEN_PREFIX}{user_id}"), token)?;
    log::info!("access_token saved!");
    ().in_ok()
}

/// Deletes a token by `user id`.
pub(crate) fn delete_access_token(user_id: &str, db_url: &str) -> eyre::Result<()>
{
    log::info!(" [:: LOG ::]    ( @:[fn::delete_access_token] deleting access_token | silent on failure");
    let mut con = redis::Client::open(db_url)?.get_connection()?;
    con.del(format!("{TOKEN_PREFIX}{user_id}"))?;
    log::info!(" [:: LOG ::]    ( @:[fn::delete_access_token] access_token deleted!");
    ().in_ok()
}

/// Because `refresh token` is received only once, it needs to be moved from old token to a new one.
pub fn combine_old_new_tokens(old_token_user_id: &str, new_token: YouTubeAccessToken, db_url: &str) -> YouTubeAccessToken
{
    match get_access_token(old_token_user_id, db_url)
    {
        Ok(YouTubeAccessToken { refresh_token: Some(ref_token), .. }) =>
            YouTubeAccessToken { refresh_token: ref_token.into(), ..new_token },
        _ => new_token
    }
}

/// Constructs request for acquiring new `access token`.
pub(crate) fn refresh_token_req(oauth2_secret: ApplicationSecret, token: &YouTubeAccessToken) -> eyre::Result<reqwest::RequestBuilder>
{
    let params =
        [
            ("client_id", oauth2_secret.client_id.as_str()),
            ("client_secret", oauth2_secret.client_secret.as_str()),
            ("refresh_token", token.refresh_token.as_ref().ok_or(eyre::eyre!("No refresh token"))?),
            ("grant_type", "refresh_token")
        ];
    let uri = reqwest::Url::parse_with_params("https://oauth2.googleapis.com/token", &params)?;
    let req =
        reqwest::Client::new()
            .post(reqwest::Url::parse("https://oauth2.googleapis.com/token")?)
            .header(hyper::header::HOST, "oauth2.googleapis.com")
            .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(uri.query().ok_or(eyre::eyre!("No Query!"))?.to_owned());
    Ok(req)
}

/// Makes request for new `access token` if needed, then saves and returns it.
pub(crate) async fn refresh_access_token
(
    user_id: &str,
    token: YouTubeAccessToken,
    db_url: &str,
    refresh_token_request: reqwest::RequestBuilder
)
    -> eyre::Result<YouTubeAccessToken>
{
    let time_remains = token.expires_in.sub(time::OffsetDateTime::now_utc()).whole_minutes();
    log::info!(" [:: LOG ::]    ( @:[fn::refresh_access_token] (token is valid for) 'time_remains' is [| '{:?}' |] )", &time_remains);
    if time_remains < 10
    {
        let resp = refresh_token_request.send().await?;
        log::info!(" [:: LOG ::]    ( @:[fn::refresh_access_token] 'resp.status()' is [| '{:?}' |] )", &resp.status());
        let new_token = resp.json::<YouTubeAccessToken>().await?;
        let combined_token = YouTubeAccessToken { refresh_token: token.refresh_token, ..new_token };
        set_access_token(user_id, &serde_json::to_string(&combined_token)?, db_url)?;
        combined_token.in_ok()
    }
    else
    { token.in_ok() }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use google_youtube3::oauth2;
    
    use crate::youtube::types::YouTubeAccessToken;
    
    use super::*;
    
    #[test]
    fn get_save_token()
    {
        simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info)).unwrap();
    
        let redis_url = env!("REDIS_URL");
        let access_token = env!("TEST_ACCESS_TOKEN").to_owned();
        let refresh_token = env!("TEST_REFRESH_TOKEN").to_owned().into();
        
        let user_id = "Александр Иванов";
    
        let token =
            YouTubeAccessToken
            {
                access_token,
                expires_in: time::OffsetDateTime::now_utc(),
                refresh_token,
                scope: vec!["first".to_owned(), "second".to_owned()],
                token_type: "Bearer".to_owned()
            };
    
        set_access_token(user_id, &serde_json::to_string(&token).unwrap(), redis_url).unwrap();
        let saved_token = get_access_token(user_id, redis_url).unwrap();
        delete_access_token(user_id, redis_url).unwrap();
        
        assert_eq!(token.refresh_token.as_ref().unwrap(), saved_token.refresh_token.as_ref().unwrap());
        assert_eq!(token.access_token, saved_token.access_token);
        assert_eq!(token.expires_in, saved_token.expires_in);
        assert_eq!(token.token_type, saved_token.token_type);
        assert_eq!(token.scope, saved_token.scope);
    }
    
    #[tokio::test]
    async fn get_set_from_to_db()
    {
        simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info)).unwrap();
        
        let redis_url = env!("REDIS_URL");
        let access_token = env!("TEST_ACCESS_TOKEN").to_owned();
        let refresh_token = env!("TEST_REFRESH_TOKEN").to_owned().into();
        let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
        // api key is required for making calls from anywhere, instead of manually added urls in oauth credentials
        let oauth_api_key = env!("GOOGLE_OAUTH_API_KEY");
        
        let user_id = "Александр Иванов";
        
        let token =
            YouTubeAccessToken
            {
                access_token,
                expires_in: time::OffsetDateTime::now_utc(),
                refresh_token,
                scope: vec!["first".to_owned(), "second".to_owned()],
                token_type: "Bearer".to_owned()
            };
        
        let secret = oauth2::read_application_secret(secret_path).await.unwrap();
        let mut token_req = refresh_token_req(secret, &token).unwrap();
        token_req = token_req.query(&[("key", &oauth_api_key)]);
        
        let refreshed_access_token =
            refresh_access_token(user_id, token.clone(), redis_url, token_req).await.unwrap();
        
        assert_eq!(token.refresh_token, refreshed_access_token.refresh_token);
        assert_eq!(token.access_token, refreshed_access_token.access_token);
    }
}

