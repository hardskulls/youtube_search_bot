use std::any::type_name;
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
                .query(&[("part", "contentDetails,id,snippet")])
                .query(&[("maxResults", "50"), ("mine", "true")])
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
                .query(&[("part", "contentDetails,id,snippet,status")])
                .query(&[("maxResults", "50"), ("mine", "true")])
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
        log::debug!("@:[fn::build_req]:[{}] <playlistId> is: {}", type_name::<Self::Target>(), self.playlist_id);
        
        let mut req =
            client.get(reqwest::Url::parse(YOUTUBE_PLAYLIST_ITEMS_API)?)
                .query(&[("part", "contentDetails,id,snippet,status")])
                .query(&[("maxResults", "50"), ("playlistId", self.playlist_id)])
                .header(reqwest::header::AUTHORIZATION, format!("Bearer {access_token}"))
                .header(reqwest::header::ACCEPT, "application/json");
        if let Some(page) = page_token
        { req = req.query(&[("pageToken", &page)]) }
        
        log::debug!("@:[fn::build_req] <req> is: {:?}", req);
        
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


#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use error_traits::PassErrWith;
    use reqwest::Client;
    use crate::model::net::types::{PlaylistItemRequester, PlaylistRequester, SubscriptionRequester};
    use std::io::Write;
    use env_logger::fmt::Formatter;
    use log::Record;
    use super::*;
    
    
    fn format_logs(buf: &mut Formatter, record: &Record) -> std::io::Result<()>
    {
        let file = record.file().unwrap_or("unknown file");
        let line = record.line().unwrap_or(0);
        let utc = chrono::Utc::now().format("DATE ~ %Y/%m/%d || TIME ~ %H:%M:%S");
        let local = chrono::Local::now().format("DATE ~ %Y/%m/%d || TIME ~ %H:%M:%S");
        let (level, args) = (record.level(), record.args());
        let separator = "===================================================================";
        
        writeln!
        (
            buf,
            "\
                \n{separator} \
                \n\nLOG : {level} \
                \n   ->   LOGGED AT ~ {file}:{line} \
                \n   ->   Local({local}) \
                \n   ->   Utc({utc}) \
                \n\n\n{args} \
                \n\n{separator}\n\n\n \
                "
        )
    }
    
    #[test]
    fn youtube_api_request_builder_test()
    {
        // Logger is initiated globally, ..
        // ..so in tests it should be initialized with `.try_init().ok()`
        env_logger::Builder::from_default_env()
            .format(format_logs)
            .try_init()
            .ok();
        
        let client = Client::new();
        let playlist_id = "Ljyu90PKhi75sDReqOVLaGpwTaogl68CjhmZWOKc";
        
        // Test `PlaylistItemRequester`.
        let (access_token, page_token) = ("gfTHy^75$367Frt%4dHHJytE$#A", None);
        let req_builder = PlaylistItemRequester { playlist_id };
        let _build =
            req_builder.build_req(&client, access_token, page_token)
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap()
                .build()
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap();
        
        // Test `PlaylistRequester`.
        let (req_builder, page_token) = (PlaylistRequester, None);
        let _build =
            req_builder.build_req(&client, access_token, page_token)
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap()
                .build()
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap();
        
        // Test `SubscriptionRequester`.
        let (req_builder, page_token) = (SubscriptionRequester, None);
        let _build =
            req_builder.build_req(&client, access_token, page_token)
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap()
                .build()
                .pass_err_with(|e| log::error!("error: {e:?}"))
                .unwrap();
    }
}


