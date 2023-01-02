use std::fmt::{Debug, Display};
use std::future::Future;
use teloxide::{requests::Requester, Bot, types::ChatId};

use thiserror::Error;
use crate::StdResult;

#[derive(Error, Debug, Clone)]
pub enum NetworkError
{
    #[error("[ {:?} ] : ( Failed to send value to telegram servers. )", Self::SendError)]
    SendError,
    #[error("[ {:?} ] : ()", Self::UpdateListenerError)]
    UpdateListenerError,
    #[error("[ {:?} ] : ( Failed to set a webhook. )", Self::WebHookSetupError)]
    WebHookSetupError,
}

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Failed to parse value. )", Self)]
pub struct ParseError;

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Text is missing. )", Self)]
pub struct NoTextError;

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Callback data is missing. )", Self)]
pub struct NoCallbackDataError;

#[derive(Error, Debug, Clone)]
#[error("[ MissingEnvVarError ] : ( Couldn't find environment variable \"{}\". )", var)]
pub struct MissingEnvVarError<'a> { pub var: &'a str }

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Problem with storage that stores dialogue state. )", Self)]
pub struct DialogueStateStorageError;

#[derive(Error, Debug, Clone)]
pub enum EndpointErrors
{
    #[error("[ {:?} ] : ( Something wrong with commands )", Self::CommandError)]
    CommandError,
    #[error("[ {:?} ] : ( Something wrong with game state )", Self::GameError)]
    GameError,
}

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Something wrong with setting up the project )", Self)]
pub struct ProjectSetupError;

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Serialization failed )", Self)]
pub struct SerializationError;

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Deserialization failed )", Self)]
pub struct DeserializationError;

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( No `MessageWithKB`, or its inner `Option<Message>` is `None`. )", Self)]
pub struct NoMessageWithKB;


pub async fn notify_user_on_err<'a, F, X, OK, S, FUT>(f: F, x: &'a X, bot: &Bot, send_to: ChatId, text: S)
    -> eyre::Result<OK>
    where
        FUT: Future<Output = eyre::Result<OK>>,
        F: Fn(&'a X) -> FUT,
        S: Into<String> + Send,
{
    match f(x).await
    {
        Ok(ok) => Ok(ok),
        Err(err) =>
            {
                let _ = bot.send_message(send_to, text).await;
                Err(err)
            }
    }
}

/// When `Ok()` and `Err` variants of `Result` are the same
/// type, it returns this type whether it's an error, or not.
pub trait MergeOkErr<T>
{
    fn merge_ok_err(self) -> T;
}

impl<T> MergeOkErr<T> for StdResult<T, T>
{
    fn merge_ok_err(self) -> T
    {
        match self
        {
            Ok(ok) => ok,
            Err(err) => err
        }
    }
}

/// This trait provides little helper method that replaces error completely ignoring it.
/// Required in order to get rid of ugly `.map_err(|_| bar)` calls.
pub trait MapErrBy<T, N>
{
    fn map_err_by(self, f: fn() -> N) -> StdResult<T, N>;
}

impl<T, E, N> MapErrBy<T, N> for StdResult<T, E>
{
    fn map_err_by(self, f: fn() -> N) -> StdResult<T, N>
    {
        self.map_err(|_| f())
    }
}

/// If error is present, this trait logs it and returns back.
pub trait LogErr
{
    fn log_err(self, log_msg: &str) -> Self;
}

impl<T, E> LogErr for StdResult<T, E>
    where
        E: Display
{
    fn log_err(self, log_msg: &str) -> Self
    {
        match self
        {
            Ok(ok) => Ok(ok),
            Err(e) =>
                {
                    log::error!("{log_msg}{e}");
                    Err(e)
                }
        }
    }
}

#[cfg(test)]
mod tests
{
    use miette::Diagnostic;
    use super::*;

    #[derive(Error, Debug, Clone, Diagnostic)]
    #[error("[ {:?} ] : ( Failed to parse value. )", Self)]
    struct TestError;

    #[test]
    fn miette_lib_test() -> miette::Result<()>
    {
        let maybe_number = "not a number".parse::<u8>().ok();
        let maybe_number = maybe_number.ok_or(TestError)?;
        assert_eq!(maybe_number, 8);
        Ok(())
    }
}


