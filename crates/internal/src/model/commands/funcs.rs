
use error_traits::{LogErr, MapErrBy, WrapInErr, WrapInOk};

use crate::{FlatRes, StdResult};
use crate::model::db::{delete_access_token, get_access_token};
use crate::model::dialogue::funcs::get_dialogue_data;
use crate::model::dialogue::types::{ListCommandSettings, MessageTriplet, SearchCommandSettings, State, TheDialogue};
use crate::model::errors::NoTextError;
use crate::model::utils::{HTMLise, maybe_print};
use crate::model::youtube::types::YouTubeAccessToken;


fn build_log_out_req(token : YouTubeAccessToken) -> eyre::Result<reqwest::RequestBuilder>
{
    log::info!(" [:: LOG ::]     @[fn]:[model::commands::build_log_out_req] :: [Started]");
    let url = "https://oauth2.googleapis.com/revoke";
    let params : &[(&str, &str)] = &[("token", &token.refresh_token.unwrap_or(token.access_token))];
    let body = reqwest::Url::parse_with_params(url, params)?.query().ok_or(NoTextError)?.to_owned();
    reqwest::Client::new()
        .post(reqwest::Url::parse(url)?)
        .header(hyper::header::HOST, "oauth2.googleapis.com")
        .header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .in_ok()
}

/// Revoke `refresh token` and delete token from db.
pub(crate) async fn log_out(user_id : &str, db_url : &str) -> FlatRes<MessageTriplet>
{
    log::info!(" [:: LOG ::]     @[fn]:[model::commands::log_out] :: [Started]");
    let log_prefix = " [:: LOG ::]  :  @fn:[commands::funcs::log_out]  ->  error: ";
    let err = || ("Couldn't log out ❌".to_owned(), None, None);
    
    if let Ok(token) = get_access_token(user_id, db_url)
    {
        let req = build_log_out_req(token).log_err(log_prefix).map_err_by(err)?;
        let resp = req.send().await.log_err(log_prefix).map_err_by(err)?;
        if !resp.status().is_success()
        { return err().in_err() }
        
        delete_access_token(user_id, db_url).log_err(log_prefix).map_err_by(err)?;
        
        ("Logged out successfully ✅".to_owned(), None, None).in_ok()
    }
    else
    { err().in_err() }
}

/// Pretty print config.
fn print_search_config(search_settings : &SearchCommandSettings) -> String
{
    let SearchCommandSettings { target, search_in, .. } = search_settings;
    let SearchCommandSettings { result_limit, text_to_search, .. } = search_settings;
    let t =
        format!
        (
            "{}{}{}{}",
            maybe_print(format!("\n🎯 {}  =  ", "Target".to_bold()), target, ""),
            maybe_print(format!("\n💳 {}  =  ", "Search in".to_bold()), search_in, ""),
            maybe_print(format!("\n🧮 {}  =  ",  "Result limit".to_bold()), result_limit, ""),
            maybe_print(format!("\n💬 {}  =  ",  "Text to search".to_bold()), text_to_search, "")
        );
    if t.is_empty()
    { "You've activated 'search command' 🔎".to_owned() }
    else
    { format!("Your search parameters are{t}") }
}

/// Pretty print config.
fn print_list_config(list_settings : &ListCommandSettings) -> String
{
    let ListCommandSettings { target, result_limit, sorting } = list_settings;
    let t =
        format!
        (
            "{}{}{}",
            maybe_print(format!("\n🎯 {}  =  ", "Target".to_bold()), target, ""),
            maybe_print(format!("\n🗃 {}  =  ", "Sorting".to_bold()), sorting, ""),
            maybe_print(format!("\n🧮 {}  =  ",  "Result limit".to_bold()), result_limit, "")
        );
    if t.is_empty()
    { "You've activated 'list command' 📃".to_owned() }
    else
    { format!("Your list parameters are{t}") }
}

pub(crate) async fn info(dialogue : &TheDialogue) -> StdResult<MessageTriplet, MessageTriplet>
{
    log::info!(" [:: LOG ::]     @[fn]:[model::commands::info] :: [Started]");
    let log_prefix = " [:: LOG ::]  :  @fn:[commands::funcs::info]  ->  error: ";
    let create_msg = |m : &str| (m.to_owned(), None, None);
    let user_error: fn() -> MessageTriplet = || ("Info command failed ❌".to_owned(), None, None);
    
    let d_data = get_dialogue_data(dialogue).await.log_err(log_prefix).map_err_by(user_error)?;
    match d_data.state
    {
        State::Starting => Ok(create_msg("Bot just started 🚀")),
        State::SearchCommandActive(search_config) => create_msg(&print_search_config(&search_config)).in_ok(),
        State::ListCommandActive(list_config) => create_msg(&print_list_config(&list_config)).in_ok()
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use crate::model::commands::funcs::{build_log_out_req, print_list_config, print_search_config};
    use crate::model::dialogue::types::{ListCommandSettings, SearchCommandSettings};
    use crate::model::keyboards::types::Requestable;
    use crate::model::net::traits::RespTargetSubscriptions;
    use crate::model::youtube::types::YouTubeAccessToken;
    
    #[test]
    fn printable_test()
    {
        let c = SearchCommandSettings::default();
        assert_eq!(print_search_config(&c), "You've activated 'search command' 🔎");
        
        let mut c = ListCommandSettings::default();
        assert_eq!(print_list_config(&c), "You've activated 'list command' 📃");
        
        c.target = Requestable::Subscription(RespTargetSubscriptions).into();
        assert_eq!(print_list_config(&c), "Your list config is\n🎯 <b>Target</b>  is  Subscription");
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


