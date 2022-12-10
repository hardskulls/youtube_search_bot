
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
    }
}

pub mod commands
{
    pub mod types { pub use crate::mods::commands::Command; }
    pub mod funcs { pub use crate::mods::commands::{handle_unknown_command, handle_commands, is_other_command}; }
}

pub mod dialogue
{
    pub mod funcs { pub use crate::mods::dialogue::{handle_callback_data, handle_text}; }
    pub mod types { pub use crate::mods::dialogue::types::{TheDialogue, DialogueData, State, ListConfigData, SearchConfigData, }; }
}

pub mod errors
{
    pub mod types { pub use crate::mods::errors::*; }
}

pub(crate) type StdResult<T, E> = Result<T, E>;


