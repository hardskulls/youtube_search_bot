pub(crate) mod auth_server;
pub(crate) mod db;
pub(crate) mod errors;
pub(crate) mod utils;
pub(crate) mod dialogue {
    pub(crate) mod funcs;
    pub(crate) mod types;
}
pub(crate) mod youtube {
    pub(crate) mod types;
    pub(crate) mod funcs {
        pub(crate) mod common;
        pub(crate) mod list_cmd;
        pub(crate) mod search_cmd;
        pub(crate) mod search_videos_in_playlists;
    }
    pub(crate) mod traits;
}
pub(crate) mod net {
    pub(crate) mod funcs;
    pub(crate) mod traits;
    pub(crate) mod types;
}
pub(crate) mod commands {
    pub(crate) mod funcs;
    pub(crate) mod types;
}
pub(crate) mod keyboards {
    pub(crate) mod funcs;
    pub(crate) mod traits;
    pub(crate) mod types;
}
pub(crate) mod handlers {
    pub(crate) mod commands;
    pub(crate) mod text;
    pub(crate) mod callback {
        pub(crate) mod common;
        pub(crate) mod list_cmd;
        pub(crate) mod search_cmd;
        pub(crate) mod search_videos_in_playlits;
    }
}
