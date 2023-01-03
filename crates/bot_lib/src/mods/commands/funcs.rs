use std::fmt::{Debug, Display};
use teloxide::Bot;
use teloxide::requests::Requester;
use teloxide::types::{InlineKeyboardMarkup, Me, Message};
use teloxide::utils::command::BotCommands;
use error_traits::{LogErr, MapErrBy};
use crate::dialogue::types::{DialogueData, ListConfigData, SearchConfigData, State, TheDialogue};
use crate::mods::db::{delete_access_token, get_access_token};
use crate::mods::dialogue::funcs::get_dialogue_data;
use crate::mods::errors::{NoTextError};
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

fn maybe_print<T: Display + Debug>(prefix: &str, printable: &Option<T>, default: &str) -> String
{
    if let Some(p) = printable
    { format!("{prefix}{p:#?}") }
    else
    { default.to_owned() }
}

fn print_search_config(c: &SearchConfigData) -> String
{
    let SearchConfigData { target, search_by, result_limit } = c;
    let t =
        format!
        (
            "{}{}{}",
            maybe_print("\nTarget  is  ", target, ""),
            maybe_print("\nSearch By  is  ", search_by, ""),
            maybe_print("\nResult Limit  is  ", result_limit, "")
        );
    if t.is_empty()
    { "You've activated 'search command'".to_owned() }
    else
    { format!("Your search config is{t}") }
}

fn print_list_config(c: &ListConfigData) -> String
{
    let ListConfigData { target, result_limit, sort_by, filter } = c;
    let t =
        format!
        (
            "{}{}{}{}",
            maybe_print("\nTarget  is  ", target, ""),
            maybe_print("\nFilter  is  ", filter,  ""),
            maybe_print("\nSort By  is  ", sort_by, ""),
            maybe_print("\nResult Limit  is  ", result_limit, "")
        );
    if t.is_empty()
    { "You've activated 'list command'".to_owned() }
    else
    { format!("Your list config is{t}") }
}

pub(crate) async fn info(dialogue: &TheDialogue) -> StdResult<MessageContents, MessageContents>
{
    let log_msg = " [:: LOG ::]  :  @fn:[commands::funcs::info]  ->  error: ";
    let create_msg = |m: &str| (m.to_owned(), None, None);
    let default_err: fn() -> MessageContents = || ("Info command failed ❌".to_owned(), None, None);
    let d_data = get_dialogue_data(dialogue).await.log_err(log_msg).map_err_by(default_err)?;
    match d_data.state
    {
        State::Starting => Ok(create_msg("Bot just started 🚀")),
        State::SearchCommandActive(search_config) => Ok(create_msg(&print_search_config(&search_config))),
        State::ListCommandActive(list_config) => Ok(create_msg(&print_list_config(&list_config)))
    }
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

#[cfg(test)]
mod tests
{
    use crate::mods::inline_keyboards::types::ListTarget;
    use super::*;
    
    #[test]
    fn printable_test()
    {
        let c = SearchConfigData::default();
        assert_eq!(print_search_config(&c), "You've activated 'search command'");
        
        let mut c = ListConfigData::default();
        assert_eq!(print_list_config(&c), "You've activated 'list command'");
        
        c.target = ListTarget::Subscription.into();
        assert_eq!(print_list_config(&c), "Your list config is\ntarget   Subscription");
    }
}


