use google_youtube3::oauth2::read_application_secret;
use redis::Commands;
use crate::mods::youtube::types::YouTubeAccessToken;

pub(crate) fn get_access_token(user_id: &str) -> eyre::Result<YouTubeAccessToken>
{
    log::info!("getting access_token from a database | (silent on failure)");
    let client = redis::Client::open(std::env::var("REDIS_URL")?);
    log::info!(" [:: LOG ::] ... : ( @:[fn::get_access_token] 'client' is [| '{:#?}' |] )", &client);
    let client = client?;
    let mut con = client.get_connection();
    log::info!(" [:: LOG ::] ... : ( @:[fn::get_access_token] 'con' is [| '{:#?}' |] )", &con);
    let con = con?;
    let serialized_access_token = con.get::<_, String>(user_id);
    log::info!(" [:: LOG ::] ... : ( @:[fn::get_access_token] 'serialized_access_token' is [| '{:#?}' |] )", &serialized_access_token);
    let serialized_access_token = serialized_access_token?;
    let youtube_access_token = serde_json::from_str::~YouTubeAccessToken>(&serialized_access_token);
    log::info!(" [:: LOG ::] ... : ( @:[fn::get_access_token] 'youtube_access_token' is [| '{:#?}' |] )", &youtube_access_token);
    let youtube_access_token = youtube_access_token?;
    log::info!("access_token acquired!");
    Ok(youtube_access_token)
}

pub(crate) fn set_access_token(user_id: &str, token: &str) -> eyre::Result<()>
{
    log::info!("saving access_token to a database | (silent on failure)");
    let client = redis::Client::open(std::env::var("REDIS_URL")?)?;
    let mut con = client.get_connection()?;
    con.set(user_id, token)?;
    log::info!("access_token saved!");
    Ok(())
}

pub(crate) async fn refresh_access_token(user_id: &str, token: YouTubeAccessToken) -> eyre::Result<YouTubeAccessToken>
{
    let time_remains = time::OffsetDateTime::now_utc() - token.expires_in;
    if time_remains.whole_minutes() < 10
    {
        let secret_path = std::env::var("OAUTH_SECRET_PATH")?;
        let secret = read_application_secret(secret_path).await?;
        let params =
            [
                ("client_id", secret.client_id.as_str()),
                ("client_secret", secret.client_secret.as_str()),
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
        let new_token = resp.json::<YouTubeAccessToken>().await?;
        set_access_token(user_id, &serde_json::to_string(&new_token)?)?;
        Ok(YouTubeAccessToken { refresh_token: token.refresh_token, ..new_token })
    }
    else
    { Ok(token) }
}


