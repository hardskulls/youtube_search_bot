
pub(crate) type StdResult<T, E> = Result<T, E>;

pub(crate) type FlatRes<T> = StdResult<T, T>;

pub mod errors;
pub mod commands;
pub mod dialogue;
mod youtube;
mod keyboards
{
    pub(crate) mod funcs;
    pub(crate) mod types;
    pub(crate) mod traits;
}
pub mod auth_server;
mod db;
mod net;
mod utils;


