use google_youtube3::api::{Subscription, SubscriptionListResponse};
use google_youtube3::YouTube;
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use teloxide::
{
    Bot,
    types::{InlineKeyboardMarkup, Message},
};
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::ParseMode;
use url::{ParseError, Url};

use KeyBoard::{ListCommand, SearchCommand};
use State::{ListCommandActive, SearchCommandActive};

use crate::mods::
{
    dialogue::helpers::{edit_keyboard, get_callback_data, get_dialogue_data, get_text},
    dialogue::helpers::update_state_and_send_message,
    dialogue::types::{DialogueData, ListConfigData, SearchConfigData, State, TheDialogue},
    errors::EndpointErrors,
    inline_keyboards::funcs::{CreateKB, KeyboardText},
    inline_keyboards::types::{KeyBoard, ListCommandKB, SearchCommandKB},
};
use crate::mods::inline_keyboards::types::SearchMode;
use crate::mods::youtube::{search_subs, youtube_service};
use crate::mods::youtube::helpers::make_auth_url;
use crate::mods::youtube::types::{ACCESS_TYPE, CLIENT_ID, REDIRECT_URI, RESPONSE_TYPE, SCOPE_YOUTUBE, SCOPE_YOUTUBE_READONLY};

pub(crate) type YouTubeService = YouTube<HttpsConnector<HttpConnector>>;

pub async fn handle_start_state(bot: Bot, msg: Message) -> eyre::Result<()>
{
    bot.send_message(msg.chat.id, "Bot is running! 🚀 \nSend /start_game command to start a game 🕹").await?;
    Ok(())
}

pub async fn handle_text(bot: Bot, msg: Message, dialogue: TheDialogue) -> eyre::Result<()>
{
    let dialogue_data = get_dialogue_data(&dialogue).await?;
    let callback = dialogue_data.last_callback.as_ref().ok_or(EndpointErrors::GameError)?.clone();
    let keyboard: KeyBoard = serde_json::from_str(&get_callback_data(&callback).await?)?;
    let text = get_text(&msg).await?;
    let (message_text, opt_keyboard, opt_dialogue_data): (String, Option<InlineKeyboardMarkup>, Option<DialogueData>) =
        match (dialogue_data.state.as_ref(), keyboard)
        {
            (State::Starting, ..) => ("Set your search config first ".to_owned(), None, None),
            (SearchCommandActive(search_config), SearchCommand(SearchCommandKB::ResultLimit)) =>
                match text.parse::<u32>()
                {
                    Ok(num) if num > 1 =>
                        {
                            let state = SearchCommandActive(SearchConfigData { result_limit: Some(num), ..search_config.clone() });
                            let (kb, callback) = (SearchCommandKB::SearchConfig, dialogue_data.last_callback.as_ref().unwrap());
                            edit_keyboard(&bot, kb.keyboard_text(), kb.create_kb(), callback).await?;
                            ("Accepted! ✅".to_owned(), None, DialogueData { state, ..dialogue_data.clone() }.into())
                        }
                    _ => ("Send a number greater than 0". to_owned(), None, None)
                },
            (ListCommandActive(list_config), ListCommand(ListCommandKB::ResultLimit)) =>
                match text.parse::<u32>()
                {
                    Ok(num) if num > 1 =>
                        {
                            let state = ListCommandActive(ListConfigData { result_limit: Some(num), ..list_config.clone() });
                            let (kb, callback) = (ListCommandKB::ListConfig, dialogue_data.last_callback.as_ref().unwrap());
                            edit_keyboard(&bot, kb.keyboard_text(), kb.create_kb(), callback).await?;
                            ("Accepted! ✅".to_owned(), None, DialogueData { state, ..dialogue_data.clone() }.into())
                        }
                    _ => ("Send a number greater than 0". to_owned(), None, None)
                }
            (SearchCommandActive(SearchConfigData { search_by: Some(s), target: Some(_), result_limit: Some(r) }), _) =>
                execute_search(&bot, &msg, &dialogue_data, text, *r, s).await?,
            (ListCommandActive(ListConfigData { sort_by: Some(_), target: Some(_), filter: Some(_), result_limit: Some(r) }), _) =>
                execute_search(&bot, &msg, &dialogue_data, text, *r, &SearchMode::Title).await?,
            _ => ("Oops!".to_owned(), None, None)
        };
    update_state_and_send_message(dialogue.into(), opt_dialogue_data, opt_keyboard, bot, msg.chat.id, message_text).await?;
    Ok(())
}

async fn execute_search(bot: &Bot, msg: &Message, dialogue_data: &DialogueData, text_to_search: &str, result_lim: u32, search_mode: &SearchMode)
    -> eyre::Result<(String, Option<InlineKeyboardMarkup>, Option<DialogueData>)>
{
    let url = default_auth_url()?;
    bot.send_message(msg.chat.id, format!("Use this link to log in <a href=\"{}\">{}</a>", url, "Log In"))
        .parse_mode(ParseMode::Html).await?;
    let hub = youtube_service("youtube_search_bot/crates/secret.json").await?;
    let first_response = search_subs(&hub, 50).await?;
    let mut res = vec![];
    let _ = get_subs_list(&hub, first_response.1, search_mode, &mut res, text_to_search).await;
    for s in res.iter().take(result_lim as _)
    {
        let snip = s.snippet.as_ref().unwrap();
        let (title, descr, chan_id) =
            (snip.channel_title.as_ref(), snip.description.as_ref(), snip.resource_id.as_ref().unwrap().channel_id.as_ref());
        let text = format!("Title: {} \n\n Description: {} \n\n Link: youtube.com/channel/{}", title.unwrap(), descr.unwrap(), chan_id.unwrap());
        let _ = bot.send_message(msg.chat.id, text).await;
    }
    Ok(("Finished! ✔".to_owned(), None, Some(DialogueData { state: State::Starting, ..dialogue_data.clone() })))
}

fn default_auth_url() -> Result<Url, ParseError>
{
    let scopes = &[SCOPE_YOUTUBE, SCOPE_YOUTUBE_READONLY];
    let optional_params = &[("ACCESS_TYPE".to_owned().to_lowercase(), ACCESS_TYPE)];
    let (client_id, redirect_uri, response_type) = (CLIENT_ID, REDIRECT_URI, RESPONSE_TYPE);
    make_auth_url(client_id, redirect_uri, response_type, scopes, optional_params)
}

async fn get_subs_list<'a, 'b>
(
    youtube_hub: &YouTubeService,
    subs_list: SubscriptionListResponse,
    search_mode: &SearchMode,
    store_in: &mut Vec<Subscription>,
    text_to_search: &str
)
    -> eyre::Result<()>
where 'b: 'a
{
    if let Some(items) = subs_list.items
    { find_matches(search_mode, store_in, items, text_to_search); }
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
        { find_matches(search_mode, store_in, items, text_to_search); }
    }
    Ok(())
}

fn find_matches(search_mode: &SearchMode, store_in: &mut Vec<Subscription>, search_in: Vec<Subscription>, text: &str)
{
    let text_to_search = text.to_lowercase();
    for sub in search_in
    {
        let snip = sub.snippet.as_ref().unwrap();
        let compare_by = if let &SearchMode::Title = search_mode { snip.channel_title.as_ref() } else { snip.description.as_ref() };
        if let Some(title_or_descr) = compare_by
        { if title_or_descr.to_lowercase().contains(&text_to_search) { store_in.push(sub) } }
    }
}


