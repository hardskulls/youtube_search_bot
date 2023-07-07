
use parse_display::Display;
use serde::{Deserialize, Deserializer, Serialize};
use crate::model::youtube::traits::Searchable;


/// Represents a `token` as returned by `OAuth2` servers.
///
/// It is produced by all authentication flows.
/// It authenticates certain operations, and must be refreshed once it reached it's expiry date.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub(crate) struct YouTubeAccessToken
{
    /// The token that your application sends to authorize a Google API request.
    pub(crate) access_token: String,
    /// Date and time when access_token expires (it does so after 1 hour).
    #[serde(deserialize_with = "expires_in_field_deserialize")]
    pub(crate) expires_in: time::OffsetDateTime,
    /// A token that you can use to obtain a new access token. Refresh tokens are valid until
    /// the user revokes access. Again, this field is only present in this response if you set
    /// the access_type parameter to offline in the initial request to Google's authorization server.
    pub(crate) refresh_token: Option<String>,
    /// The scopes of access granted by the access_token expressed as a list of
    /// space-delimited, case-sensitive strings.
    #[serde(deserialize_with = "scope_field_deserialize")]
    pub(crate) scope: Vec<String>,
    /// The type of token returned. At this time, this field's value is always set to Bearer.
    pub(crate) token_type: String,
}

/// Custom serde helper for `YouTubeAccessToken` `scope` field.
/// Serialized representation might be a) string of space separated scopes or b) vector of scopes.
fn scope_field_deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
{
    /// One of two internal representations of `YouTubeAccessToken` `scope` field.
    #[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
    #[serde(untagged)]
    enum ScopeInternalRepr
    {
        SpaceSeparatedScopes(String),
        VecOfScopes(Vec<String>),
    }
    
    match ScopeInternalRepr::deserialize(deserializer)?
    {
        ScopeInternalRepr::VecOfScopes(vec_of_scopes) => Ok(vec_of_scopes),
        ScopeInternalRepr::SpaceSeparatedScopes(s) =>
            {
                let vec_of_str = s.split(' ').map(|item| item.to_owned()).collect();
                Ok(vec_of_str)
            }
    }
}

/// Custom serde helper for `YouTubeAccessToken` `expires_in` field.
/// Serialized representation might be a string of space separated scopes or vector of scopes.
fn expires_in_field_deserialize<'de, D>(deserializer: D) -> Result<time::OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>
{
    /// One of two internal representations of `YouTubeAccessToken` `expires_in` field.
    #[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
    #[serde(untagged)]
    enum ExpiresInInternalRepr
    {
        ExpiresAfterSeconds(i64),
        ExpiresAt(time::OffsetDateTime)
    }
    
    let seconds_or_date_time = ExpiresInInternalRepr::deserialize(deserializer)?;
    match seconds_or_date_time
    {
        ExpiresInInternalRepr::ExpiresAt(offset_date_time) => Ok(offset_date_time),
        ExpiresInInternalRepr::ExpiresAfterSeconds(seconds) =>
            {
                let expires_in = time::OffsetDateTime::now_utc() + time::Duration::seconds(seconds);
                Ok(expires_in)
            }
    }
}

/// Google OAuth2 url.
pub(crate) const AUTH_URL_BASE: &str = "https://accounts.google.com/o/oauth2/v2/auth?";

/// Required in token request to get exchange code.
/// The code will be exchanged for an `access token`).
pub(crate) const RESPONSE_TYPE: &str = "code";

pub(crate) const SCOPE_YOUTUBE_READONLY: &str = "https://www.googleapis.com/auth/youtube.readonly";

/// Required in token request to get optional `refresh token` in addition to `access token`.
pub(crate) const ACCESS_TYPE: &str = "offline";

#[derive(Debug, Display)]
#[display(style = "snake_case")]
pub(crate) enum RequiredAuthURLParams
{ ClientId, RedirectUri, ResponseType, Scope }

#[derive(Default, Debug, Clone)]
pub(crate) struct SearchableItem
{
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) link: Option<String>
}

impl Searchable for SearchableItem
{
    fn title(&self) -> Option<&str> 
    { self.title.as_deref()?.into() }
    
    fn description(&self) -> Option<&str>
    { self.description.as_deref()?.into() }
    
    fn date(&self) -> Option<&str>
    { self.date.as_deref()?.into() }
    
    fn link(&self) -> Option<String>
    { self.link.clone() }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use google_youtube3::api::{Subscription, SubscriptionListResponse};
    use time::Duration;
    use super::*;
    
    
    #[test]
    fn serialize_deserialize_string_test()
    {
        let (access_token, refresh_token) =
            ("access_token".to_owned(), Some("refresh_token".to_owned()));
        let (scope, token_type) =
            (vec!["hey".to_owned()], "id_token".to_owned());
        let expires_in = time::OffsetDateTime::now_utc();
        let token = YouTubeAccessToken { access_token, expires_in, refresh_token, scope, token_type };
        let serialized = serde_json::to_string(&token).unwrap();
        dbg!(&serialized);
        let deserialized = serde_json::from_str::<YouTubeAccessToken>(&serialized).unwrap();
        assert_eq!(token, deserialized);
    }
    
    #[test]
    fn deserialize_from_json_test()
    {
        let token =
            r#"
                {
                    "access_token": "1/fFAGRNJru1FTz70BzhT3Zg",
                    "expires_in": 3920,
                    "token_type": "Bearer",
                    "scope": "https://www.googleapis.com/auth/drive.metadata.readonly",
                    "refresh_token": "1//xEoDL4iW3cxlI7yDbSRFYNG01kVKM2C-259HOF2aQbI"
                }
            "#;
        let deserialized_token = serde_json::from_str::<YouTubeAccessToken>(token);
        
        assert!(matches!(deserialized_token, Ok(_)), "cause: {deserialized_token:?}");
        
        let deserialized_token = dbg!(deserialized_token.unwrap());
        
        assert!(!deserialized_token.scope.is_empty());
        assert!(deserialized_token.scope.contains(&"https://www.googleapis.com/auth/drive.metadata.readonly".to_owned()));
        assert!(deserialized_token.expires_in > time::OffsetDateTime::now_utc() + Duration::minutes(59));
        assert_eq!(deserialized_token.access_token, "1/fFAGRNJru1FTz70BzhT3Zg");
        assert!(matches!(deserialized_token.refresh_token, Some(_)));
        assert_eq!(deserialized_token.refresh_token.as_ref().unwrap(), "1//xEoDL4iW3cxlI7yDbSRFYNG01kVKM2C-259HOF2aQbI");
        
        let serialized_token = dbg!(serde_json::to_string(&deserialized_token).unwrap());
        let _deserialized_again_token = dbg!(serde_json::from_str::<YouTubeAccessToken>(&serialized_token).unwrap());
    }
    
    #[test]
    fn subscription_list_resp_deserialize_test()
    {
        let subs = std::fs::read_to_string("../../test_assets/subscription_list_json_response.json").unwrap();
        let subs_list_resp = serde_json::from_str::<SubscriptionListResponse>(&subs).unwrap();
        
        assert!(matches!(subs_list_resp.next_page_token, Some(..)));
        assert!(matches!(subs_list_resp.page_info, Some(..)));
        assert!(matches!(subs_list_resp.kind, Some(..)));
        assert!(matches!(subs_list_resp.etag, Some(..)));
        
        let s: &Subscription = subs_list_resp.items.as_ref().unwrap().get(0).unwrap();
        
        assert!(matches!(s.snippet, Some(..)));
        assert!(matches!(s.snippet.as_ref().unwrap().resource_id.as_ref(), Some(..)));
    }
}


