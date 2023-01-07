use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue::{Dialogue, ErasedStorage};
use teloxide::types::{CallbackQuery, InlineKeyboardMarkup, Message};

use crate::mods::inline_keyboards::types::{ListFilter, ListTarget, SearchMode, SearchTarget, SortMode};

/// A type that is returned in the end of handlers.
pub type MessageTriplet = (String, Option<InlineKeyboardMarkup>, Option<DialogueData>);

/// Framework wrapper storing all dialogue data.
/// Available in handlers.
pub type TheDialogue = Dialogue<DialogueData, ErasedStorage<DialogueData>>;

/// Stores settings for `search` command.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct SearchConfigData
{
    pub(crate) target: Option<SearchTarget>,
    pub(crate) search_by: Option<SearchMode>,
    pub(crate) result_limit: Option<u32>
}

/// Stores settings for `list` command.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct ListConfigData
{
    pub(crate) target: Option<ListTarget>,
    pub(crate) sort_by: Option<SortMode>,
    pub(crate) filter: Option<ListFilter>,
    pub(crate) result_limit: Option<u32>
}

/// Stores `dialogue state`.
#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub enum State
{
    #[default]
    Starting,
    SearchCommandActive(SearchConfigData),
    ListCommandActive(ListConfigData),
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


