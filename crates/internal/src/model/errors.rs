use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error(
        "[ {:?} ] : ( Failed to send value to telegram servers. )",
        Self::SendError
    )]
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
#[error(
    "[ MissingEnvVarError ] : ( Couldn't find environment variable \"{}\". )",
    var
)]
pub struct MissingEnvVarError<'a> {
    pub var: &'a str,
}

#[derive(Error, Debug, Clone)]
#[error(
    "[ {:?} ] : ( Problem with storage that stores dialogue state. )",
    Self
)]
pub struct DialogueStateStorageError;

#[derive(Error, Debug, Clone)]
pub enum EndpointErrors {
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
#[error(
    "[ {:?} ] : ( No `MessageWithKB`, or its inner `Option<Message>` is `None`. )",
    Self
)]
pub struct NoMessageWithKB;

#[derive(Error, Debug, Clone)]
#[error(
    "[ {:?} ] : ( No `MessageWithKB`, or its inner `Option<Message>` is `None`. )",
    Self
)]
pub struct MissingType;
