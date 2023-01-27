use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::{Dialogue, ErasedStorage};
use teloxide::types::{CallbackQuery, InlineKeyboardMarkup, Message};

use error_traits::WrapInErr;

use crate::keyboards::types::{SearchIn, Sorting, Target};
use crate::StdResult;
use crate::utils::{HTMLise, print_if_none};

/// A type that is returned in the end of handlers.
pub type MessageTriplet = (String, Option<InlineKeyboardMarkup>, Option<DialogueData>);

pub type MessageAndData<T> = (teloxide::requests::JsonRequest<T>, Option<DialogueData>);

/// Framework wrapper storing all dialogue data.
/// Available in handlers.
pub type TheDialogue = Dialogue<DialogueData, ErasedStorage<DialogueData>>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct SearchConfig
{
    pub(crate) target: Target,
    pub(crate) result_limit: u32,
    pub(crate) search_in: SearchIn,
    pub(crate) text_to_search: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub(crate) struct ListConfig
{
    pub(crate) target: Target,
    pub(crate) result_limit: u32,
    pub(crate) sorting: Sorting,
}

/// Stores settings for `search` command.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct SearchCommandSettings
{
    pub(crate) target: Option<Target>,
    pub(crate) result_limit: Option<u32>,
    pub(crate) search_in: Option<SearchIn>,
    pub(crate) text_to_search: Option<String>,
}

impl SearchCommandSettings
{
    pub(crate) fn build_config(self) -> StdResult<SearchConfig, String>
    {
        match self
        {
            Self { target: Some(t), result_limit: Some(r), search_in: Some(s), text_to_search: Some(text) } =>
                Ok(SearchConfig { target: t, result_limit: r, search_in: s, text_to_search: text }),
            Self { target: t, result_limit: r, search_in: s, text_to_search: text } =>
                {
                    let t = print_if_none(t, format!("\n🎯 {}", "Target".to_bold()).as_str());
                    let r = print_if_none(r, format!("\n🧮 {}", "Result limit".to_bold()));
                    let s = print_if_none(s, format!("\n💳 {}", "Search in".to_bold()));
                    let text = print_if_none(text, format!("\n💬 {}", "Text to search".to_bold()));
                    format!("You are missing {t}{r}{s}{text}").in_err()
                }
        }
    }
}

/// Stores settings for `list` command.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct ListCommandSettings
{
    pub(crate) target: Option<Target>,
    pub(crate) result_limit: Option<u32>,
    pub(crate) sorting: Option<Sorting>
}

impl ListCommandSettings
{
    pub(crate) fn build_config(self) -> StdResult<ListConfig, String>
    {
        match self
        {
            Self { target: Some(t), result_limit: Some(r), sorting: Some(s) } =>
                Ok(ListConfig { target: t, result_limit: r, sorting: s }),
            Self { target: t, result_limit: r, sorting: s } =>
                {
                    let t = print_if_none(t, format!("\n🎯 {}", "Target".to_bold()));
                    let r = print_if_none(r, format!("\n🧮 {}", "Result limit".to_bold()));
                    let s = print_if_none(s, format!("\n🗃 {}", "Sorting".to_bold()));
                    format!("You are missing {t}{r}{s}").in_err()
                }
        }
    }
}

/// Stores `dialogue state`.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub enum State
{
    #[default]
    Starting,
    SearchCommandActive(SearchCommandSettings),
    ListCommandActive(ListCommandSettings),
}

impl AsRef<State> for State
{
    #[inline]
    fn as_ref(&self) -> &State { self }
}

/// Main message with a keyboard attached.
/// Better than sending new inline keyboard each time.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct MessageWithKB { pub opt_message: Option<Message> }

/// Stores dialogue state and other required data.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct DialogueData
{
    pub state: State,
    pub message_with_kb: MessageWithKB,
    pub last_callback: Option<CallbackQuery>
}

pub enum Either<F, L>
{
    First(F),
    Last(L)
}


