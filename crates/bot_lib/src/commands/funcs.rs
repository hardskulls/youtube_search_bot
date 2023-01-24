use std::fmt::{Debug, Display};

use error_traits::{LogErr, MapErrBy, WrapInErr, WrapInOk};

use crate::{FlatRes, StdResult};
use crate::db::{delete_access_token, get_access_token};
use crate::dialogue::funcs::get_dialogue_data;
use crate::dialogue::types::{ListCommandSettings, MessageTriplet, SearchCommandSettings, State, TheDialogue};
use crate::errors::NoTextError;
use crate::youtube::types::YouTubeAccessToken;

fn build_log_out_req(token: YouTubeAccessToken) -> eyre::Result<reqwest::RequestBuilder>
{
    let url = "https://oauth2.googleapis.com/revoke";
    let params: &[(&str, &str)] = &[("token", &token.refresh_token.unwrap_or(token.access_token))];
    let body = reqwest::Url::parse_with_params(url, params)?.query().ok_or(NoTextError)?.to_owned();
    reqwest::Client::new()
        .post(reqwest::Url::parse(url)?)
        .header(hyper::header::HOST, "oauth2.googleapis.com")
        .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .in_ok()
}

/// Revoke `refresh token` and delete token from db.
pub(crate) async fn log_out(user_id: &str, db_url: &str) -> FlatRes<MessageTriplet>
{
    let err = || ("Couldn't log out ❌".to_owned(), None, None);
    if let Ok(token) = get_access_token(user_id, db_url)
    {
        let req = build_log_out_req(token).map_err_by(err)?;
        let resp = req.send().await.map_err_by(err)?;
        if !resp.status().is_success()
        { return err().in_err() }
        
        delete_access_token(user_id, db_url).map_err_by(err)?;
    
        ("Logged out successfully ✔".to_owned(), None, None).in_ok()
    }
    else
    { err().in_err() }
}

fn maybe_print<T: Display + Debug>(prefix: &str, printable: &Option<T>, default: &str) -> String
{
    if let Some(p) = printable
    { format!("{prefix}{p:#?}") }
    else
    { default.to_owned() }
}

fn print_search_config(c: &SearchCommandSettings) -> String
{
    let SearchCommandSettings { target, search_in: search_by, result_limit, .. } = c;
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

fn print_list_config(c: &ListCommandSettings) -> String
{
    let ListCommandSettings { target, result_limit, sorting: sort_by } = c;
    let t =
        format!
        (
            "{}{}{}",
            maybe_print("\nTarget  is  ", target, ""),
            maybe_print("\nSort By  is  ", sort_by, ""),
            maybe_print("\nResult Limit  is  ", result_limit, "")
        );
    if t.is_empty()
    { "You've activated 'list command'".to_owned() }
    else
    { format!("Your list config is{t}") }
}

pub(crate) async fn info(dialogue: &TheDialogue) -> StdResult<MessageTriplet, MessageTriplet>
{
    let log_prefix = " [:: LOG ::]  :  @fn:[commands::funcs::info]  ->  error: ";
    let create_msg = |m: &str| (m.to_owned(), None, None);
    let default_err: fn() -> MessageTriplet = || ("Info command failed ❌".to_owned(), None, None);
    let d_data = get_dialogue_data(dialogue).await.log_err(log_prefix).map_err_by(default_err)?;
    match d_data.state
    {
        State::Starting => Ok(create_msg("Bot just started 🚀")),
        State::SearchCommandActive(search_config) => Ok(create_msg(&print_search_config(&search_config))),
        State::ListCommandActive(list_config) => Ok(create_msg(&print_list_config(&list_config)))
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use crate::dialogue::types::{ListCommandSettings, SearchCommandSettings};
    use crate::keyboards::types::Target;
    use crate::youtube::types::YouTubeAccessToken;
    
    use super::*;
    
    #[test]
    fn printable_test()
    {
        let c = SearchCommandSettings::default();
        assert_eq!(print_search_config(&c), "You've activated 'search command'");
        
        let mut c = ListCommandSettings::default();
        assert_eq!(print_list_config(&c), "You've activated 'list command'");
        
        c.target = Target::Subscription.into();
        assert_eq!(print_list_config(&c), "Your list config is\nTarget  is  Subscription");
    }
    
    #[test]
    fn request_build_test()
    {
        let (access_token, refresh_token) = ("acc_tok_653654265432".into(), "ref_tok_76876576345".to_owned().into());
        let (expires_in, scope, token_type) = (time::OffsetDateTime::now_utc(), vec![], "Bearer".to_owned());
        let token = YouTubeAccessToken { access_token, expires_in, refresh_token, scope, token_type };
        let req = build_log_out_req(token.clone()).unwrap().build().unwrap();
        assert_eq!(req.headers().get(hyper::header::HOST).unwrap().to_str().unwrap(), "oauth2.googleapis.com");
        assert_eq!(req.headers().get(hyper::header::CONTENT_TYPE).unwrap().to_str().unwrap(), "application/x-www-form-urlencoded");
        assert_eq!(req.url().as_str(), "https://oauth2.googleapis.com/revoke");
        let expected_body = reqwest::Body::from(format!("token={t}", t = token.refresh_token.unwrap()));
        assert_eq!(req.body().unwrap().as_bytes().unwrap(), expected_body.as_bytes().unwrap());
    }
}


