use google_youtube3::api::{Playlist, Subscription};


pub trait Searchable
{
    fn title(&self) -> Option<&str>;
    
    fn description(&self) -> Option<&str>;
    
    fn link(&self) -> Option<String>;
}

impl Searchable for Subscription
{
    fn title(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(title) = snip.title.as_ref()
            {
                if !title.is_empty()
                { return title.as_str().into(); }
            }
        }
        None
    }
    
    fn description(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(description) = snip.description.as_ref()
            {
                if !description.is_empty()
                { return description.as_str().into(); }
            }
        }
        None
    }
    
    fn link(&self) -> Option<String>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(resource_id) = snip.resource_id.as_ref()
            {
                if let Some(chan_id) = resource_id.channel_id.as_ref()
                { return Some(format!("https://youtube.com/channel/{chan_id}")) }
            }
        }
        None
    }
}

impl Searchable for Playlist
{
    fn title(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(title) = snip.title.as_ref()
            {
                if !title.is_empty()
                { return title.as_str().into(); }
            }
        }
        None
    }
    
    fn description(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(description) = snip.description.as_ref()
            {
                if !description.is_empty()
                { return description.as_str().into(); }
            }
        }
        None
    }
    
    fn link(&self) -> Option<String>
    {
        if let Some(id) = self.id.as_ref()
        { return format!("https://youtube.com/playlist?list={id}").into() }
        None
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
        let path = std::env::var("PLIST_JSON_RESP").unwrap();
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


