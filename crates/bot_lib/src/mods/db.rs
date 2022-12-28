use google_youtube3::oauth2::ApplicationSecret;
use redis::Commands;
use crate::mods::youtube::types::YouTubeAccessToken;

pub(crate) fn get_access_token(user_id: &str, redis_url: &str) -> eyre::Result<YouTubeAccessToken>
{
    let user_id = format!("youtube_access_token_rand_fuy6776d75ygku8i7_user_id_{user_id}");
    log::info!("getting access_token from a database | (silent on failure)");
    let client = redis::Client::open(redis_url);
    log::info!("[:: LOG ::]    ( @:[fn::get_access_token] 'client' is [| '{:#?}' |] )", &client);
    let client = client?;
    let mut con = client.get_connection()?;
    let serialized_access_token = con.get::<_, String>(user_id);
    log::info!("[:: LOG ::]    ( @:[fn::get_access_token] 'serialized_access_token' is [| '{:#?}' |] )", &serialized_access_token);
    let serialized_access_token = serialized_access_token?;
    let youtube_access_token = serde_json::from_str::<YouTubeAccessToken>(&serialized_access_token);
    log::info!("[:: LOG ::]    ( @:[fn::get_access_token] 'youtube_access_token' is [| '{:#?}' |] )", &youtube_access_token);
    let youtube_access_token = youtube_access_token?;
    log::info!("access_token acquired!");
    Ok(youtube_access_token)
}

pub(crate) fn set_access_token(user_id: &str, token: &str, redis_url: &str) -> eyre::Result<()>
{
    let user_id = format!("youtube_access_token_rand_fuy6776d75ygku8i7_user_id_{user_id}");
    log::info!("saving access_token to a database | (silent on failure)");
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection()?;
    con.set(user_id, token)?;
    log::info!("access_token saved!");
    Ok(())
}

pub(crate) async fn refresh_access_token
(
    user_id: &str,
    token: YouTubeAccessToken,
    redis_url: &str,
    oauth2_secret: ApplicationSecret
)
    -> eyre::Result<YouTubeAccessToken>
{
    let time_remains = time::OffsetDateTime::now_utc() - token.expires_in;
    if time_remains.whole_minutes() < 10
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
            reqwest::Client::new().post(reqwest::Url::parse("https://oauth2.googleapis.com/token")?)
                .header(hyper::header::HOST, "oauth2.googleapis.com")
                .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(uri.query().unwrap().to_owned());
        let resp = req.send().await?;
        log::info!(" [:: LOG ::]    ( @:[fn::refresh_access_token] 'resp' is [| '{:?}' |] )", &resp);
        let new_token = resp.json::<YouTubeAccessToken>().await;
        log::info!(" [:: LOG ::]    ( @:[fn::refresh_access_token] 'new_token' is [| '{:?}' |] )", &new_token);
        dbg!(&new_token);
        let new_token = new_token?;
        set_access_token(user_id, &serde_json::to_string(&new_token)?, redis_url)?;
        Ok(YouTubeAccessToken { refresh_token: token.refresh_token, ..new_token })
    }
    else
    { Ok(token) }
}

#[cfg(test)]
mod tests
{
    use google_youtube3::oauth2;
    use super::*;
    
    #[tokio::test]
    async fn get_set_from_to_db()
    {
        simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info)).unwrap();
        let redis_url = std::env::var("REDIS_URL").unwrap();
        let secret_path = std::env::var("OAUTH_SECRET_PATH").unwrap();
        let secret = oauth2::read_application_secret(secret_path).await.unwrap();
        let user_id = "Александр Иванов";
        let token =
            YouTubeAccessToken
            {
                access_token: "ya29.a0AX9GBdUmplhCJxiwXfKQxJkuXFGljbc1Y4BLLHb4XpRH0xJfmStBZ3geXTkuRtiP-RsYsaI9opw9YuaaHylW9WAnPfq8G26vmaWiPcNjvIzd_nhMBv33h-Z181-z_EzXiqu8Ia4v4liPU2NS5azxarhgXoRxaCgYKAaASARMSFQHUCsbC453qB9CbVF7igPLkpAA0wQ0163".to_owned(),
                expires_in: time::OffsetDateTime::now_utc(),
                refresh_token: Some("1//04AOacug-qj0pCgYIARAAGAQSNwF-L9Ir48RirxxurCPcZvKuHvHaty_e8nnEq2bCA6af5-cVJIXU0f54-hhYnRRVFwHuaUXPvyc".to_owned()),
                scope: vec![],
                token_type: "Bearer".to_owned()
            };
        set_access_token(user_id, &serde_json::to_string(&token).unwrap(), &redis_url).unwrap();
        let saved_token = get_access_token(user_id, &redis_url).unwrap();
        assert_eq!(token.refresh_token.as_ref().unwrap(), saved_token.refresh_token.as_ref().unwrap());
        assert_eq!(token.access_token, saved_token.access_token);
        assert_eq!(token.expires_in, saved_token.expires_in);
        refresh_access_token(user_id, saved_token, &redis_url, secret).await.unwrap();
        let refreshed_access_token = get_access_token(user_id, &redis_url).unwrap();
        assert_eq!(refreshed_access_token.refresh_token, token.refresh_token);
    }
}


