use crate::mods::youtube::types::{AUTH_URL_BASE, RequiredAuthURLParams};
use crate::StdResult;

pub(crate) fn make_auth_url<V>(client_id: V, redirect_uri: V, response_type: V, scope: &[V], optional_params: &[(String, V)])
    -> StdResult<url::Url, url::ParseError>
    where
        V: AsRef<str> + Clone, // K: AsRef<str>, I: IntoIterator, I::Item: std::borrow::Borrow<(K, V)>,
{
    let keys = (RequiredAuthURLParams::ClientId, RequiredAuthURLParams::RedirectUri, RequiredAuthURLParams::ResponseType);
    let required_params = [(keys.0.to_string(), client_id), (keys.1.to_string(), redirect_uri), (keys.2.to_string(), response_type)];
    let params = [&required_params[..], optional_params].concat();
    let mut url: url::Url = url::Url::parse_with_params(AUTH_URL_BASE, &params)?;
    let (scope_key, scope_list) = (RequiredAuthURLParams::Scope.to_string(), join(scope, ","));
    url.query_pairs_mut().append_pair(&scope_key, &scope_list);
    Ok(url)
}

pub(crate) fn join<T>(pieces: &[T], separator: &str) -> String
    where T: AsRef<str>,
{
    let mut iter = pieces.iter();
    let first =
        match iter.next()
        {
            Some(p) => p,
            None => return String::new(),
        };
    let num_separators = pieces.len() - 1;
    let pieces_size: usize = pieces.iter().map(|p| p.as_ref().len()).sum();
    let size = pieces_size + separator.len() * num_separators;
    let mut result = String::with_capacity(size);
    result.push_str(first.as_ref());
    for p in iter
    {
        result.push_str(separator);
        result.push_str(p.as_ref());
    }
    debug_assert_eq!(size, result.len());
    result
}

pub(crate) fn query_pairs(uri: &axum::http::Uri) -> impl Iterator<Item = (&str, &str)>
{
    let s = uri.query().unwrap_or("");
    let res = s.split('&').filter_map(|st| st.split_once('='));
    res
}

#[cfg(test)]
mod tests
{
    use google_youtube3::oauth2::read_application_secret;

    use crate::mods::youtube::types::URL_1;

    use super::*;

    #[tokio::test]
    async fn make_auth_url_test() -> eyre::Result<()>
    {
        let secret =
            read_application_secret("C:/Users/Bender/Documents/Code/MyCode/Current/youtube_search_bot/crates/secret.json").await.unwrap();
        let (client_id, redirect_uri, response_type) = (secret.client_id, secret.redirect_uris[0].clone(), "code".into());
        let scopes = ["https://www.googleapis.com/auth/youtube".to_owned(), "https://www.googleapis.com/auth/youtube.readonly".to_owned()];
        let opt_params = [("access_type".to_owned(), "offline".to_owned())];
        let url = make_auth_url(client_id, redirect_uri, response_type, &scopes, &opt_params).unwrap();
        let uri_1 = url.as_str().parse::<axum::http::Uri>().unwrap();
        let uri_2 = URL_1.parse::<axum::http::Uri>().unwrap();
        let mut val_1: Vec<_> = query_pairs(&uri_1).collect();
        val_1.sort();
        let mut val_2: Vec<_> = query_pairs(&uri_2).collect();
        val_2.sort();
        for (i, x) in val_1.iter().enumerate()
        {
            assert_eq!(x, &val_2[i]);
        }
        Ok(())
    }
}


