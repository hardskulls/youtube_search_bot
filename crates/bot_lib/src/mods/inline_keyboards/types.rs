use serde::{Deserialize, Serialize};
use parse_display::Display;

/// Target to search for. Used in `SearchCommandKB`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum SearchTarget
{
    #[display("{} 🎫")]
    Subscription,
    #[display("{} 📺")]
    PlayList,
}

/// Defines ~~how~~ to search. Used in `SearchCommandKB`.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum SearchMode
{
    #[display("By {} 📋")]
    Title,
    #[display("By {} 📜")]
    Description,
}

/// TODO: Document and simplify this thing.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display, Default)]
pub(crate) enum SearchCommandKB
{
    #[display("{} 🔎")] #[display(style = "Title Case")] #[default]
    SearchConfig,
    #[display(style = "Title Case")] #[display("{} 📤")]
    ResultLimit,
    #[display("{} 🗳")]
    Target,
    #[display(style = "Title Case")] #[display("{} 📡")]
    SearchBy,
    #[display("{0}")]
    SearchByContent(SearchMode),
    #[display("{0}")]
    TargetContent(SearchTarget)
}

// TODO: Finish
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum ListFilter
{
    #[display("By {} 📊")]
    Hey,
    #[display("By {} 📑")]
    Ho, 
}

// TODO: Finish
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum SortMode
{
    #[display("{} 📊")]
    Date,
    #[display("{} 📑")]
    Alphabet,
}

// TODO: Finish
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum ListTarget
{
    #[display("{} 🎫")]
    Subscription,
    #[display("{} 📺")]
    PlayList,
}

/// TODO: Document and simplify this thing.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display, Default)]
pub(crate) enum ListCommandKB
{
    #[display("{} 🧾")] #[display(style = "Title Case")] #[default]
    ListConfig,
    #[display(style = "Title Case")] #[display("{} 📤")]
    ResultLimit,
    #[display("{} 🗳")]
    Target,
    #[display("{} 📊")]
    Filter,
    #[display(style = "Title Case")] #[display("{} 📤")]
    SortBy,
    #[display("{0}")]
    TargetContent(ListTarget),
    #[display("{0}")]
    FilterContent(ListFilter),
    #[display("By {0}")]
    SortContent(SortMode),
}

/// Main wrapper that includes all available keyboards.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Display)]
pub(crate) enum KeyBoard
{
    #[display("{0}")]
    SearchCommand(SearchCommandKB),
    #[display("{0}")]
    ListCommand(ListCommandKB),
}


#[cfg(test)]
mod tests
{
    // use to_debug::ToDebug;
    use crate::mods::inline_keyboards::types::SearchCommandKB::{SearchBy, SearchConfig};
    use super::*;

    #[test]
    fn serialize_enum_test()
    {
        assert_eq!(SearchBy.to_string(), "Search By 📡");
        assert_eq!(SearchConfig.to_string(), "Search Config 🔎");
    }

    #[test]
    fn display_derive_test()
    {
        let serialized_enum: String = serde_json::to_string(&KeyBoard::SearchCommand(SearchBy)).unwrap();
        let deserialized_enum: KeyBoard = serde_json::from_str(&serialized_enum).unwrap();
        assert_eq!(deserialized_enum, KeyBoard::SearchCommand(SearchBy));
    }
}


