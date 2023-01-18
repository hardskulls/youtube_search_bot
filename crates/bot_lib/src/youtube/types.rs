use parse_display::Display;
use serde::{Deserialize, Deserializer, Serialize};

/// Represents a `token` as returned by `OAuth2` servers.
///
/// It is produced by all authentication flows.
/// It authenticates certain operations, and must be refreshed once it reached it's expiry date.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct YouTubeAccessToken
{
    /// The token that your application sends to authorize a Google API request.
    pub access_token: String,
    /// Date and time when access_token expires (it does so after 1 hour).
    #[serde(deserialize_with = "expires_in_field_deserialize")]
    pub expires_in: time::OffsetDateTime,
    /// A token that you can use to obtain a new access token. Refresh tokens are valid until
    /// the user revokes access. Again, this field is only present in this response if you set
    /// the access_type parameter to offline in the initial request to Google's authorization server.
    pub refresh_token: Option<String>,
    /// The scopes of access granted by the access_token expressed as a list of
    /// space-delimited, case-sensitive strings.
    #[serde(deserialize_with = "scope_field_deserialize")]
    pub scope: Vec<String>,
    /// The type of token returned. At this time, this field's value is always set to Bearer.
    pub token_type: String,
}

/// One of two internal representations of `YouTubeAccessToken` `scope` field.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
#[serde(untagged)]
enum ScopeInternalRepr
{
    SpaceSeparatedScopes(String),
    VecOfScopes(Vec<String>),
}

/// Custom serde helper for `YouTubeAccessToken` `scope` field.
/// Serialized representation might be a string of space separated scopes or a vector of scopes.
fn scope_field_deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
{
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

/// One of two internal representations of `YouTubeAccessToken` `expires_in` field.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
#[serde(untagged)]
enum ExpiresInInternalRepr
{
    ExpiresAfterSeconds(i64),
    ExpiresAt(time::OffsetDateTime)
}

/// Custom serde helper for `YouTubeAccessToken` `expires_in` field.
/// Serialized representation might be a string of space separated scopes or vector of scopes.
fn expires_in_field_deserialize<'de, D>(deserializer: D) -> Result<time::OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>
{
    let seconds_or_date_time = ExpiresInInternalRepr::deserialize(deserializer)?;
    match seconds_or_date_time
    {
        ExpiresInInternalRepr::ExpiresAt(offset_date_time) => Ok(offset_date_time),
        ExpiresInInternalRepr::ExpiresAfterSeconds(seconds) =>
            {
                let t = time::Duration::seconds(seconds);
                let expires_in = time::OffsetDateTime::now_utc() + t;
                Ok(expires_in)
            }
    }
}

pub(crate) const AUTH_URL_BASE: &str = "https://accounts.google.com/o/oauth2/v2/auth?";

/// Required in token request to get a code to be exchanged for `access token`.
pub(crate) const RESPONSE_TYPE: &str = "code";

pub(crate) const SCOPE_YOUTUBE_READONLY: &str = "https://www.googleapis.com/auth/youtube.readonly";

/// Required in token request to get optional `refresh token` in addition to `access token`.
pub(crate) const ACCESS_TYPE: &str = "offline";

#[derive(Debug, Display)]
#[display(style = "snake_case")]
pub(crate) enum RequiredAuthURLParams
{ ClientId, RedirectUri, ResponseType, Scope }

#[cfg(test)]
mod tests
{
    use google_youtube3::api::{Subscription, SubscriptionListResponse};
    use time::Duration;
    use super::*;
    
    #[test]
    fn enum_deserialize_test()
    {
        let s = "3600";
        let _seconds_or_date_time = dbg!(serde_json::from_str::<ExpiresInInternalRepr>(s).unwrap());
        let s = "[2022,360,19,23,6,313629700,0,0,0]";
        let _seconds_or_date_time = dbg!(serde_json::from_str::<ExpiresInInternalRepr>(s).unwrap());
    }
    
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
        let path = env!("ACCESS_TOKEN_EXAMPLE");
        let contents = std::fs::read_to_string(path).unwrap();
        let deserialized_2 = serde_json::from_str::<YouTubeAccessToken>(&contents);
        assert!(matches!(deserialized_2, Ok(_)));
    }
    
    #[test]
    fn string_to_vec_deserialization_test()
    {
        let token =
            r#"
                {
                    "access_token":     "token87t877679",
                    "expires_in":       3600,
                    "refresh_token":    "hvliyhgl89y8",
                    "scope":            "jhgf kjhvliyf kvikuf.ugk/jhghk.con khfu",
                    "token_type":       "Bearer"
                }
            "#;
        let deserialized_token = serde_json::from_str::<YouTubeAccessToken>(token);
        assert!(matches!(deserialized_token, Ok(_)), "cause: {deserialized_token:?}");
        
        let deserialized_token = dbg!(deserialized_token.unwrap());
        assert!(!deserialized_token.scope.is_empty());
        assert!(deserialized_token.scope.contains(&"kvikuf.ugk/jhghk.con".to_owned()));
        assert!(deserialized_token.expires_in > time::OffsetDateTime::now_utc() + Duration::minutes(59));
        assert_eq!(deserialized_token.access_token, "token87t877679");
        assert!(matches!(deserialized_token.refresh_token, Some(_)));
        assert_eq!(deserialized_token.refresh_token.as_ref().unwrap(), "hvliyhgl89y8");
        let serialized_token = dbg!(serde_json::to_string(&deserialized_token).unwrap());
        let _deserialized_again_token = dbg!(serde_json::from_str::<YouTubeAccessToken>(&serialized_token).unwrap());
    }
    
    #[test]
    fn sub_list_resp_deserialize_test()
    {
        let path = env!("SUBS_LIST_RESP");
        let subs = std::fs::read_to_string(path).unwrap();
        let subs_list_resp = serde_json::from_str::<SubscriptionListResponse>(&subs).unwrap();
        dbg!(subs_list_resp.next_page_token);
        dbg!(subs_list_resp.page_info);
        dbg!(subs_list_resp.kind);
        dbg!(subs_list_resp.etag);
        let s: &Subscription = subs_list_resp.items.as_ref().unwrap().get(0).unwrap();
        dbg!(&s.snippet);
        dbg!(s.snippet.as_ref().unwrap().resource_id.as_ref());
    }
}


