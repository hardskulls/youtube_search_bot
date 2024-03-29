use std::fmt::Display;

use crate::model::keyboards::funcs::{button, inline_button};
use crate::model::keyboards::types::{
    Buttons, ListCommandButtons, Requestable, SearchCommandButtons, SearchIn,
    SearchVideoInPlaylistsCommandButtons, Sorting,
};
use crate::model::net::types::{PlaylistRequester, SubscriptionRequester};
use crate::model::utils::HTMLise;
use teloxide::types::InlineKeyboardMarkup;

/// Creates `InlineKeyboardMarkup`.
pub(crate) trait CreateKB {
    fn create_kb(&self) -> Option<InlineKeyboardMarkup>;
}

impl CreateKB for SearchCommandButtons {
    fn create_kb(&self) -> Option<InlineKeyboardMarkup> {
        use Buttons::SearchButtons;
        match *self {
            SearchCommandButtons::ResultLimit | SearchCommandButtons::TextToSearch => None,
            SearchCommandButtons::TargetOptions => InlineKeyboardMarkup::default()
                .append_to_row(
                    0,
                    button(SearchButtons(SearchCommandButtons::Target(
                        Requestable::Subscription(SubscriptionRequester),
                    ))),
                )
                .append_to_row(
                    0,
                    button(SearchButtons(SearchCommandButtons::Target(
                        Requestable::Playlist(PlaylistRequester),
                    ))),
                )
                .append_to_row(
                    1,
                    inline_button("Cancel ❌", SearchButtons(SearchCommandButtons::ButtonList)),
                )
                .into(),
            SearchCommandButtons::SearchInOptions => InlineKeyboardMarkup::default()
                .append_to_row(
                    0,
                    button(SearchButtons(SearchCommandButtons::SearchIn(
                        SearchIn::Title,
                    ))),
                )
                .append_to_row(
                    0,
                    button(SearchButtons(SearchCommandButtons::SearchIn(
                        SearchIn::Description,
                    ))),
                )
                .append_to_row(
                    1,
                    inline_button("Cancel ❌", SearchButtons(SearchCommandButtons::ButtonList)),
                )
                .into(),
            SearchCommandButtons::Target(_)
            | SearchCommandButtons::SearchIn(_)
            | SearchCommandButtons::ButtonList
            | SearchCommandButtons::Execute => InlineKeyboardMarkup::default()
                .append_to_row(
                    0,
                    button(SearchButtons(SearchCommandButtons::TargetOptions)),
                )
                .append_to_row(
                    0,
                    button(SearchButtons(SearchCommandButtons::SearchInOptions)),
                )
                .append_to_row(1, button(SearchButtons(SearchCommandButtons::ResultLimit)))
                .append_to_row(1, button(SearchButtons(SearchCommandButtons::TextToSearch)))
                .append_to_row(2, button(SearchButtons(SearchCommandButtons::Execute)))
                .append_to_row(
                    2,
                    inline_button("Cancel ❌", SearchButtons(SearchCommandButtons::ButtonList)),
                )
                .into(),
        }
    }
}

impl CreateKB for ListCommandButtons {
    fn create_kb(&self) -> Option<InlineKeyboardMarkup> {
        use Buttons::ListButtons;
        match *self {
            ListCommandButtons::ResultLimit => None,
            ListCommandButtons::TargetOptions => InlineKeyboardMarkup::default()
                .append_to_row(
                    0,
                    button(ListButtons(ListCommandButtons::Target(
                        Requestable::Subscription(SubscriptionRequester),
                    ))),
                )
                .append_to_row(
                    0,
                    button(ListButtons(ListCommandButtons::Target(
                        Requestable::Playlist(PlaylistRequester),
                    ))),
                )
                .append_to_row(
                    1,
                    inline_button("Cancel ❌", ListButtons(ListCommandButtons::ButtonList)),
                )
                .into(),
            ListCommandButtons::SortingOptions => InlineKeyboardMarkup::default()
                .append_to_row(
                    0,
                    button(ListButtons(ListCommandButtons::Sorting(
                        Sorting::Alphabetical,
                    ))),
                )
                .append_to_row(
                    0,
                    button(ListButtons(ListCommandButtons::Sorting(Sorting::Date))),
                )
                .append_to_row(
                    1,
                    inline_button("Cancel ❌", ListButtons(ListCommandButtons::ButtonList)),
                )
                .into(),
            ListCommandButtons::Target(_)
            | ListCommandButtons::Sorting(_)
            | ListCommandButtons::ButtonList
            | ListCommandButtons::Execute => InlineKeyboardMarkup::default()
                .append_to_row(0, button(ListButtons(ListCommandButtons::TargetOptions)))
                .append_to_row(0, button(ListButtons(ListCommandButtons::SortingOptions)))
                .append_to_row(1, button(ListButtons(ListCommandButtons::ResultLimit)))
                .append_to_row(2, button(ListButtons(ListCommandButtons::Execute)))
                .append_to_row(
                    2,
                    inline_button("Cancel ❌", ListButtons(ListCommandButtons::ButtonList)),
                )
                .into(),
        }
    }
}

impl CreateKB for SearchVideoInPlaylistsCommandButtons {
    fn create_kb(&self) -> Option<InlineKeyboardMarkup> {
        use crate::model::keyboards::types::Buttons::SearchVideoInPlaylistsButtons;
        use SearchVideoInPlaylistsCommandButtons::{
            ButtonList, Execute, ResultLimit, SearchInOptions, TextToSearch,
        };
        match *self {
            ResultLimit | TextToSearch => None,
            SearchInOptions => InlineKeyboardMarkup::default()
                .append_to_row(
                    0,
                    button(SearchVideoInPlaylistsButtons(
                        SearchVideoInPlaylistsCommandButtons::SearchIn(SearchIn::Title),
                    )),
                )
                .append_to_row(
                    0,
                    button(SearchVideoInPlaylistsButtons(
                        SearchVideoInPlaylistsCommandButtons::SearchIn(SearchIn::Description),
                    )),
                )
                .append_to_row(
                    1,
                    inline_button("Cancel ❌", SearchVideoInPlaylistsButtons(ButtonList)),
                )
                .into(),
            ButtonList | Execute | SearchVideoInPlaylistsCommandButtons::SearchIn(_) => {
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(SearchVideoInPlaylistsButtons(TextToSearch)))
                    .append_to_row(0, button(SearchVideoInPlaylistsButtons(SearchInOptions)))
                    .append_to_row(1, button(SearchVideoInPlaylistsButtons(ResultLimit)))
                    .append_to_row(2, button(SearchVideoInPlaylistsButtons(Execute)))
                    .append_to_row(
                        2,
                        inline_button("Cancel ❌", SearchVideoInPlaylistsButtons(ButtonList)),
                    )
                    .into()
            }
        }
    }
}

/// Text to show in message with inline keyboard.
pub(crate) trait KeyboardText {
    fn kb_text(&self) -> String;
}

impl KeyboardText for SearchCommandButtons {
    fn kb_text(&self) -> String {
        match *self {
            SearchCommandButtons::ResultLimit => "Choose result limit 🧮".into(),
            SearchCommandButtons::TargetOptions => format!(
                "Choose {what} you want to search 🎯",
                what = "what".to_bold()
            ),
            SearchCommandButtons::SearchInOptions => format!(
                "Choose {_where} you want to search 💳",
                _where = "where".to_bold()
            ),
            SearchCommandButtons::TextToSearch => format!(
                "Send the {text} you want to search 💬",
                text = "text".to_bold()
            ),
            SearchCommandButtons::Execute
            | SearchCommandButtons::ButtonList
            | SearchCommandButtons::Target(_)
            | SearchCommandButtons::SearchIn(_) => "Search command settings ⚙".into(),
        }
    }
}

impl KeyboardText for ListCommandButtons {
    fn kb_text(&self) -> String {
        match *self {
            ListCommandButtons::ResultLimit => "Choose result limit 🧮".into(),
            ListCommandButtons::TargetOptions => format!(
                "Choose {what} you want to search 🎯",
                what = "what".to_bold()
            ),
            ListCommandButtons::SortingOptions => "Choose result sorting 🗃".into(),
            ListCommandButtons::Execute
            | ListCommandButtons::ButtonList
            | ListCommandButtons::Target(_)
            | ListCommandButtons::Sorting(_) => "List command settings ⚙".into(),
        }
    }
}

impl KeyboardText for SearchVideoInPlaylistsCommandButtons {
    fn kb_text(&self) -> String {
        match *self {
            SearchVideoInPlaylistsCommandButtons::ResultLimit => "Choose result limit 🧮".into(),
            SearchVideoInPlaylistsCommandButtons::TextToSearch => format!(
                "Send the {text} you want to search 💬",
                text = "text".to_bold()
            ),
            _ => "Search video in playlists command settings ⚙".into(),
        }
    }
}

pub(crate) trait ButtonText: Display {
    fn button_text(&self) -> String {
        self.to_string()
    }
}

impl ButtonText for Buttons {}

impl ButtonText for Requestable {}

impl ButtonText for SearchCommandButtons {}
impl ButtonText for SearchIn {}

impl ButtonText for ListCommandButtons {}
impl ButtonText for Sorting {}

impl ButtonText for SearchVideoInPlaylistsCommandButtons {}

// TODO : This trait gives strange error when used.
/*pub(crate) trait CreateKB
{
    fn search_config() -> InlineKeyboardMarkup
    {
        let mut row_1 : Vec<InlineKeyboardButton> = keyboard_new_row(3);
        row_1[0] = inline_button("Result Limit", SearchCommandKB::ResultLimit);
        row_1[1] = inline_button("Target", SearchCommandKB::Target);
        row_1[2] = inline_button("Search By", SearchCommandKB::SearchBy);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", SearchCommandKB::SearchConfig);
        InlineKeyboardMarkup { inline_keyboard : vec![row_1, row_2] }
    }

    fn search_target() -> InlineKeyboardMarkup
    {
        let mut row_1 : Vec<InlineKeyboardButton> = keyboard_new_row(2);
        row_1[0] = inline_button("Subscription", SearchTarget::Subscription);
        row_1[1] = inline_button("Playlist", SearchTarget::PlayList);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", SearchCommandKB::SearchConfig);
        InlineKeyboardMarkup { inline_keyboard : vec![row_1, row_2] }
    }

    fn search_by() -> InlineKeyboardMarkup
    {
        let mut row_1 : Vec<InlineKeyboardButton> = keyboard_new_row(2);
        row_1[0] = inline_button("Title", SearchBy::Title);
        row_1[1] = inline_button("Description", SearchBy::Description);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", SearchCommandKB::SearchConfig);
        InlineKeyboardMarkup { inline_keyboard : vec![row_1, row_2] }
    }

    fn list_config() -> InlineKeyboardMarkup
    {
        let mut row_1 : Vec<InlineKeyboardButton> = keyboard_new_row(4);
        row_1[0] = inline_button("Result Limit", ListCommandKB::ResultLimit);
        row_1[1] = inline_button("Target", ListCommandKB::Target);
        row_1[2] = inline_button("Filter", ListCommandKB::Filter);
        row_1[3] = inline_button("Sort By", ListCommandKB::SortBy);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", ListCommandKB::ListConfig);
        InlineKeyboardMarkup { inline_keyboard : vec![row_1, row_2] }
    }
}*/

#[cfg(test)]
mod tests {}
