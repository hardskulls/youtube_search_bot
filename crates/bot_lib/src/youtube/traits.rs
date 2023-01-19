use google_youtube3::api::{Playlist, Subscription};


pub trait Searchable
{
    fn title(&self) -> Option<&str>;
    
    fn description(&self) -> Option<&str>;
    
    fn date(&self) -> Option<&str>;
    
    fn link(&self) -> Option<String>;
}

impl Searchable for Subscription
{
    fn title(&self) -> Option<&str>
    {
        let title: &str = self.snippet.as_ref()?.title.as_ref()?;
        if title.is_empty()
        { None }
        else
        { title.into() }
    }
    
    fn description(&self) -> Option<&str>
    {
        let description: &str = self.snippet.as_ref()?.description.as_ref()?;
        if description.is_empty()
        { None }
        else
        { description.into() }
    }
    
    fn date(&self) -> Option<&str>
    {
        self.snippet.as_ref()?.published_at.as_ref()?.as_str().into()
    }
    
    fn link(&self) -> Option<String>
    {
        let chan_id: &str = self.snippet.as_ref()?.resource_id.as_ref()?.channel_id.as_ref()?;
        format!("https://youtube.com/channel/{chan_id}").into()
    }
}

impl Searchable for Playlist
{
    fn title(&self) -> Option<&str>
    {
        let title: &str = self.snippet.as_ref()?.title.as_ref()?;
        if title.is_empty()
        { None }
        else
        { title.into() }
    }
    
    fn description(&self) -> Option<&str>
    {
        let description: &str = self.snippet.as_ref()?.description.as_ref()?;
        if description.is_empty()
        { None }
        else
        { description.into() }
    }
    
    fn date(&self) -> Option<&str>
    {
        self.snippet.as_ref()?.published_at.as_ref()?.as_str().into()
    }
    
    fn link(&self) -> Option<String>
    {
        let id: &str = self.id.as_ref()?;
        format!("https://youtube.com/playlist?list={id}").into()
    }
}

#[cfg(test)]
mod tests
{
    use google_youtube3::api::PlaylistListResponse;
    use super::*;
    
    #[test]
    fn plist_test() -> eyre::Result<()>
    {
        let path = env!("PATH_TO_PLAYLIST_JSON_EXAMPLE");
        let f = std::fs::read_to_string(path)?;
        let pl_resp =
            serde_json::from_str::<PlaylistListResponse>(&f).unwrap();
        let y = pl_resp.token_pagination;
        dbg!(&y);
        let items = pl_resp.items.unwrap();
        let first_playlist = items.get(0).unwrap().clone();
        assert!(matches!(first_playlist.description(), None));
        assert_eq!(first_playlist.title().unwrap(), "Посмотреть позже 21");
        Ok(())
    }
}


