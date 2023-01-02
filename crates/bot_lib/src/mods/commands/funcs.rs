use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::types::{InlineKeyboardMarkup, Me, Message};
use teloxide::utils::command::BotCommands;
use crate::dialogue::types::DialogueData;
use crate::mods::db::{delete_access_token, get_access_token};
use crate::mods::errors::{NoTextError, MapErrBy};
use crate::mods::youtube::types::YouTubeAccessToken;
use crate::StdResult;

type MessageContents = (String, Option<InlineKeyboardMarkup>, Option<DialogueData>);

fn build_log_out_req(token: YouTubeAccessToken) -> eyre::Result<reqwest::RequestBuilder>
{
    let url = "https://oauth2.googleapis.com/revoke";
    let params: &[(&str, &str)] = &[("token", &token.refresh_token.unwrap_or(token.access_token))];
    let body = reqwest::Url::parse_with_params(url, params)?.query().ok_or(NoTextError)?.to_owned();
    let req =
        reqwest::Client::new()
            .post(reqwest::Url::parse(url)?)
            .header(hyper::header::HOST, "oauth2.googleapis.com")
            .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(body);
    Ok(req)
}

/// Revoke `refresh token` and delete token from db.
pub(crate) async fn log_out(user_id: &str, db_url: &str) -> StdResult<MessageContents, MessageContents>
{
    let err = || ("Couldn't log out ❌".to_owned(), None, None);
    if let Ok(token) = get_access_token(user_id, db_url)
    {
        let req = build_log_out_req(token).map_err_by(err)?;
        let resp = req.send().await.map_err_by(err)?;
        if !resp.status().is_success()
        { return Err(err()) }
        
        delete_access_token(user_id, db_url).map_err_by(err)?;
        
        Ok(("Logged out successfully ✔".to_owned(), None, None))
    }
    else
    { Err(err()) }
}

#[inline]
pub fn is_other_command<B: BotCommands>(msg: Message, me: Me) -> bool
{
    let bot_name = me.user.username.expect("Bots must have a username");
    if let Some(text) = msg.text()
    { matches!(text.chars().next(), Some('/')) && B::parse(text, bot_name.as_str()).is_err() }
    else
    { false }
}

/// Tell user that an unknown command was received.
#[inline]
pub async fn handle_unknown_command(bot: Bot, msg: Message) -> eyre::Result<()>
{
    bot.send_message(msg.chat.id, "Unknown command 🤷‍♀️").await?;
    Ok(())
}


