use parse_display::Display;

pub(crate) trait MapErrToString<T>
{
    fn map_err_to_str(self) -> Result<T, String>;
}

impl<T, E> MapErrToString<T> for std::result::Result<T, E>
    where E: ToString
{
    fn map_err_to_str(self) -> Result<T, String>
    {
        self.map_err(|e| e.to_string())
    }
}

pub(crate) const AUTH_URL_BASE : &str = "https://accounts.google.com/o/oauth2/v2/auth?";

pub(crate) const URL_1 : &str =
    "\
        https://accounts.google.com/o/oauth2/auth?\
        scope=https://www.googleapis.com/auth/youtube%20https://www.googleapis.com/auth/youtube.readonly&\
        access_type=offline&\
        redirect_uri=http://127.0.0.1:62320&\
        response_type=code&\
        client_id=799749940076-oktc5l1861j0ilnp3jndb9elrk38krus.apps.googleusercontent.com\
    ";

pub(crate) const CLIENT_ID: &str = "799749940076-oktc5l1861j0ilnp3jndb9elrk38krus.apps.googleusercontent.com";

pub(crate) const REDIRECT_URI: &str = "code";

pub(crate) const RESPONSE_TYPE: &str = "code";

pub(crate) const SCOPE_YOUTUBE : &str = "https://www.googleapis.com/auth/youtube";

pub(crate) const SCOPE_YOUTUBE_READONLY : &str = "https://www.googleapis.com/auth/youtube.readonly";

pub(crate) const ACCESS_TYPE : &str = "offline";

pub(crate) struct TelegramBotInstalledFlow;

#[derive(Debug, Display)]
#[display(style = "snake_case")]
pub(crate) enum RequiredAuthURLParams
{ ClientId, RedirectUri, ResponseType, Scope }

#[derive(Debug, Display)]
#[display(style = "snake_case")]
pub(crate) enum OptionalAuthURLParams
{ AccessType, State, IncludeGrantedScopes, LoginHint, Prompt }

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn url_equal()
    {
        // assert_eq!(URL_1, URL_2);
    }
}


