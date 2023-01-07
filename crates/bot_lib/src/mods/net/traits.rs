use google_youtube3::api::{Playlist, PlaylistListResponse, Subscription, SubscriptionListResponse};
use reqwest::{Client, RequestBuilder};
use error_traits::InOk;
use crate::mods::youtube::traits::Searchable;


pub struct ListSubscriptions;

pub struct ListPlaylists;


pub trait ItemsListRequestBuilder
{
    type Target: serde::de::DeserializeOwned;
    
    fn build_req(&self, client: &Client, access_token: &str, next_page_tok: Option<String>) -> eyre::Result<RequestBuilder>;
}

impl ItemsListRequestBuilder for ListSubscriptions
{
    type Target = SubscriptionListResponse;
    
    fn build_req(&self, client: &Client, access_token: &str, next_page_tok: Option<String>)
        -> eyre::Result<RequestBuilder>
    {
        let mut req =
            client
                .get(reqwest::Url::parse("https://www.googleapis.com/youtube/v3/subscriptions")?)
                .query(&[("part", "snippet,contentDetails"), ("maxResults", "50"), ("mine", "true")])
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                .header(reqwest::header::ACCEPT, "application/json");
        if let Some(page) = next_page_tok
        { req = req.query(&[("pageToken", &page)]) }
        req.in_ok()
    }
}

impl ItemsListRequestBuilder for ListPlaylists
{
    type Target = PlaylistListResponse;
    
    fn build_req(&self, client: &Client, access_token: &str, next_page_tok: Option<String>)
        -> eyre::Result<RequestBuilder>
    {
        let mut req =
            client
                .get(reqwest::Url::parse("https://youtube.googleapis.com/youtube/v3/playlists")?)
                .query(&[("part", "snippet,contentDetails"), ("maxResults", "50"), ("mine", "true")])
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                .header(reqwest::header::ACCEPT, "application/json");
        if let Some(page) = next_page_tok
        { req = req.query(&[("pageToken", &page)]) }
        req.in_ok()
    }
}

pub struct ItemSearchRes<S: Searchable>
{
    pub items: Option<Vec<S>>,
    pub next_page_token: Option<String>
}


pub trait ItemsResponsePage
{
    type Item: Searchable;
    
    fn items_search_res(self) -> ItemSearchRes<Self::Item>;
}

impl ItemsResponsePage for SubscriptionListResponse
{
    type Item = Subscription;
    
    fn items_search_res(self) -> ItemSearchRes<Self::Item>
    {
        ItemSearchRes { items: self.items, next_page_token: self.next_page_token }
    }
}

impl ItemsResponsePage for PlaylistListResponse
{
    type Item = Playlist;
    
    fn items_search_res(self) -> ItemSearchRes<Self::Item>
    {
        ItemSearchRes { items: self.items, next_page_token: self.next_page_token }
    }
}


