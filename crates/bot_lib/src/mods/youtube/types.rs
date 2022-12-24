use parse_display::Display;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::StdResult;

/// Represents a `token` as returned by `OAuth2` servers.
///
/// It is produced by all authentication flows.
/// It authenticates certain operations, and must be refreshed once it reached it's expiry date.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct YouTubeAccessToken
{
    /// used when authorizing calls to `oauth2` enabled services.
    pub access_token: Option<String>,
    /// used to refresh an expired `access_token`.
    pub refresh_token: Option<String>,
    /// The time when the `token` expires.
    pub expires_at: Option<OffsetDateTime>,
    /// Optionally included by the `OAuth2` server and may contain information to verify the identity
    /// used to obtain the `access token`.
    /// Specifically `Google API:s` include this if the additional scopes `email` and/or `profile`
    /// are used. In that case the content is an `JWT token`.
    pub id_token: Option<String>,
}

pub(crate) trait MapErrToString<T>
{
    fn map_err_to_str(self) -> Result<T, String>;
}

impl<T, E> MapErrToString<T> for StdResult<T, E>
    where E: ToString
{
    fn map_err_to_str(self) -> StdResult<T, String>
    {
        self.map_err(|e| e.to_string())
    }
}

pub(crate) const AUTH_URL_BASE: &str = "https://accounts.google.com/o/oauth2/v2/auth?";

pub(crate) const RESPONSE_TYPE: &str = "code";


pub(crate) const SCOPE_YOUTUBE_READONLY : &str = "https://www.googleapis.com/auth/youtube.readonly";

pub(crate) const ACCESS_TYPE: &str = "offline";

#[derive(Debug, Display)]
#[display(style = "snake_case")]
pub(crate) enum RequiredAuthURLParams
{ ClientId, RedirectUri, ResponseType, Scope }
/*
#[derive(Debug, Display)]
#[display(style = "snake_case")]
pub(crate) enum OptionalAuthURLParams
{ AccessType, State, IncludeGrantedScopes, LoginHint, Prompt }
*/


