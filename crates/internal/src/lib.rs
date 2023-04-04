#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

pub(crate) type StdResult<T, E> = Result<T, E>;

pub(crate) type FlatRes<T> = StdResult<T, T>;

pub mod auth_server
{ pub use crate::model::auth_server::{handle_auth_code, serve_all}; }

pub mod commands
{ pub use crate::model::commands::types::Command; }

pub mod handlers
{
    pub use crate::controllers::text::handle_text;
    pub use crate::controllers::commands::{handle_commands, is_other_command, handle_unknown_command};
    pub use crate::controllers::callback::handle_callback;
}

pub mod errors
{ pub use crate::model::errors::*; }

pub mod dialogue
{ pub use crate::model::dialogue::types::DialogueData; }

pub(crate) mod model;
pub(crate) mod view
{
    pub(crate) mod types;
    pub(crate) mod funcs;
}
pub(crate) mod controllers
{
    pub(crate) mod text;
    pub(crate) mod commands;
    pub(crate) mod callback;
}


