
use parse_display::Display;
use serde::{Deserialize, Serialize};

use crate::model::net::traits::{RespTargetPlaylists, RespTargetSubscriptions};


/*/// Target of `list` or `search` commands.
/// Used in `SearchCommandButtons` and `ListCommandButtons`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum Target
{
    #[display("{} 🏷")]
    Subscription,
    #[display("{} ⏯")]
    PlayList,
}*/

/// Target of `list` or `search` commands.
/// Used in `SearchCommandButtons` and `ListCommandButtons`.
#[derive(Clone, Debug, Serialize, Deserialize, Display)]
pub enum Requestable
{
    #[display("Subscription 📋")]
    Subscription(RespTargetSubscriptions),
    #[display("Playlist 📜")]
    Playlist(RespTargetPlaylists)
}

/// Defines where to search. Used in `SearchCommandKB`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub enum SearchIn
{
    #[display("{} 📋")]
    Title,
    #[display("{} 📜")]
    Description,
}

/// List of `Inline Keyboard` buttons for `search` bot command.
#[derive(Debug, Clone, Serialize, Deserialize, Display, Default)]
pub enum SearchCommandButtons
{
    #[display("{} 🔎")] #[display(style = "Title case")] #[default]
    SearchSettings,
    #[display("{} ✅")]
    Execute,
    #[display(style = "Title case")] #[display("{} 🧮")]
    ResultLimit,
    #[display("Target 🎯")]
    TargetOptions,
    #[display("{0}")]
    Target(Requestable),
    #[display("Search in 💳")]
    SearchInOptions,
    #[display("{0}")]
    SearchIn(SearchIn),
    #[display(style = "Title case")] #[display("{} 💬")]
    TextToSearch,
}

/// Sorting foe list command.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub enum Sorting
{
    #[display("{} 🗓")]
    Date,
    #[display("{} 🔠")]
    Alphabetical,
}

/// List of `Inline Keyboard` buttons for `list` bot command.
#[derive(Debug, Clone, Serialize, Deserialize, Display, Default)]
pub enum ListCommandButtons
{
    #[display("{} 📃")] #[display(style = "Title case")] #[default]
    ListSettings,
    #[display("{} ✅")]
    Execute,
    #[display(style = "Title case")] #[display("{} 🧮")]
    ResultLimit,
    #[display("Target 🎯")]
    TargetOptions,
    #[display("{0}")]
    Target(Requestable),
    #[display("Sorting 🗃")]
    SortingOptions,
    #[display("{0}")]
    Sorting(Sorting),
}

/// Main wrapper that includes all available keyboards.
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum Buttons
{
    #[display("{0}")]
    SearchButtons(SearchCommandButtons),
    #[display("{0}")]
    ListButtons(ListCommandButtons),
}


#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests
{
    use crate::model::keyboards::traits::ButtonText;
    use crate::model::keyboards::types::SearchCommandButtons::{SearchInOptions, SearchSettings};
    
    // use to_debug::ToDebug;
    use super::*;
    
    #[test]
    fn serialize_enum_test()
    {
        assert_eq!(SearchInOptions.to_string(), "Search in 💳");
        assert_eq!(SearchSettings.button_text(), "Search settings 🔎");
    }

    #[test]
    fn display_derive_test()
    {
        let serialized_enum : String = serde_json::to_string(&Buttons::SearchButtons(SearchInOptions)).unwrap();
        let deserialized_enum : Buttons = serde_json::from_str(&serialized_enum).unwrap();
        assert!(matches!(deserialized_enum, Buttons::SearchButtons(SearchInOptions)));
    }
}


