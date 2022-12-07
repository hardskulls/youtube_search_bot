
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
    pub mod funcs { pub use crate::mods::dialogue::{callback_handling::handle_callback_data, text_handling::{handle_text, handle_start_state}}; }
    pub mod types { pub use crate::mods::dialogue::types::{TheDialogue, DialogueData, State, ListConfigData, SearchConfigData, }; }
}

pub mod errors
{
    pub mod types { pub use crate::mods::errors::*; }
}


