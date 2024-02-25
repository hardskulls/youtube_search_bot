use parse_display::Display;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

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
pub enum Requestable {
    #[display("Subscription 📋")]
    Subscription(SubscriptionRequester),
    #[display("Playlist 📜")]
    Playlist(PlaylistRequester),
}

impl Debug for Requestable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Requestable::Subscription(..) => write!(f, "Subscription"),
            Requestable::Playlist(..) => write!(f, "Playlist"),
        }
    }
}

/// Defines where to search. Used in `SearchCommandKB`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub enum SearchIn {
    #[display("{} 📋")]
    Title,
    #[display("{} 📜")]
    Description,
}

/// List of `Inline Keyboard` buttons for `search` bot command.
#[derive(Debug, Clone, Serialize, Deserialize, Display, Default)]
pub enum SearchCommandButtons {
    #[display("{} 🔎")]
    #[display(style = "Title case")]
    #[default]
    ButtonList,
    #[display("{} ✅")]
    Execute,
    #[display(style = "Title case")]
    #[display("{} 🧮")]
    ResultLimit,
    #[display("Target 🎯")]
    TargetOptions,
    #[display("{0}")]
    Target(Requestable),
    #[display("Search in 💳")]
    SearchInOptions,
    #[display("{0}")]
    SearchIn(SearchIn),
    #[display(style = "Title case")]
    #[display("{} 💬")]
    TextToSearch,
}

/// Sorting foe list command.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub enum Sorting {
    #[display("{} 🗓")]
    Date,
    #[display("{} 🔠")]
    Alphabetical,
}

/// List of `Inline Keyboard` buttons for `list` bot command.
#[derive(Debug, Clone, Serialize, Deserialize, Display, Default)]
pub enum ListCommandButtons {
    #[display("{} 📃")]
    #[display(style = "Title case")]
    #[default]
    ButtonList,
    #[display("{} ✅")]
    Execute,
    #[display(style = "Title case")]
    #[display("{} 🧮")]
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

/// List of `Inline Keyboard` buttons for `search_videos_in_playlists` bot command.
#[derive(Debug, Clone, Serialize, Deserialize, Display, Default)]
pub enum SearchVideoInPlaylistsCommandButtons {
    #[display("{} 📃")]
    #[display(style = "Title case")]
    #[default]
    ButtonList,
    #[display("{} ✅")]
    Execute,
    #[display(style = "Title case")]
    #[display("{} 🧮")]
    ResultLimit,
    #[display("Search in 💳")]
    SearchInOptions,
    #[display("{0}")]
    SearchIn(SearchIn),
    #[display(style = "Title case")]
    #[display("{} 💬")]
    TextToSearch,
}

/// Main wrapper that includes all available keyboards.
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum Buttons {
    #[display("{0}")]
    SearchButtons(SearchCommandButtons),
    #[display("{0}")]
    ListButtons(ListCommandButtons),
    #[display("{0}")]
    SearchVideoInPlaylistsButtons(SearchVideoInPlaylistsCommandButtons),
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests {
    use crate::model::keyboards::traits::ButtonText;
    use crate::model::keyboards::types::SearchCommandButtons::{ButtonList, SearchInOptions};

    // use to_debug::ToDebug;
    use super::*;

    #[test]
    fn serialize_enum_test() {
        assert_eq!(SearchInOptions.to_string(), "Search in 💳");
        assert_eq!(ButtonList.button_text(), "Button list 🔎");
    }

    #[test]
    fn display_derive_test() {
        let serialized_enum: String =
            serde_json::to_string(&Buttons::SearchButtons(SearchInOptions)).unwrap();
        let deserialized_enum: Buttons = serde_json::from_str(&serialized_enum).unwrap();
        assert!(matches!(
            deserialized_enum,
            Buttons::SearchButtons(SearchInOptions)
        ));
    }

    #[test]
    fn display_derive_for_requestable_test() {
        assert_eq!(
            "Subscription 📋",
            Requestable::Subscription(SubscriptionRequester).button_text()
        );
        assert_eq!(
            "Subscription 📋",
            Requestable::Subscription(SubscriptionRequester).to_string()
        );
        let b = Buttons::SearchButtons(SearchCommandButtons::Target(Requestable::Subscription(
            SubscriptionRequester,
        )));
        assert_eq!(
            b.button_text(),
            Requestable::Subscription(SubscriptionRequester).button_text()
        );
        assert_eq!(
            Requestable::Subscription(SubscriptionRequester).to_string(),
            Requestable::Subscription(SubscriptionRequester).button_text()
        );
    }
}
