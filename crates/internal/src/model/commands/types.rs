
use teloxide::macros::BotCommands;


/// List of commands available in the bot.
#[derive(Clone, BotCommands)]
#[command(rename_rule = "snake_case", description = "These commands are available")]
pub enum Command
{
    #[command(description = "Start Bot")]
    Start,
    #[command(description = "Info")]
    Info,
    #[command(description = "Search Something")]
    Search,
    #[command(description = "List Something")]
    List,
    #[command(description = "Log Out")]
    LogOut,
    #[command(description = "Search Videos In My Playlists")]
    SearchVideosInMyPlaylists,
}


