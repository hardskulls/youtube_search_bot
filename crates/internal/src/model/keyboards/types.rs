use std::fmt::{Debug, Formatter};
use parse_display::Display;
use serde::{Deserialize, Serialize};

use crate::model::net::types::{PlaylistRequester, SubscriptionRequester};


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
#[derive(Clone, Serialize, Deserialize, Display)]
pub enum Requestable
{
    #[display("Subscription 📋")]
    Subscription(SubscriptionRequester),
    #[display("Playlist 📜")]
    Playlist(PlaylistRequester)
}

impl Debug for Requestable
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self
        {
            Requestable::Subscription(..) => write!(f, "Subscription"),
            Requestable::Playlist(..) => write!(f, "Playlist")
        }
    }
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
        let serialized_enum: String = serde_json::to_string(&Buttons::SearchButtons(SearchInOptions)).unwrap();
        let deserialized_enum: Buttons = serde_json::from_str(&serialized_enum).unwrap();
        assert!(matches!(deserialized_enum, Buttons::SearchButtons(SearchInOptions)));
    }
    
    #[test]
    fn display_derive_for_requestable_test()
    {
        assert_eq!("Subscription 📋", Requestable::Subscription(SubscriptionRequester).button_text());
        assert_eq!("Subscription 📋", Requestable::Subscription(SubscriptionRequester).to_string());
        let b = Buttons::SearchButtons(SearchCommandButtons::Target(Requestable::Subscription(SubscriptionRequester)));
        assert_eq!(b.button_text(), Requestable::Subscription(SubscriptionRequester).button_text());
        assert_eq!(Requestable::Subscription(SubscriptionRequester).to_string(), Requestable::Subscription(SubscriptionRequester).button_text());
    }
}


