use google_youtube3::{api::Subscription, oauth2::read_application_secret, api::SubscriptionListResponse};
use teloxide::
{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{InlineKeyboardMarkup, Message, ParseMode}
};
use url::Url;

use crate::mods::dialogue::types::{DialogueData, ListConfigData, SearchConfigData, State::{self, ListCommandActive, SearchCommandActive}, Either};
use crate::mods::inline_keyboards::types::SearchMode;
use crate::mods::youtube::{search_subs, youtube_service};
use crate::mods::youtube::funcs::make_auth_url;
use crate::mods::youtube::types::{ACCESS_TYPE, RESPONSE_TYPE, SCOPE_YOUTUBE_READONLY, YouTubeService};

pub(crate) fn parse_number(text: &str, configs: Either<&SearchConfigData, &ListConfigData>, dialogue_data: &DialogueData)
    -> (String, Option<InlineKeyboardMarkup>, Option<DialogueData>)
{
    match text.parse::<u32>()
    {
        Ok(num) if num > 1 =>
            {
                let result_limit = num.into();
                let state =
                    match configs
                    {
                        Either::First(search_config) => SearchCommandActive(SearchConfigData { result_limit, ..search_config.clone() }),
                        Either::Last(list_config) => ListCommandActive(ListConfigData { result_limit, ..list_config.clone() })
                    };
                ("Accepted! ✅".to_owned(), None, DialogueData { state, ..dialogue_data.clone() }.into())
            }
        _ => ("Send a number greater than 0".to_owned(), None, None)
    }
}

pub(crate) async fn execute_search(bot: &Bot, msg: &Message, dialogue_data: &DialogueData, text_to_look_for: &str, result_lim: u32, search_mode: &SearchMode)
    -> eyre::Result<(String, Option<InlineKeyboardMarkup>, Option<DialogueData>)>
{
    let provide_user_auth_url = format!("Use this link to log in <a href=\"{}\">{}</a>", default_auth_url().await?, "Log In");
    bot.send_message(msg.chat.id, provide_user_auth_url).parse_mode(ParseMode::Html).await?;

    let hub = youtube_service("client_secret_web_client_for_youtube_search_bot.json").await?;
    let (_, subs_list_resp): (_, SubscriptionListResponse) = search_subs(&hub, 50).await?;

    let capacity = subs_list_resp.page_info.as_ref().unwrap().total_results.as_ref().unwrap_or(&20);
    let mut subscription_list = Vec::with_capacity(*capacity as usize);

    let _ = get_subs_list(&hub, subs_list_resp, search_mode, &mut subscription_list, text_to_look_for).await;
    for s in subscription_list.iter().take(result_lim as _)
    {
        let snip = s.snippet.as_ref().unwrap();
        let (title, descr, chan_id) =
            (snip.channel_title.as_ref(), snip.description.as_ref(), snip.resource_id.as_ref().unwrap().channel_id.as_ref());
        let text = format!("Title: {} \n\n Description: {} \n\n Link: youtube.com/channel/{}", title.unwrap(), descr.unwrap(), chan_id.unwrap());
        let _ = bot.send_message(msg.chat.id, text).await;
    }

    Ok(("Finished! ✔".to_owned(), None, Some(DialogueData { state: State::Starting, ..dialogue_data.clone() })))
}

async fn default_auth_url() -> eyre::Result<Url>
{
    let secret = read_application_secret("client_secret_web_client_for_youtube_search_bot.json").await?;

    let (client_id, redirect_uri) = (secret.client_id.as_str(), secret.redirect_uris[0].as_str());
    let (scope, response_type) = (&[SCOPE_YOUTUBE_READONLY], RESPONSE_TYPE);
    let optional_params = &[("ACCESS_TYPE".to_owned().to_lowercase(), ACCESS_TYPE)];

    let url = make_auth_url(client_id, redirect_uri, response_type, scope, optional_params)?;
    Ok(url)
}

pub(crate) async fn get_subs_list
(
    youtube_hub: &YouTubeService,
    subs_list: SubscriptionListResponse,
    search_mode: &SearchMode,
    store_in: &mut Vec<Subscription>,
    text_to_look_for: &str
)
    -> eyre::Result<()>
{
    if let Some(items) = subs_list.items
    { find_matches(search_mode, store_in, items, text_to_look_for); }

    let mut next_page_token = subs_list.next_page_token.clone();

    while let Some(page) = next_page_token
    {
        let (_, subscription_list_resp) =
            youtube_hub.subscriptions().list(&vec!["part".into()])
                .max_results(50)
                .mine(true)
                .page_token(&page)
                .doit()
                .await?;

        next_page_token = subscription_list_resp.next_page_token.clone();

        if let Some(items) = subscription_list_resp.items
        { find_matches(search_mode, store_in, items, text_to_look_for); }
    }
    Ok(())
}

fn find_matches(search_mode: &SearchMode, store_in: &mut Vec<Subscription>, search_in: Vec<Subscription>, text_to_look_for: &str)
{
    let text_to_search = text_to_look_for.to_lowercase();
    for sub in search_in
    {
        let snip = sub.snippet.as_ref().unwrap();
        let compare_by = if let &SearchMode::Title = search_mode { snip.channel_title.as_ref() } else { snip.description.as_ref() };

        if let Some(title_or_descr) = compare_by
        { if title_or_descr.to_lowercase().contains(&text_to_search) { store_in.push(sub) } }
    }
}


