use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::{Dialogue, ErasedStorage};
use teloxide::types::{CallbackQuery, InlineKeyboardMarkup, Message};

use error_traits::WrapInErr;

use crate::keyboards::types::{SearchIn, Sorting, Target};
use crate::StdResult;
use crate::utils::print_if_none;

/// A type that is returned in the end of handlers.
pub type MessageTriplet = (String, Option<InlineKeyboardMarkup>, Option<DialogueData>);

pub type MessageAndData<T> = (teloxide::requests::JsonRequest<T>, Option<DialogueData>);

/// Framework wrapper storing all dialogue data.
/// Available in handlers.
pub type TheDialogue = Dialogue<DialogueData, ErasedStorage<DialogueData>>;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SearchConfig
{
    pub target: Target,
    pub result_limit: u32,
    pub search_in: SearchIn,
    pub text_to_search: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ListConfig
{
    pub target: Target,
    pub result_limit: u32,
    pub sorting: Sorting,
}

/// Stores settings for `search` command.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct SearchCommandSettings
{
    pub target: Option<Target>,
    pub result_limit: Option<u32>,
    pub search_in: Option<SearchIn>,
    pub text_to_search: Option<String>,
}

impl SearchCommandSettings
{
    pub fn build_config(self) -> StdResult<SearchConfig, String>
    {
        match self
        {
            Self { target: Some(t), result_limit: Some(r), search_in: Some(s), text_to_search: Some(text) } =>
                Ok(SearchConfig { target: t, result_limit: r, search_in: s, text_to_search: text }),
            Self { target: t, result_limit: r, search_in: s, text_to_search: text } =>
                {
                    let t = print_if_none(t, "\nTarget");
                    let r = print_if_none(r, "\nResult Limit");
                    let s = print_if_none(s, "\nSearch In");
                    let text = print_if_none(text, "\nText To Search");
                    format!("You are missing {t}{r}{s}{text}").in_err()
                }
        }
    }
}

/// Stores settings for `list` command.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct ListCommandSettings
{
    pub target: Option<Target>,
    pub result_limit: Option<u32>,
    pub sorting: Option<Sorting>
}

impl ListCommandSettings
{
    pub fn build_config(self) -> StdResult<ListConfig, String>
    {
        match self
        {
            Self { target: Some(t), result_limit: Some(r), sorting: Some(s) } =>
                Ok(ListConfig { target: t, result_limit: r, sorting: s }),
            Self { target: t, result_limit: r, sorting: s } =>
                {
                    let t = print_if_none(t, "\nTarget");
                    let r = print_if_none(r, "\nResult Limit");
                    let s = print_if_none(s, "\nSorting");
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


