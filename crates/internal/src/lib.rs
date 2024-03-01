pub type StdRes<T, E> = Result<T, E>;

pub type MonoRes<T> = StdRes<T, T>;

pub type DefaultError = String;

pub type Res<T, E = eyre::Report> = eyre::Result<T, E>;

pub mod auth_server {
    pub use crate::model::auth_server::{handle_auth_code, serve_all};
}

pub mod commands {
    pub use crate::model::commands::types::Command;
}

pub mod handlers {
    pub use crate::controllers::callback::handle_callback;
    pub use crate::controllers::commands::{
        handle_commands, handle_unknown_command, is_other_command,
    };
    pub use crate::controllers::text::handle_text;
}

pub mod errors {
    pub use crate::model::errors::*;
}

pub mod dialogue {
    pub use crate::model::dialogue::types::DialogueData;
}

pub(crate) mod model;
pub(crate) mod view {
    pub(crate) mod funcs;
    pub(crate) mod types;
}
pub(crate) mod controllers {
    pub(crate) mod callback;
    pub(crate) mod commands;
    pub(crate) mod text;
}
