use parse_display::Display;
use serde::{Deserialize, Deserializer, Serialize};
use crate::StdResult;

/// Represents a `token` as returned by `OAuth2` servers.
///
/// It is produced by all authentication flows.
/// It authenticates certain operations, and must be refreshed once it reached it's expiry date.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct YouTubeAccessToken
{
    /// The token that your application sends to authorize a Google API request.
    pub access_token: String,
    /// Time when access_token expires (it does so after 1 hour).
    #[serde(deserialize_with = "time_deserialize")]
    pub expires_in: time::OffsetDateTime,
    /// A token that you can use to obtain a new access token. Refresh tokens are valid until
    /// the user revokes access. Again, this field is only present in this response if you set
    /// the access_type parameter to offline in the initial request to Google's authorization server.
    pub refresh_token: Option<String>,
    /// The scopes of access granted by the access_token expressed as a list of
    /// space-delimited, case-sensitive strings.
    #[serde(deserialize_with = "string_of_strings_deserialize")]
    pub scope: Vec<String>,
    /// The type of token returned. At this time, this field's value is always set to Bearer.
    pub token_type: String,
}

fn string_of_strings_deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    let vec_of_str =
        str_sequence
            .split(' ')
            .map(|item| item.to_owned())
            .collect();
    Ok(vec_of_str)
}

fn time_deserialize<'de, D>(deserializer: D) -> Result<time::OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
{
    let expires_after_seconds = Deserialize::deserialize(deserializer)?;
    let t = time::Duration::seconds(expires_after_seconds);
    let expires_in = time::OffsetDateTime::now_utc() + t;
    Ok(expires_in)
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


pub(crate) const SCOPE_YOUTUBE_READONLY: &str = "https://www.googleapis.com/auth/youtube.readonly";

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
        assert!(matches!(deserialized_token, Ok(_)), "cause: {:?}", deserialized_token);
        
        let deserialized_token = deserialized_token.unwrap();
        assert!(!deserialized_token.scope.is_empty());
        assert!(deserialized_token.scope.contains(&"kvikuf.ugk/jhghk.con".to_owned()));
        assert!(deserialized_token.expires_in > time::OffsetDateTime::now_utc() + Duration::minutes(59));
        assert_eq!(deserialized_token.access_token, "token87t877679");
        assert!(matches!(deserialized_token.refresh_token, Some(_)));
        assert_eq!(deserialized_token.refresh_token.unwrap(), "hvliyhgl89y8");
    }
    
    #[test]
    fn sub_list_resp_deserialize_test()
    {
        let path = std::env::var("SUBS_LIST_RESP").unwrap();
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


