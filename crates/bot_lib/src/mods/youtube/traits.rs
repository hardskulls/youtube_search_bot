use google_youtube3::api::{Playlist, Subscription};


pub trait Searcheable
{
    fn title(&self) -> Option<&str>;
    
    fn description(&self) -> Option<&str>;
    
    fn link(&self) -> Option<String>;
}

impl Searcheable for Subscription
{
    fn title(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(title) = snip.title.as_ref()
            { return title.as_str().into() }
        }
        None
    }
    
    fn description(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(description) = snip.description.as_ref()
            { return description.as_str().into() }
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

impl Searcheable for Playlist
{
    fn title(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(title) = snip.title.as_ref()
            { return title.as_str().into() }
        }
        None
    }
    
    fn description(&self) -> Option<&str>
    {
        if let Some(snip) = self.snippet.as_ref()
        {
            if let Some(description) = snip.description.as_ref()
            { return description.as_str().into() }
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


