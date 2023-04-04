
use error_traits::WrapInOk;
use google_youtube3::api::{Playlist, PlaylistListResponse, Subscription, SubscriptionListResponse};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

use crate::model::youtube::traits::{IntoSearchableItem, Searchable};

// TODO : Choose a better naming and description.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RespTargetSubscriptions;

// TODO : Choose a better naming and description.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RespTargetPlaylists;

// TODO : Choose a better naming.
/// Trait for building 'list' request in YouTube API.
pub(crate) trait YouTubeApiListRequestBuilder
{
    type Target : serde::de::DeserializeOwned;
    
    fn build_req(&self, client : &Client, access_token : &str, page_token : Option<String>) -> eyre::Result<RequestBuilder>;
}

impl YouTubeApiListRequestBuilder for RespTargetSubscriptions
{
    type Target = SubscriptionListResponse;
    
    fn build_req(&self, client : &Client, access_token : &str, page_token : Option<String>)
        -> eyre::Result<RequestBuilder>
    {
        let mut req =
            client
                .get(reqwest::Url::parse("https://www.googleapis.com/youtube/v3/subscriptions")?)
                .query(&[("part", "snippet,contentDetails"), ("maxResults", "50"), ("mine", "true")])
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                .header(reqwest::header::ACCEPT, "application/json");
        if let Some(page) = page_token
        { req = req.query(&[("pageToken", &page)]) }
        req.in_ok()
    }
}

impl YouTubeApiListRequestBuilder for RespTargetPlaylists
{
    type Target = PlaylistListResponse;
    
    fn build_req(&self, client : &Client, access_token : &str, page_token : Option<String>)
        -> eyre::Result<RequestBuilder>
    {
        let mut req =
            client
                .get(reqwest::Url::parse("https://youtube.googleapis.com/youtube/v3/playlists")?)
                .query(&[("part", "snippet,contentDetails"), ("maxResults", "50"), ("mine", "true")])
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                .header(reqwest::header::ACCEPT, "application/json");
        if let Some(page) = page_token
        { req = req.query(&[("pageToken", &page)]) }
        req.in_ok()
    }
}

//pub(crate) struct ItemSearchRes<S : Searchable>
//{
//    pub(crate) items : Option<Vec<S>>,
//    pub(crate) next_page_token : Option<String>
//}

/// Trait represents a page of response from request to YouTube API.
pub(crate) trait YouTubeApiResponsePage
{
    type Item : Searchable + IntoSearchableItem;
    
    fn next_page_token(&self) -> Option<String>;
    
    fn total_results(&self) -> Option<u32>;
    
    fn items(self) -> Option<Vec<Self::Item>>;
}

impl YouTubeApiResponsePage for SubscriptionListResponse
{
    type Item = Subscription;
    
    fn next_page_token(&self) -> Option<String>
    {
        self.next_page_token.clone()
    }
    
    fn total_results(&self) -> Option<u32>
    {
        self.page_info.as_ref()?.total_results?.try_into().ok()
    }
    
    fn items(self) -> Option<Vec<Self::Item>>
    {
        self.items
    }
}

impl YouTubeApiResponsePage for PlaylistListResponse
{
    type Item = Playlist;
    
    fn next_page_token(&self) -> Option<String>
    {
        self.next_page_token.clone()
    }
    
    fn total_results(&self) -> Option<u32>
    {
        self.page_info.as_ref()?.total_results?.try_into().ok()
    }
    
    fn items(self) -> Option<Vec<Self::Item>>
    {
        self.items
    }
}


