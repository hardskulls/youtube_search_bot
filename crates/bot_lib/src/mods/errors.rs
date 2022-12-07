use std::fmt::Debug;
use std::future::Future;
use teloxide::{requests::Requester, Bot, types::ChatId};

use thiserror::Error;

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
pub struct SerializeError;

#[derive(Error, Debug, Clone)]
#[error("[ {:?} ] : ( Deserialization failed )", Self)]
pub struct DeserializeError;


pub async fn notify_user_on_err<'a, F, X, OK, ERR, S, FUT>(f: F, x: &'a X, bot: &Bot, send_to: ChatId, text: S)
    -> error_stack::Result<OK, ERR>
    where
        FUT: Future<Output = error_stack::Result<OK, ERR>>,
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



