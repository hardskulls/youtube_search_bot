use redis::Commands;
use crate::mods::youtube::types::YouTubeAccessToken;

pub(crate) fn get_access_token(user_id: &str) -> eyre::Result<YouTubeAccessToken>
{
    log::info!("getting access_token from a database | (silent on failure)");
    let client = redis::Client::open(std::env::var("REDIS_URL")?)?;
    let mut con = client.get_connection()?;
    let serialized_access_token: String = con.get(user_id)?;
    let youtube_access_token: YouTubeAccessToken = serde_json::from_str(&serialized_access_token)?;
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


