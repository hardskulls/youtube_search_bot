use google_youtube3::{api::Subscription, oauth2::read_application_secret};

use teloxide::
{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{InlineKeyboardMarkup, Message, ParseMode}
};
use url::Url;
use crate::mods::db::get_access_token;

use crate::mods::dialogue::types::{DialogueData, ListConfigData, SearchConfigData, State::{self, ListCommandActive, SearchCommandActive}, Either};
use crate::mods::inline_keyboards::types::SearchMode;
use crate::mods::youtube::{list_subscriptions, make_auth_url};
use crate::mods::youtube::types::{ACCESS_TYPE, RESPONSE_TYPE, SCOPE_YOUTUBE_READONLY, YouTubeAccessToken};

pub(crate) fn parse_number(text: &str, configs: Either<&SearchConfigData, &ListConfigData>, dialogue_data: &DialogueData)
    -> (String, Option<InlineKeyboardMarkup>, Option<DialogueData>)
{
    match text.parse::<u32>()
    {
        Ok(num) if num > 1 =>
            {
                let state =
                    match configs
                    {
                        Either::First(search_config) =>
                            SearchCommandActive(SearchConfigData { result_limit: num.into(), ..search_config.clone() }),
                        Either::Last(list_config) =>
                            ListCommandActive(ListConfigData { result_limit: num.into(), ..list_config.clone() })
                    };
                ("Accepted! ✅".to_owned(), None, DialogueData { state, ..dialogue_data.clone() }.into())
            }
        _ => ("Send a number greater than 0".to_owned(), None, None)
    }
}

pub(crate) async fn execute_search
(
    bot: &Bot,
    msg: &Message,
    dialogue_data: &DialogueData,
    text_to_look_for: &str,
    result_lim: u32,
    search_mode: &SearchMode,
)
    -> eyre::Result<(String, Option<InlineKeyboardMarkup>, Option<DialogueData>)>
{
    let user_id = msg.from().unwrap().full_name();
    let access_token =
        match get_access_token(&user_id)
        {
            Ok(YouTubeAccessToken { access_token: Some(token), .. }) => token,
            _ =>
                {
                    let url = default_auth_url(&user_id).await?;
                    let auth_url = format!("Use this link to log in <a href=\"{}\">{}</a>", url, "Log In");
                    bot.send_message(msg.chat.id, auth_url).parse_mode(ParseMode::Html).await?;
                    return Ok(("Please, log in first ".to_owned(), None, None))
                }
        };
    bot.send_message(msg.chat.id, "Searching, please wait 🕵️‍♂️").await?;
    
    let subscription_list = get_subs_list(search_mode, text_to_look_for, &access_token, result_lim).await?;
    
    for s in subscription_list.into_iter().take(result_lim as usize)
    {
        let snip = s.snippet.unwrap();
        let (title, descr, chan_id) =
            (snip.channel_title.unwrap(), snip.description.unwrap(), snip.resource_id.unwrap().channel_id.unwrap());
        let text = format!("Title: {} \n\n Description: {} \n\n Link: youtube.com/channel/{}", title, descr, chan_id);
        let _sent_msg = bot.send_message(msg.chat.id, text).await;
        log::info!(" [:: LOG ::] ... : ( @:[fn::execute_search] '_sent_msg' is [| '{:#?}' |] )", &_sent_msg);
    }

    Ok(("Finished! ✔".to_owned(), None, Some(DialogueData { state: State::Starting, ..dialogue_data.clone() })))
}

async fn default_auth_url(user_id: &str) -> eyre::Result<Url>
{
    let secret_path = std::env::var("OAUTH_SECRET_PATH").unwrap();
    let secret = read_application_secret(secret_path).await?;

    let (client_id, redirect_uri) = (secret.client_id.as_str(), secret.redirect_uris[0].as_str());
    let (scope, response_type) = (&[SCOPE_YOUTUBE_READONLY], RESPONSE_TYPE);
    let state = format!("for_user={u}xplusxstate_code=liuhw9p38y08q302q02h0gp9g0p2923924u0s", u = user_id);
    let optional_params = &[("ACCESS_TYPE".to_owned().to_lowercase(), ACCESS_TYPE), ("state".to_owned(), state.as_str())];

    let url = make_auth_url(client_id, redirect_uri, response_type, scope, optional_params)?;
    Ok(url)
}

pub(crate) async fn get_subs_list
(
    search_mode: &SearchMode,
    text_to_look_for: &str,
    access_token: &str,
    max_res: u32
)
    -> eyre::Result<Vec<Subscription>>
{
    log::info!(" [:: LOG ::] ... : ( @:[fn::get_subs_list] started )");
    log::info!
    (
        " [:: LOG ::] ... : ( @:[fn::get_subs_list] FIRST 'subs_list_resp' is [| '{:?}' |] )",
        (&search_mode, &text_to_look_for, &max_res)
    );
    let client = reqwest::Client::new();
    let subs_list_resp =
        list_subscriptions(&client, None, access_token).await.unwrap_or_default();
    log::info!
    (
        " [:: LOG ::] ... : ( @:[fn::list_subscriptions] FIRST 'subs_list_resp' is [| '{:?}' |] )",
        (
            subs_list_resp.next_page_token.as_ref(), subs_list_resp.page_info.as_ref(),
            subs_list_resp.items.as_ref().unwrap_or(&vec![]).len()
        )
    );
    let mut store_in: Vec<Subscription> = Vec::new();
    
    if let Some(items) = subs_list_resp.items
    { find_matches(search_mode, &mut store_in, items, text_to_look_for); }

    let mut next_page_token = subs_list_resp.next_page_token.clone();
    while next_page_token.is_some()
    {
        let subscription_list_resp =
            list_subscriptions(&client, next_page_token, access_token).await.unwrap_or_default();

        next_page_token = subscription_list_resp.next_page_token.clone();

        if let Some(items) = subscription_list_resp.items
        { find_matches(search_mode, &mut store_in, items, text_to_look_for); }
        
        if store_in.len() >= max_res as usize
        { next_page_token = None }
    }
    log::info!(" [:: LOG ::] ... : ( @:[fn::get_subs_list] FINAL 'store_in.len()' is [| '{:#?}' |] )", (&store_in.len()));
    log::info!(" [:: LOG ::] ... : ( @:[fn::get_subs_list] ended )");
    Ok(store_in)
}

fn find_matches(search_mode: &SearchMode, store_in: &mut Vec<Subscription>, search_in: Vec<Subscription>, text_to_look_for: &str)
{
    log::info!(" [:: LOG ::] ... : ( @:[fn::find_matches] started )");
    log::info!(" [:: LOG ::] ... : ( @:[fn::find_matches] 'store_in.len()' is [| '{:#?}' |] )", (&store_in.len()));
    let text_to_search = text_to_look_for.to_lowercase();
    log::info!(" [:: LOG ::] ... : ( @:[fn::find_matches] 'text_to_search' is [| '{:#?}' |] )", (&text_to_search));
    for sub in search_in
    {
        log::info!(" [:: LOG ::] ... : ( @:[fn::find_matches] 'sub' is [| '{:#?}' |] )", (&sub));
        let snip = sub.snippet.as_ref().unwrap();
        let compare_by = if let &SearchMode::Title = search_mode { snip.channel_title.as_ref() } else { snip.description.as_ref() };
        log::info!(" [:: LOG ::] ... : ( @:[fn::find_matches] 'compare_by' is [| '{:#?}' |] )", (&compare_by));

        if let Some(title_or_descr) = compare_by
        { if title_or_descr.to_lowercase().contains(&text_to_search) { store_in.push(sub) } }
    }
    log::info!(" [:: LOG ::] ... : ( @:[fn::find_matches] 'store_in.len()' is [| '{:#?}' |] )", (&store_in.len()));
    log::info!(" [:: LOG ::] ... : ( @:[fn::find_matches] ended )");
}

#[cfg(test)]
mod tests
{
    use axum::http::Request;
    use crate::mods::net::find_by_key;
    use super::*;
    
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
}


