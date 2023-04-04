
pub(crate) mod db;
pub(crate) mod errors;
pub(crate) mod auth_server;
pub(crate) mod utils;
pub(crate) mod dialogue
{
    pub(crate) mod types;
    pub(crate) mod funcs;
}
pub(crate) mod youtube
{
    pub(crate) mod types;
    pub(crate) mod funcs;
    pub(crate) mod traits;
}
pub(crate) mod net
{
    pub(crate) mod funcs;
    pub(crate) mod traits;
}
pub(crate) mod commands
{
    pub(crate) mod types;
    pub(crate) mod funcs;
}
pub(crate) mod keyboards
{
    pub(crate) mod types;
    pub(crate) mod funcs;
    pub(crate) mod traits;
}
pub(crate) mod handlers
{
    pub(crate) mod text;
    pub(crate) mod commands;
    pub(crate) mod callback;
}


