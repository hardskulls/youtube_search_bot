use std::fmt::{Debug};
use google_youtube3::oauth2::read_application_secret;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Message, ParseMode};
use url::Url;

use crate::mods::db::{get_access_token, refresh_access_token, refresh_token_req};
use crate::mods::dialogue::types::{DialogueData, Either, ListConfigData, MessageTriplet, SearchConfigData, State::{self, ListCommandActive, SearchCommandActive}};
use crate::mods::inline_keyboards::types::SearchMode;
use crate::mods::net::traits::{ItemsListRequestBuilder, ItemsResponsePage};
use crate::mods::youtube::{search_items, make_auth_url};
use crate::mods::youtube::traits::Searchable;
use crate::mods::youtube::types::{ACCESS_TYPE, RESPONSE_TYPE, SCOPE_YOUTUBE_READONLY};

/// Helper function used for `handle_text` handler.
/// Parses user input as number in order to set it as `result limit` setting.
pub(crate) fn parse_number(text: &str, configs: Either<&SearchConfigData, &ListConfigData>, dialogue_data: &DialogueData)
    -> MessageTriplet
{
    match text.parse::<u16>()
    {
        Ok(num) if num > 1 =>
            {
                let state =
                    match configs
                    {
                        Either::First(search_config) =>
                            SearchCommandActive(SearchConfigData { result_limit: (num as u32).into(), ..search_config.clone() }),
                        Either::Last(list_config) =>
                            ListCommandActive(ListConfigData { result_limit: (num as u32).into(), ..list_config.clone() })
                    };
                ("Accepted! ✅".to_owned(), None, DialogueData { state, ..dialogue_data.clone() }.into())
            }
        _ => ("Send a number greater than 0".to_owned(), None, None)
    }
}

async fn send_results<'i, S, T>(bot: &Bot, msg: &Message, list: T, result_lim: u32)
    where
        S: Searchable + 'i,
        T: IntoIterator<Item = &'i S>
{
    for s in list.into_iter().take(result_lim as _)
    {
        let (title, descr, chan_id) =
            (
                s.title().unwrap_or("No channel title").to_owned(),
                s.description().unwrap_or("No channel description").to_owned(),
                s.link().unwrap_or_else(|| "No channel id".to_owned())
            );
        let text = format!("<b>{}</b>{}{}", title + " \n\n", descr + " \n\n", chan_id);
        let _sent_msg = bot.send_message(msg.chat.id, text).parse_mode(ParseMode::Html).await;
        log::info!(" [:: LOG ::]    ( @:[fn::text_handling::send_results] '_sent_msg' is [| '{:#?}' |] )", &_sent_msg);
    }
}

/// Helper function used for `handle_text` handler.
/// Final func that does searching when everything is ready. 
pub(crate) async fn execute_search<T>
(
    bot: &Bot,
    msg: &Message,
    dialogue_data: &DialogueData,
    text_to_look_for: &str,
    result_lim: u32,
    search_mode: &SearchMode,
    what_to_search: T
)
    -> eyre::Result<MessageTriplet>
    where
        T: ItemsListRequestBuilder,
        T::Target: Default + Debug + ItemsResponsePage
{
    let user_id = msg.from().ok_or(eyre::eyre!("No User Id"))?.id.to_string();
    let redis_url = std::env::var("REDIS_URL")?;
    let Ok(token) =
        get_access_token(&user_id, &redis_url)
        else
        {
            let url = default_auth_url(&user_id).await?;
            let auth_url = format!("Use this link to log in <a href=\"{url}\">Log In</a>");
            bot.send_message(msg.chat.id, auth_url).parse_mode(ParseMode::Html).await?;
            return Ok(("Please, log in and send your text again ".to_owned(), None, None))
        };
    
    let secret_path = std::env::var("OAUTH_SECRET_PATH")?;
    let secret = read_application_secret(secret_path).await?;
    let token_req = refresh_token_req(secret, &token)?;
    let access_token = refresh_access_token(&user_id, token, &redis_url, token_req).await?.access_token;
    
    bot.send_message(msg.chat.id, "Searching, please wait 🕵️‍♂️").await?;
    let subscription_list =
        search_items(search_mode, what_to_search, text_to_look_for, &access_token, result_lim).await?;
    
    send_results(bot, msg, &subscription_list, result_lim).await;
    let result_count = subscription_list.iter().take(result_lim as _).count();

    Ok((format!("Finished! ✔ \nFound {result_count} results"), None, Some(DialogueData { state: State::Starting, ..dialogue_data.clone() })))
}

/// Construct authorization url.
async fn default_auth_url(user_id: &str) -> eyre::Result<Url>
{
    let secret_path = std::env::var("OAUTH_SECRET_PATH").unwrap();
    let secret = read_application_secret(secret_path).await?;

    let (client_id, redirect_uri) = (secret.client_id.as_str(), secret.redirect_uris[0].as_str());
    let (scope, response_type) = (&[SCOPE_YOUTUBE_READONLY], RESPONSE_TYPE);
    let state = format!("for_user={user_id}xplusxstate_code=liuhw9p38y08q302q02h0gp9g0p2923924u0s");
    let optional_params = &[("ACCESS_TYPE".to_owned().to_lowercase(), ACCESS_TYPE), ("state".to_owned(), state.as_str())];

    let url = make_auth_url(client_id, redirect_uri, response_type, scope, optional_params)?;
    Ok(url)
}

#[cfg(test)]
mod tests
{
    use std::default::Default;
    use axum::http::Request;
    use crate::mods::inline_keyboards::types::SearchTarget;
    
    use crate::mods::net::find_by_key;
    
    use super::*;
    
    // TODO: Finish or remove
    #[tokio::test]
    async fn default_url_test() -> eyre::Result<()>
    {
        let for_user = "user 47";
        let url = default_auth_url(for_user).await?;
        let query_str = url.query().ok_or(eyre::eyre!("No Query"))?;
        let q_pairs: Vec<(_, _)> = url.query_pairs().collect();
        let d_q: Vec<(_, _)> = form_urlencoded::parse(query_str.as_bytes()).collect();
        let decoded_query: String =
            form_urlencoded::parse(query_str.as_bytes())
                .map(|(k, v)| [&k, "=", &v, "&"].concat())
                .collect();
        let req = Request::builder().uri(url.as_str()).body(())?;
        let v: Vec<(_, _)> = form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes()).collect();
        let v2: String =
            form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes())
                .map(|(k, v)| [&k, "=", &v, "&"].concat())
                .collect();
        
        dbg!(&url);
        dbg!(query_str);
        dbg!(&decoded_query);
        dbg!(&q_pairs);
        dbg!(&d_q);
        dbg!(&req.uri().query().unwrap_or(""));
        dbg!(&v);
        
        let state = find_by_key(&v2, "&", "state")?;
        let state_code = find_by_key(state, "xplusx", "state_code")?;
        let contains_code = state_code.contains("liuhw9p38y08q302q02h0gp9g0p2923924u0s");
        let extracted_for_user = find_by_key(state, "xplusx", "for_user")?;
        
        assert_eq!(for_user, extracted_for_user);
        assert!(decoded_query.contains(for_user));
        assert!(contains_code);
        
        Ok(())
    }
    
    #[test]
    fn parse_number_test()
    {
        let text = "48";
        let d_data = DialogueData::default();
        let search_config = SearchConfigData { target: SearchTarget::Subscription.into(), ..Default::default() };
        let config = Either::<_, &ListConfigData>::First(&search_config);
        let res = parse_number(text, config, &d_data);
        let expected = "Accepted! ✅".to_owned();
        assert_eq!(res.0, expected);
        assert!(matches!(res.1, None));
        assert!(matches!(res.2, Some(DialogueData { state: SearchCommandActive(SearchConfigData { target: Some(SearchTarget::Subscription), result_limit: Some(48), .. }), ..})));
    }
}


