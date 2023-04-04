use error_traits::WrapInOk;
use crate::model::errors::ParseError;
use crate::StdResult;

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

