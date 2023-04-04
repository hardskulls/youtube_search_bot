
use std::fmt::Display;
use crate::StdResult;

/// If error is present, this trait logs it and returns back.
pub trait LogErr
{
    fn log_err(self, log_msg : &str) -> Self;
}

impl<T, E> LogErr for StdResult<T, E>
    where
        E : Display
{
    fn log_err(self, log_prefix : &str) -> Self
    {
        if let Err(e) = &self
        { log::error!("{log_prefix}{e}") }
        self
    }
}


