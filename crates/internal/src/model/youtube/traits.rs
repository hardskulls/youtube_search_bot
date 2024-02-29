use crate::model::utils::HTMLise;
use crate::model::youtube::types::SearchableItem;
use google_youtube3::api::{Playlist, PlaylistItem, Subscription};

/// Anything that can be searched on user's `YouTube` channel.
pub(crate) trait Searchable {
    fn title(&self) -> Option<&str>;

    fn description(&self) -> Option<&str>;

    fn date(&self) -> Option<&str>;

    fn link(&self) -> Option<String>;

    fn about(&self) -> Option<String>;
}

impl Searchable for Subscription {
    fn title(&self) -> Option<&str> {
        self.snippet
            .as_ref()?
            .title
            .as_deref()
            .filter(|i| !i.trim().is_empty())
    }

    fn description(&self) -> Option<&str> {
        self.snippet
            .as_ref()?
            .description
            .as_deref()
            .filter(|s| !s.trim().is_empty())
    }

    fn date(&self) -> Option<&str> {
        self.snippet
            .as_ref()?
            .published_at
            .as_deref()
            .filter(|s| !s.trim().is_empty())
    }

    fn link(&self) -> Option<String> {
        let chan_id = self
            .snippet
            .as_ref()?
            .resource_id
            .as_ref()?
            .channel_id
            .as_ref();
        chan_id
            .filter(|s| !s.trim().is_empty())
            .map(|chan_id| format!("https://youtube.com/channel/{chan_id}"))
    }

    fn about(&self) -> Option<String> {
        None
    }
}

impl Searchable for Playlist {
    fn title(&self) -> Option<&str> {
        self.snippet
            .as_ref()?
            .title
            .as_deref()
            .filter(|s| !s.trim().is_empty())
    }

    fn description(&self) -> Option<&str> {
        self.snippet
            .as_ref()?
            .description
            .as_deref()
            .filter(|s| !s.trim().is_empty())
    }

    fn date(&self) -> Option<&str> {
        self.snippet
            .as_ref()?
            .published_at
            .as_deref()
            .filter(|s| !s.trim().is_empty())
    }

    fn link(&self) -> Option<String> {
        let plist_id = self.id.as_ref();
        plist_id
            .filter(|s| !s.trim().is_empty())
            .map(|plist_id| format!("https://youtube.com/playlist?list={plist_id}"))
    }

    fn about(&self) -> Option<String> {
        None
    }
}

impl Searchable for PlaylistItem {
    fn title(&self) -> Option<&str> {
        self.snippet.as_ref()?.title.as_deref()
    }

    fn description(&self) -> Option<&str> {
        self.snippet.as_ref()?.description.as_deref()
    }

    fn date(&self) -> Option<&str> {
        self.snippet.as_ref()?.published_at.as_deref()
    }

    fn link(&self) -> Option<String> {
        let mut link = None;
        if let Some(ref snippet) = self.snippet {
            let video_id = snippet
                .resource_id
                .as_ref()
                .and_then(|r_id| r_id.video_id.clone());
            let index_in_pl = snippet.position;
            if let (Some(pl_item_id), Some(v), Some(idx)) =
                (self.id.as_ref(), video_id, index_in_pl)
            {
                let id = pl_item_id;
                link = format!("https://www.youtube.com/watch?v={v}&list={id}&index={idx}").into();
            }
        }
        link
    }

    fn about(&self) -> Option<String> {
        None
    }
}

pub(crate) trait IntoSearchableItem {
    fn into_item(self) -> SearchableItem;
}

impl IntoSearchableItem for Subscription {
    fn into_item(self) -> SearchableItem {
        let mut item = SearchableItem::default();
        if let Some(snippet) = self.snippet {
            item.title = snippet.title.filter(|s| !s.trim().is_empty());
            item.description = snippet.description.filter(|s| !s.trim().is_empty());
            item.date = snippet.published_at.filter(|s| !s.trim().is_empty());
            item.link = snippet
                .resource_id
                .and_then(|r_id| r_id.channel_id)
                .filter(|s| !s.trim().is_empty())
                .map(|chan_id| format!("https://youtube.com/channel/{chan_id}"));
        }
        item
    }
}

impl IntoSearchableItem for Playlist {
    fn into_item(self) -> SearchableItem {
        let mut item = SearchableItem::default();
        if let Some(snippet) = self.snippet {
            item.title = snippet.title.filter(|s| !s.trim().is_empty());
            item.description = snippet.description.filter(|s| !s.trim().is_empty());
            item.date = snippet.published_at.filter(|s| !s.trim().is_empty());
        }
        if let Some(plist_id) = self.id {
            if !plist_id.trim().is_empty() {
                item.link = format!("https://youtube.com/playlist?list={plist_id}").into();
            }
        }
        item
    }
}

impl IntoSearchableItem for PlaylistItem {
    fn into_item(self) -> SearchableItem {
        let mut item = SearchableItem::default();
        if let Some(snippet) = self.snippet {
            item.title = snippet.title.filter(|s| !s.trim().is_empty());
            item.description = snippet.description.filter(|s| !s.trim().is_empty());
            item.date = snippet.published_at.filter(|s| !s.trim().is_empty());
            let video_id = snippet.resource_id.and_then(|r_id| r_id.video_id);
            let index_in_pl = snippet.position;
            if let (Some(pl_item_id), Some(v), Some(idx)) = (self.id, video_id, index_in_pl) {
                let id = pl_item_id;
                item.link =
                    format!("https://www.youtube.com/watch?v={v}&list={id}&index={idx}").into();
            }
            let (playlist_id, video_owner_channel_title, video_owner_channel_id) = (
                snippet.playlist_id,
                snippet.video_owner_channel_title,
                snippet.video_owner_channel_id,
            );
            item.about = construct_about_for_pl_item(
                playlist_id,
                video_owner_channel_title,
                video_owner_channel_id,
            )
            .into();
        }
        item
    }
}

fn construct_about_for_pl_item(
    playlist_id: Option<String>,
    video_owner_channel_title: Option<String>,
    video_owner_channel_id: Option<String>,
) -> String {
    let form = |pl_id| format!("https://youtube.com/playlist?list={pl_id}");
    let pl_link = playlist_id.map(form);

    let video_owner_title = video_owner_channel_title;

    let form = |vid_id| format!("https://youtube.com/channel/{vid_id}");
    let video_owner_chan_link = video_owner_channel_id.map(form);

    let mut about = format!("{about}\n\n--------------------", about = "About".to_bold());

    if let Some(pl_link) = pl_link {
        about += &*format!("\n\n[Playlist]\n \n{pl_link}");
    }

    if video_owner_title.is_some() || video_owner_chan_link.is_some() {
        about += "\n\n[Published by]\n";
    }

    if let Some(vid_owner_title) = video_owner_title {
        about += &*format!("\n{vid_owner_title}");
    }

    if let Some(owner_chan_link) = video_owner_chan_link {
        about += &*format!("\n{owner_chan_link}");
    }

    about
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use google_youtube3::api::PlaylistListResponse;

    #[test]
    fn plist_test() -> eyre::Result<()> {
        let f = std::fs::read_to_string("../../test_assets/playlist_list_json_response.json")?;
        let pl_resp = serde_json::from_str::<PlaylistListResponse>(&f).unwrap();
        let y = pl_resp.token_pagination;
        dbg!(&y);
        let items = pl_resp.items.unwrap();
        let first_playlist = items.get(0).unwrap().clone();
        assert!(matches!(first_playlist.description(), None));
        assert_eq!(first_playlist.title().unwrap(), "Истории");
        Ok(())
    }
}
