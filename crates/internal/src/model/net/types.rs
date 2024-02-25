use serde::{Deserialize, Serialize};

/// Custom query separator to encode multiple key-value pairs.into one url query value (key=value&key=value).
pub const QUERY_SEPARATOR: &str = "xplusx";

/// State code to check in incoming oauth2 response.
pub const STATE_CODE: &str = env!("STATE_CODE");

/// Url for acquiring `access token` via
/// - exchanging `auth code` (first time)
///
/// or
/// - refreshing `access token` using `refresh token`.
pub const GET_ACCESS_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/// Url for revoking `access token.
pub const REVOKE_ACCESS_TOKEN_URL: &str = "https://oauth2.googleapis.com/revoke";

pub const YOUTUBE_SUBSCRIPTIONS_API: &str = "https://www.googleapis.com/youtube/v3/subscriptions";

pub const YOUTUBE_PLAYLISTS_API: &str = "https://youtube.googleapis.com/youtube/v3/playlists";

pub const YOUTUBE_PLAYLIST_ITEMS_API: &str =
    "https://youtube.googleapis.com/youtube/v3/playlistItems";

// TODO : Choose a better naming and description.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionRequester;

// TODO : Choose a better naming and description.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaylistRequester;

// TODO : Choose a better naming and description.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaylistItemRequester<'a> {
    pub playlist_id: &'a str,
}
