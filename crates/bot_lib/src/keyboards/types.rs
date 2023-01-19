use serde::{Deserialize, Serialize};
use parse_display::Display;

/// Target of `list` or `search` commands. 
/// Used in `SearchCommandButtons` and `ListCommandButtons`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum Target
{
    #[display("{} 🎫")]
    Subscription,
    #[display("{} 📺")]
    PlayList,
}

/// Defines where to search. Used in `SearchCommandKB`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum SearchIn
{
    #[display("By {} 📋")]
    Title,
    #[display("By {} 📜")]
    Description,
}

/// List of `Inline Keyboard` buttons for `search` bot command.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display, Default)]
pub(crate) enum SearchCommandButtons
{
    #[display("{} 🔎")] #[display(style = "Title case")] #[default]
    SearchSettings,
    #[display("{} ✅")]
    Execute,
    #[display(style = "Title case")] #[display("{} 📤")]
    ResultLimit,
    #[display("Target 🗳")]
    TargetOptions,
    #[display("{0}")]
    Target(Target),
    #[display("Search in 📡")]
    SearchInOptions,
    #[display("{0}")]
    SearchIn(SearchIn),
    #[display(style = "Title case")] #[display("{} 📤")]
    TextToSearch,
}

// TODO: Finish
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum Sorting
{
    #[display("{} 📊")]
    Date,
    #[display("{} 📑")]
    Alphabetical,
}

/// List of `Inline Keyboard` buttons for `list` bot command.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display, Default)]
pub(crate) enum ListCommandButtons
{
    #[display("{} 🧾")] #[display(style = "Title case")] #[default]
    ListSettings,
    #[display("{} ✅")]
    Execute,
    #[display(style = "Title case")] #[display("{} 📤")]
    ResultLimit,
    #[display("Target 🗳")]
    TargetOptions,
    #[display("{0}")]
    Target(Target),
    #[display("Sorting 📤")]
    SortingOptions,
    #[display("{0}")]
    Sorting(Sorting),
    //#[display("{} 📊")]
    //Filter,
    //#[display("{0}")]
    //FilterContent(ListFilter),
}

/// Main wrapper that includes all available keyboards.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum Buttons
{
    #[display("{0}")]
    SearchButtons(SearchCommandButtons),
    #[display("{0}")]
    ListButtons(ListCommandButtons),
}


#[cfg(test)]
mod tests
{
    use crate::keyboards::types::SearchCommandButtons::{SearchInOptions, SearchSettings};
    // use to_debug::ToDebug;
    use super::*;

    #[test]
    fn serialize_enum_test()
    {
        assert_eq!(SearchInOptions.to_string(), "Search in 📡");
        assert_eq!(SearchSettings.to_string(), "Search settings 🔎");
    }

    #[test]
    fn display_derive_test()
    {
        let serialized_enum: String = serde_json::to_string(&Buttons::SearchButtons(SearchInOptions)).unwrap();
        let deserialized_enum: Buttons = serde_json::from_str(&serialized_enum).unwrap();
        assert_eq!(deserialized_enum, Buttons::SearchButtons(SearchInOptions));
    }
}


