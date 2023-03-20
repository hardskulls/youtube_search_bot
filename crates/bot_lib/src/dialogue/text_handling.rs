use std::fmt::Debug;

use google_youtube3::oauth2::read_application_secret;
use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{ChatId, ParseMode, User};
use url::Url;

use error_traits::WrapInOk;

use crate::db::{get_access_token, refresh_access_token, refresh_token_req};
use crate::dialogue::types::{DialogueData, Either, ListCommandSettings, MessageTriplet, SearchCommandSettings};
use crate::dialogue::types::State::{ListCommandActive, SearchCommandActive};
use crate::keyboards::types::{SearchIn, Sorting};
use crate::net::traits::{YouTubeApiListRequestBuilder, YouTubeApiResponsePage};
use crate::utils::HTMLise;
use crate::youtube::{list_items, make_auth_url, search_items};
use crate::youtube::traits::Searchable;
use crate::youtube::types::{ACCESS_TYPE, RESPONSE_TYPE, SCOPE_YOUTUBE_READONLY};

/// Helper function used for `handle_text` handler.
/// Parses user input as number in order to set it as `result limit` setting.
pub(crate) fn parse_number(text: &str, configs: Either<&SearchCommandSettings, &ListCommandSettings>, dialogue_data: &DialogueData)
    -> MessageTriplet
{
    match text.parse::<u16>()
    {
        Ok(num) if num > 1 =>
            {
                let state =
                    match configs
                    {
                        Either::First(search_settings) =>
                            SearchCommandActive(SearchCommandSettings { result_limit: (num as u32).into(), ..search_settings.clone() }),
                        Either::Last(list_settings) =>
                            ListCommandActive(ListCommandSettings { result_limit: (num as u32).into(), ..list_settings.clone() })
                    };
                ("Accepted! ✅".to_owned(), None, Some(DialogueData { state, ..dialogue_data.clone() }))
            }
        _ => ("Send a number greater than 0".to_owned(), None, None)
    }
}

/// Save text to search.
pub(crate) fn save_text(text: &str, search_settings: SearchCommandSettings, dialogue_data: &DialogueData)
    -> MessageTriplet
{
    let state = SearchCommandActive(SearchCommandSettings { text_to_search: text.to_owned().into(), ..search_settings });
    ("Accepted! ✅".to_owned(), None, Some(DialogueData { state, ..dialogue_data.clone() }))
}

/// Send 'search' and 'list' results to user.
async fn send_results<'i, S, T>(bot: &Bot, send_to: ChatId, list: T)
    where
        S : Searchable + 'i,
        T : IntoIterator<Item = &'i S>
{
    for s in list.into_iter()
    {
        let (title, descr) = (s.title().unwrap_or("No title 🤷‍♂️"), s.description().unwrap_or("No description 🤷‍♂️️"));
        let link = s.link().unwrap_or_else(|| "No link 🤷‍♂️".to_owned());
        let text = format!("{}{}{}", title.to_bold() + " \n\n", descr.to_owned() + " \n\n", link);
        let _sent_msg =
            bot.send_message(send_to, text)
                .parse_mode(ParseMode::Html)
                .disable_web_page_preview(true)
                .await;
        log::info!(" [:: LOG ::]    ( @:[fn::text_handling::send_results] '_sent_msg' is [| '{:#?}' |] )", &_sent_msg);
    }
}

/// Helper function used for `handle_text` handler.
/// Final func that does searching when everything is ready. 
pub(crate) async fn execute_search_command<T>
(
    bot: &Bot,
    user_id: User,
    send_to: ChatId,
    search_for: &str,
    res_limit: u32,
    search_in: &SearchIn,
    req_builder: T
)
    -> eyre::Result<MessageTriplet>
    where
        T : YouTubeApiListRequestBuilder + Send,
        T::Target : Default + Debug + YouTubeApiResponsePage + Send,
        <T::Target as YouTubeApiResponsePage>::Item : Clone,
{
    let user_id = user_id.id.0.to_string();
    let db_url = env!("REDIS_URL");
    let Ok(token) =
        get_access_token(&user_id, db_url)
        else
        {
            let auth_url = format!("Use this link to log in {}", default_auth_url(&user_id).await?.to_link("Log In"));
            bot.send_message(send_to, auth_url).parse_mode(ParseMode::Html).await?;
            return ("Please, log in and send your text again ".to_owned(), None, None).in_ok()
        };
    
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
    let secret = read_application_secret(secret_path).await?;
    let token_req = refresh_token_req(secret, &token)?;
    let access_token = refresh_access_token(&user_id, token, db_url, token_req).await?.access_token;
    
    bot.send_message(send_to, "Searching, please wait 🕵️‍♂️").await?;
    let results = search_items(search_in, req_builder, search_for, &access_token, res_limit).await;
    
    send_results(bot, send_to, &results).await;
    let result_count = results.len();
    
    (format!("Finished! ✔ \nFound {result_count} results"), None, None).in_ok()
}

pub(crate) async fn execute_list_command<T>
(
    bot: &Bot,
    user_id: User,
    send_to: ChatId,
    res_limit: u32,
    sorting: &Sorting,
    req_builder: T
)
    -> eyre::Result<MessageTriplet>
    where
        T : YouTubeApiListRequestBuilder,
        T::Target : Default + Debug + YouTubeApiResponsePage
{
    let user_id = user_id.id.0.to_string();
    let db_url = env!("REDIS_URL");
    let Ok(token) =
        get_access_token(&user_id, db_url)
        else
        {
            let auth_url = format!("Use this link to log in {}", default_auth_url(&user_id).await?.to_link("Log In"));
            bot.send_message(send_to, auth_url).parse_mode(ParseMode::Html).await?;
            return Ok(("Please, log in and send your text again ".to_owned(), None, None))
        };
    
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
    let secret = read_application_secret(secret_path).await?;
    let token_req = refresh_token_req(secret, &token)?;
    let access_token = refresh_access_token(&user_id, token, db_url, token_req).await?.access_token;
    
    bot.send_message(send_to, "Searching, please wait 🕵️‍♂️").await?;
    let results = list_items(req_builder, &access_token, sorting, res_limit).await;
    
    send_results(bot, send_to, &results).await;
    let result_count = results.len();
    
    (format!("Finished! ✔ \nFound {result_count} results"), None, None).in_ok()
}

/// Construct authorization url.
async fn default_auth_url(user_id: &str) -> eyre::Result<Url>
{
    let secret_path = env!("PATH_TO_GOOGLE_OAUTH_SECRET");
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
    
    use crate::dialogue::types::{DialogueData, Either, ListCommandSettings, SearchCommandSettings};
    use crate::keyboards::types::Target;
    use crate::net::find_by_key;
    
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
        let search_config = SearchCommandSettings { target: Target::Subscription.into(), ..Default::default() };
        let config = Either::<_, &ListCommandSettings>::First(&search_config);
        let res = parse_number(text, config, &d_data);
        let expected = "Accepted! ✅".to_owned();
        assert_eq!(res.0, expected);
        assert!(matches!(res.1, None));
        assert!(matches!(res.2, Some(DialogueData { state: SearchCommandActive(SearchCommandSettings { target: Some(Target::Subscription), result_limit: Some(48), .. }), ..})));
    }
}


