use serde::{Deserialize, Serialize};
use parse_display::Display;

/// Target of `list` or `search` commands. 
/// Used in `SearchCommandButtons` and `ListCommandButtons`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub enum Target
{
    #[display("{} 🎫")]
    Subscription,
    #[display("{} 📺")]
    PlayList,
}

/// Defines where to search. Used in `SearchCommandKB`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub enum SearchIn
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
    #[display("{} 🔎")] #[display(style = "Title Case")] #[default]
    SearchSettings,
    Execute,
    #[display(style = "Title Case")] #[display("{} 📤")]
    ResultLimit,
    #[display("{} 🗳")]
    TargetOptions,
    #[display("{0}")]
    Target(Target),
    #[display(style = "Title Case")] #[display("{} 📡")]
    SearchInOptions,
    #[display("{0}")]
    SearchIn(SearchIn),
    #[display(style = "Title Case")] #[display("{} 📤")]
    TextToSearch,
}

// TODO: Finish
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub enum Sorting
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
    #[display("{} 🧾")] #[display(style = "Title Case")] #[default]
    ListSettings,
    Execute,
    #[display(style = "Title Case")] #[display("{} 📤")]
    ResultLimit,
    #[display("{} 🗳")]
    TargetOptions,
    #[display("{0}")]
    Target(Target),
    #[display(style = "Title Case")] #[display("{} 📤")]
    SortingOptions,
    #[display("By {0}")]
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
        assert_eq!(SearchInOptions.to_string(), "Search By 📡");
        assert_eq!(SearchSettings.to_string(), "Search Config 🔎");
    }

    #[test]
    fn display_derive_test()
    {
        let serialized_enum: String = serde_json::to_string(&Buttons::SearchButtons(SearchInOptions)).unwrap();
        let deserialized_enum: Buttons = serde_json::from_str(&serialized_enum).unwrap();
        assert_eq!(deserialized_enum, Buttons::SearchButtons(SearchInOptions));
    }
}


