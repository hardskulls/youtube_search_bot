use crate::mods::errors::ParseError;
use crate::mods::youtube::types::{AUTH_URL_BASE, RequiredAuthURLParams};
use crate::StdResult;

pub(crate) fn make_auth_url<V>(client_id: V, redirect_uri: V, response_type: V, scope: &[V], optional_params: &[(String, V)])
    -> StdResult<url::Url, url::ParseError>
    where
        V: AsRef<str> + Clone, /* K: AsRef<str>, I: IntoIterator, I::Item: std::borrow::Borrow<(K, V)> */
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

pub fn query_pairs<'a, 'b>(url_query: &'a str, sep: &'b str)
    -> StdResult<impl Iterator<Item = (&'a str, &'a str)> + 'b, ParseError>
    where
         'a: 'b
{
    let res = url_query.split(sep).filter_map(|kv_pair| kv_pair.split_once('='));
    if res.clone().count() < 1 { return Err(ParseError) }
    Ok(res)
}

pub fn find_by_key<'a, 'b>(url_query: &'a str, sep: &'b str, key: &str) -> StdResult<&'a str, ParseError>
{
    query_pairs(url_query, sep)?
        .find(|&(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or(ParseError)
}

#[cfg(test)]
mod tests
{
    use google_youtube3::oauth2::read_application_secret;
    const URL_1 : &str =
        "\
        https://accounts.google.com/o/oauth2/auth?\
        scope=https://www.googleapis.com/auth/youtube%20https://www.googleapis.com/auth/youtube.readonly&\
        access_type=offline&\
        redirect_uri=http://127.0.0.1:62320&\
        response_type=code&\
        client_id=799749940076-oktc5l1861j0ilnp3jndb9elrk38krus.apps.googleusercontent.com\
        ";
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
        let mut val_1: Vec<_> = query_pairs(uri_1.query().unwrap_or(""), "&")?.collect();
        val_1.sort();
        let mut val_2: Vec<_> = query_pairs(uri_2.query().unwrap_or(""), "&")?.collect();
        val_2.sort();
        for (i, x) in val_1.iter().enumerate()
        {
            assert_eq!(x, &val_2[i]);
        }
        Ok(())
    }

    // TODO: Finish or remove test.
    #[tokio::test]
    async fn go_rust_auth_urls() -> eyre::Result<()>
    {
        let _go_url =
            "\
            https://accounts.google.com/o/oauth2/auth?\
            access_type=offline&\
            client_id=799749940076-oktc5l1861j0ilnp3jndb9elrk38krus.apps.googleusercontent.com&\
            redirect_uri=https%3A%2F%2Fpost-123456.herokuapp.com%2Fgoogle_callback&\
            response_type=code&\
            scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fyoutube.readonly&\
            state=%26state_code%3Dkut987987_576fg78d5687lojfvkzr_85y6_435sgred_vnhgx_gdut%26for_user%3DАлександр+Лебедев\
            ";
        let _rust_url =
            "\
            https://accounts.google.com/o/oauth2/v2/auth?\
            client_id=156187461731-lgrn7aba80qtljt5pvqncm60me7b8rgl.apps.googleusercontent.com&\
            redirect_uri=https%3A%2F%2Fyoutube-search-bot.onrender.com&\
            response_type=code&\
            access_type=offline&\
            scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fyoutube%2Chttps%3A%2F%2Fwww.googleapis.com%2Fauth%2Fyoutube.readonly\
            ";
        Ok(())
    }
}

