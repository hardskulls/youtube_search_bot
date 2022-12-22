
pub(crate) type StdResult<T, E> = Result<T, E>;

mod mods
{
    pub(crate) mod errors;
    pub(crate) mod commands;
    pub(crate) mod dialogue;
    pub(crate) mod youtube;
    pub(crate) mod inline_keyboards 
    {
        pub(crate) mod funcs; 
        pub(crate) mod types;
        pub(crate) mod traits;
    }
    pub(crate) mod net;
}

pub mod commands
{
    pub mod types { pub use crate::mods::commands::Command; }
    pub mod funcs { pub use crate::mods::commands::{handle_unknown_command, handle_commands, is_other_command}; }
}

pub mod dialogue
{
    pub mod funcs { pub use crate::mods::dialogue::{handle_callback_data, handle_text}; }
    pub mod types { pub use crate::mods::dialogue::types::{TheDialogue, DialogueData, State, ListConfigData, SearchConfigData}; }
}

pub mod errors
{
    pub mod types { pub use crate::mods::errors::*; }
}

pub mod net
{
    pub mod url
    {
        pub use crate::mods::youtube::funcs::{query_pairs, find_by_key};
    }
    pub mod funcs
    {
        pub use crate::mods::net::{handle_auth_code, serve_all};
    }
}


