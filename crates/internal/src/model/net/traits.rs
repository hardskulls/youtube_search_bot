
use error_traits::WrapInRes;
use google_youtube3::api::{Playlist, PlaylistItem, PlaylistItemListResponse, PlaylistListResponse, Subscription, SubscriptionListResponse};
use reqwest::{Client, RequestBuilder};
use crate::model::net::types::{PlaylistItemRequester, PlaylistRequester, SubscriptionRequester, YOUTUBE_PLAYLIST_ITEMS_API, YOUTUBE_PLAYLISTS_API, YOUTUBE_SUBSCRIPTIONS_API};
use crate::model::youtube::traits::{IntoSearchableItem, Searchable};


// TODO : Choose a better naming.
/// Trait for building 'list' request in YouTube API.
pub(crate) trait YouTubeApiRequestBuilder
{
    type Target: serde::de::DeserializeOwned;
    
    fn build_req(&self, client: &Client, access_token: &str, page_token: Option<String>)
        -> eyre::Result<RequestBuilder>;
}

impl YouTubeApiRequestBuilder for SubscriptionRequester
{
    type Target = SubscriptionListResponse;
    
    fn build_req(&self, client: &Client, access_token: &str, page_token: Option<String>)
        -> eyre::Result<RequestBuilder>
    {
        let mut req =
            client.get(reqwest::Url::parse(YOUTUBE_SUBSCRIPTIONS_API)?)
                .query(&[("part", "contentDetails,id,snippet,status"), ("maxResults", "50"), ("mine", "true")])
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                .header(reqwest::header::ACCEPT, "application/json");
        if let Some(page) = page_token
        { req = req.query(&[("pageToken", &page)]) }
        req.in_ok()
    }
}

impl YouTubeApiRequestBuilder for PlaylistRequester
{
    type Target = PlaylistListResponse;
    
    fn build_req(&self, client: &Client, access_token: &str, page_token: Option<String>)
        -> eyre::Result<RequestBuilder>
    {
        let mut req =
            client.get(reqwest::Url::parse(YOUTUBE_PLAYLISTS_API)?)
                .query(&[("part", "contentDetails,id,snippet,status"), ("maxResults", "50"), ("mine", "true")])
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                .header(reqwest::header::ACCEPT, "application/json");
        if let Some(page) = page_token
        { req = req.query(&[("pageToken", &page)]) }
        req.in_ok()
    }
}

impl<'a> YouTubeApiRequestBuilder for PlaylistItemRequester<'a>
{
    type Target = PlaylistItemListResponse;
    
    fn build_req(&self, client: &Client, access_token: &str, page_token: Option<String>)
        -> eyre::Result<RequestBuilder>
    {
        let mut req =
            client.get(reqwest::Url::parse(YOUTUBE_PLAYLIST_ITEMS_API)?)
                .query(&[("part", "contentDetails,id,snippet,status"), ("maxResults", "50")])
                .query(&("playlistId", self.playlist_id))
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
    type Item: Searchable + IntoSearchableItem;
    
    fn next_page_token(&self) -> Option<String>;
    
    fn total_results(&self) -> Option<u32>;
    
    fn items(self) -> Option<Vec<Self::Item>>;
}

impl YouTubeApiResponsePage for SubscriptionListResponse
{
    type Item = Subscription;
    
    fn next_page_token(&self) -> Option<String>
    { self.next_page_token.clone() }
    
    fn total_results(&self) -> Option<u32>
    { self.page_info.as_ref()?.total_results?.try_into().ok() }
    
    fn items(self) -> Option<Vec<Self::Item>>
    { self.items }
}

impl YouTubeApiResponsePage for PlaylistListResponse
{
    type Item = Playlist;
    
    fn next_page_token(&self) -> Option<String>
    { self.next_page_token.clone() }
    
    fn total_results(&self) -> Option<u32>
    { self.page_info.as_ref()?.total_results?.try_into().ok() }
    
    fn items(self) -> Option<Vec<Self::Item>>
    { self.items }
}

impl YouTubeApiResponsePage for PlaylistItemListResponse
{
    type Item = PlaylistItem;
    
    fn next_page_token(&self) -> Option<String>
    { self.next_page_token.clone() }
    
    fn total_results(&self) -> Option<u32>
    { self.page_info.as_ref()?.total_results?.try_into().ok() }
    
    fn items(self) -> Option<Vec<Self::Item>>
    { self.items }
}


