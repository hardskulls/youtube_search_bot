
use std::borrow::Borrow;
use error_traits::WrapInOk;
use reqwest::RequestBuilder;
use crate::model::errors::ParseError;
use crate::StdResult;


/// Builds a post request with key-value pairs placed in body.
pub(crate) fn build_post_request<P, K, V>(url : &str, body_kv_pairs : P) -> eyre::Result<RequestBuilder>
    where
        P : IntoIterator,
        P::Item : Borrow<(K, V)>,
        K : AsRef<str>,
        V : AsRef<str>
{
    let url = reqwest::Url::parse(url)?;
    let host = url.host_str().ok_or(eyre::eyre!("No host in url string"))?.to_string();
    let body_kv_pairs = url::form_urlencoded::Serializer::new(String::new()).extend_pairs(body_kv_pairs).finish();
    reqwest::Client::new()
        .post(url)
        .header(reqwest::header::HOST, host)
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body_kv_pairs)
        .in_ok()
}

pub(crate) fn join<T>(pieces : &[T], separator : &str) -> String
    where T : AsRef<str>,
{
    let mut iter = pieces.iter();
    let first =
        match iter.next()
        {
            Some(p) => p,
            None => return String::new(),
        };
    let num_separators = pieces.len() - 1;
    let pieces_size : usize = pieces.iter().map(|p| p.as_ref().len()).sum();
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

pub(crate) fn query_pairs<'a, 'b>(url_query : &'a str, sep : &'b str)
    -> StdResult<impl Iterator<Item = (&'a str, &'a str)> + 'b, ParseError>
    where
        'a : 'b
{
    let res = url_query.split(sep).filter_map(|kv_pair| kv_pair.split_once('='));
    if res.clone().count() < 1 { return Err(ParseError) }
    res.in_ok()
}

/// Returns a certain `value` in a query key-value pairs.
pub(crate) fn find_by_key<'a>(url_query : &'a str, sep : &str, key : &str) -> StdResult<&'a str, ParseError>
{
    query_pairs(url_query, sep)?
        .find(|&(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or(ParseError)
}


#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use std::str::{from_utf8, FromStr};
    use google_youtube3::hyper;
    use crate::model::net::funcs::build_post_request;
    use crate::model::net::types::REVOKE_ACCESS_TOKEN_URL;
    
    
    trait ShortUnwrap<T>
    {
        fn unwr(self) -> T;
    }
    
    impl<T> ShortUnwrap<T> for Option<T>
    {
        fn unwr(self) -> T
        { self.unwrap() }
    }
    
    impl<T, E> ShortUnwrap<T> for Result<T, E>
        where
            E : std::fmt::Debug
    {
        fn unwr(self) -> T
        { self.unwrap() }
    }
    
    
    #[test]
    fn test_request_builder()
    {
        let params : &[(&str, &str)] = &[("token", "HeyHo"), ("answer", "YoHoHo")];
        let expected_query = "token=HeyHo&answer=YoHoHo";
        let body_kv_pairs = url::form_urlencoded::Serializer::new(String::new()).extend_pairs(params).finish();
        
        assert_eq!(body_kv_pairs, expected_query);
        
        let url = reqwest::Url::parse_with_params(REVOKE_ACCESS_TOKEN_URL, params).unwr();
        
        assert_eq!(url.domain().unwr(), "oauth2.googleapis.com");
        assert_eq!(url.host_str().unwr(), "oauth2.googleapis.com");
        
        let req_builder = build_post_request(REVOKE_ACCESS_TOKEN_URL, params).unwr();
        let req = req_builder.build().unwr();
        let body_as_utf8 = from_utf8(req.body().unwr().as_bytes().unwr()).unwr();
        
        assert_eq!(body_as_utf8, expected_query);
        
        
        let url = REVOKE_ACCESS_TOKEN_URL;
        let u = hyper::http::uri::Uri::from_str(url).unwr();
        
        assert_eq!(u.authority().unwr().as_str(), "oauth2.googleapis.com");
        assert_eq!(u.scheme_str().unwr(), "https");
        assert_eq!(u.to_string(), url);
    }
}


