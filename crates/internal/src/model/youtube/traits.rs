
use google_youtube3::api::{Playlist, Subscription};
use crate::model::youtube::types::SearchableItem;


/// Anything that can be searched on user's YouTube channel.
pub(crate) trait Searchable
{
    fn title(&self) -> Option<&str>;
    
    fn description(&self) -> Option<&str>;
    
    fn date(&self) -> Option<&str>;
    
    fn link(&self) -> Option<String>;
}

impl Searchable for Subscription
{
    fn title(&self) -> Option<&str>
    { self.snippet.as_ref()?.title.as_deref().filter(|i| !i.trim().is_empty()) }
    
    fn description(&self) -> Option<&str>
    { self.snippet.as_ref()?.description.as_deref().filter(|i| !i.trim().is_empty()) }
    
    fn date(&self) -> Option<&str>
    { self.snippet.as_ref()?.published_at.as_deref().filter(|i| !i.trim().is_empty()) }
    
    fn link(&self) -> Option<String>
    {
        let chan_id: &str = self.snippet.as_ref()?.resource_id.as_ref()?.channel_id.as_ref()?;
        if !chan_id.trim().is_empty()
        { format!("https://youtube.com/channel/{chan_id}").into() }
        else
        { None }
    }
}

impl Searchable for Playlist
{
    fn title(&self) -> Option<&str>
    { self.snippet.as_ref()?.title.as_deref().filter(|i| !i.trim().is_empty()) }
    
    fn description(&self) -> Option<&str>
    { self.snippet.as_ref()?.description.as_deref().filter(|i| !i.trim().is_empty()) }
    
    fn date(&self) -> Option<&str>
    { self.snippet.as_ref()?.published_at.as_deref().filter(|i| !i.trim().is_empty()) }
    
    fn link(&self) -> Option<String>
    {
        let plist_id: &str = self.id.as_ref()?;
        if !plist_id.trim().is_empty()
        { format!("https://youtube.com/playlist?list={plist_id}").into() }
        else
        { None }
    }
}

pub(crate) trait IntoSearchableItem
{
    fn into_item(self) -> SearchableItem;
}

impl IntoSearchableItem for Subscription
{
    fn into_item(self) -> SearchableItem
    {
        let mut item = SearchableItem::default();
        if let Some(snippet) = self.snippet
        {
            item.title = snippet.title.filter(|i| !i.trim().is_empty());
            item.description = snippet.description.filter(|i| !i.trim().is_empty());
            item.date = snippet.published_at.filter(|i| !i.trim().is_empty());
            item.link =
                snippet
                    .resource_id
                    .and_then(|r_id| r_id.channel_id)
                    .filter(|i| !i.trim().is_empty())
                    .map(|chan_id| format!("https://youtube.com/channel/{chan_id}"));
        }
        item
    }
}

impl IntoSearchableItem for Playlist
{
    fn into_item(self) -> SearchableItem
    {
        let mut item = SearchableItem::default();
        if let Some(snippet) = self.snippet
        {
            item.title = snippet.title.filter(|i| !i.trim().is_empty());
            item.description = snippet.description.filter(|i| !i.trim().is_empty());
            item.date = snippet.published_at.filter(|i| !i.trim().is_empty());
        }
        if let Some(plist_id) = self.id
        {
            if !plist_id.trim().is_empty()
            { item.link = format!("https://youtube.com/playlist?list={plist_id}").into(); }
        }
        item
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use google_youtube3::api::PlaylistListResponse;
    use super::*;
    
    #[test]
    fn plist_test() -> eyre::Result<()>
    {
        let f = std::fs::read_to_string("../../test_assets/playlist_list_json_response.json")?;
        let pl_resp =
            serde_json::from_str::<PlaylistListResponse>(&f).unwrap();
        let y = pl_resp.token_pagination;
        dbg!(&y);
        let items = pl_resp.items.unwrap();
        let first_playlist = items.get(0).unwrap().clone();
        assert!(matches!(first_playlist.description(), None));
        assert_eq!(first_playlist.title().unwrap(), "Истории");
        Ok(())
    }
}


