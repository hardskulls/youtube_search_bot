use std::fmt::Display;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup};

use crate::keyboards::funcs::{button, inline_button};
use crate::keyboards::types::{Buttons, ListCommandButtons, SearchCommandButtons, SearchIn, Sorting, Target};

/// Creates `InlineKeyboardMarkup`.
pub trait CreateKB
{ fn create_kb(&self) -> Option<InlineKeyboardMarkup>; }

impl CreateKB for SearchCommandButtons
{
    fn create_kb(&self) -> Option<InlineKeyboardMarkup>
    {
        let broken_kb =
            |_| InlineKeyboardButton::new("Broken button 🚧", InlineKeyboardButtonKind::CallbackData("hgjhgjh".to_owned()));
        match *self
        {
            SearchCommandButtons::ResultLimit => None,
            SearchCommandButtons::TextToSearch => None,
            SearchCommandButtons::TargetOptions =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::Target(Target::Subscription))).unwrap_or_else(broken_kb))
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::Target(Target::PlayList))).unwrap_or_else(broken_kb))
                    .append_to_row(1, inline_button("Cancel ❌", Buttons::SearchButtons(SearchCommandButtons::SearchSettings)).unwrap_or_else(broken_kb))
                    .into(),
            SearchCommandButtons::SearchInOptions =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::SearchIn(SearchIn::Title))).unwrap_or_else(broken_kb))
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::SearchIn(SearchIn::Description))).unwrap_or_else(broken_kb))
                    .append_to_row(1, inline_button("Cancel ❌", Buttons::SearchButtons(SearchCommandButtons::SearchSettings)).unwrap_or_else(broken_kb))
                    .into(),
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::TargetOptions)).unwrap_or_else(broken_kb))
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::SearchInOptions)).unwrap_or_else(broken_kb))
                    .append_to_row(1, button(Buttons::SearchButtons(SearchCommandButtons::ResultLimit)).unwrap_or_else(broken_kb))
                    .append_to_row(1, button(Buttons::SearchButtons(SearchCommandButtons::TextToSearch)).unwrap_or_else(broken_kb))
                    .append_to_row(2, button(Buttons::SearchButtons(SearchCommandButtons::Execute)).unwrap_or_else(broken_kb))
                    .append_to_row(2, inline_button("Cancel ❌", Buttons::SearchButtons(SearchCommandButtons::SearchSettings)).unwrap_or_else(broken_kb))
                    .into(),
        }
    }
}

impl CreateKB for ListCommandButtons
{
    fn create_kb(&self) -> Option<InlineKeyboardMarkup>
    {
        let broken_kb =
            |_| InlineKeyboardButton::new("Broken button 🚧", InlineKeyboardButtonKind::CallbackData("hgjhgjh".to_owned()));
        match *self
        {
            ListCommandButtons::ResultLimit => None,
            ListCommandButtons::TargetOptions =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::Target(Target::Subscription))).unwrap_or_else(broken_kb))
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::Target(Target::PlayList))).unwrap_or_else(broken_kb))
                    .append_to_row(1, inline_button("Cancel ❌", Buttons::ListButtons(ListCommandButtons::ListSettings)).unwrap_or_else(broken_kb))
                    .into(),
            ListCommandButtons::SortingOptions =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::Sorting(Sorting::Alphabetical))).unwrap_or_else(broken_kb))
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::Sorting(Sorting::Date))).unwrap_or_else(broken_kb))
                    .append_to_row(1, inline_button("Cancel ❌", Buttons::ListButtons(ListCommandButtons::ListSettings)).unwrap_or_else(broken_kb))
                    .into(),
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::TargetOptions)).unwrap_or_else(broken_kb))
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::SortingOptions)).unwrap_or_else(broken_kb))
                    .append_to_row(1, button(Buttons::ListButtons(ListCommandButtons::ResultLimit)).unwrap_or_else(broken_kb))
                    .append_to_row(2, button(Buttons::ListButtons(ListCommandButtons::Execute)).unwrap_or_else(broken_kb))
                    .append_to_row(2, inline_button("Cancel ❌", Buttons::ListButtons(ListCommandButtons::ListSettings)).unwrap_or_else(broken_kb))
                    .into()
        }
    }
}


/// Text to show in message with inline keyboard.
pub trait KeyboardText
{ fn kb_text(&self) -> String; }

impl KeyboardText for SearchCommandButtons
{
    fn kb_text(&self) -> String
    {
        match *self
        {
            SearchCommandButtons::ResultLimit => "Choose result limit 📇",
            SearchCommandButtons::TargetOptions => "Choose what you want to search 🔎",
            SearchCommandButtons::SearchInOptions => "Choose how you want to search 📋",
            SearchCommandButtons::TextToSearch => "Send the text you want to search 📋",
            _ => "Set up your search command settings ⚙",
        }
        .to_owned()
    }
}

impl KeyboardText for ListCommandButtons
{
    fn kb_text(&self) -> String
    {
        match *self
        {
            ListCommandButtons::ResultLimit => "Choose result limit 📇",
            ListCommandButtons::TargetOptions => "Choose what you want to search 🔎",
            ListCommandButtons::SortingOptions => "Choose result sorting 📋",
            _ => "Set up your list command settings ⚙",
        }
        .to_owned()
    }
}


pub trait ButtonText: Display
{
    fn button_text(&self) -> String
    {
        self.to_string()
    }
}


impl ButtonText for Buttons {}



impl ButtonText for SearchCommandButtons {}

impl ButtonText for SearchIn {}

impl ButtonText for Target {}



impl ButtonText for ListCommandButtons {}

impl ButtonText for Sorting {}



// TODO: This trait gives strange error when used.
/*pub trait CreateKB
{
    fn search_config() -> InlineKeyboardMarkup
    {
        let mut row_1: Vec<InlineKeyboardButton> = keyboard_new_row(3);
        row_1[0] = inline_button("Result Limit", SearchCommandKB::ResultLimit);
        row_1[1] = inline_button("Target", SearchCommandKB::Target);
        row_1[2] = inline_button("Search By", SearchCommandKB::SearchBy);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", SearchCommandKB::SearchConfig);
        InlineKeyboardMarkup { inline_keyboard: vec![row_1, row_2] }
    }

    fn search_target() -> InlineKeyboardMarkup
    {
        let mut row_1: Vec<InlineKeyboardButton> = keyboard_new_row(2);
        row_1[0] = inline_button("Subscription", SearchTarget::Subscription);
        row_1[1] = inline_button("Playlist", SearchTarget::PlayList);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", SearchCommandKB::SearchConfig);
        InlineKeyboardMarkup { inline_keyboard: vec![row_1, row_2] }
    }

    fn search_by() -> InlineKeyboardMarkup
    {
        let mut row_1: Vec<InlineKeyboardButton> = keyboard_new_row(2);
        row_1[0] = inline_button("Title", SearchBy::Title);
        row_1[1] = inline_button("Description", SearchBy::Description);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", SearchCommandKB::SearchConfig);
        InlineKeyboardMarkup { inline_keyboard: vec![row_1, row_2] }
    }

    fn list_config() -> InlineKeyboardMarkup
    {
        let mut row_1: Vec<InlineKeyboardButton> = keyboard_new_row(4);
        row_1[0] = inline_button("Result Limit", ListCommandKB::ResultLimit);
        row_1[1] = inline_button("Target", ListCommandKB::Target);
        row_1[2] = inline_button("Filter", ListCommandKB::Filter);
        row_1[3] = inline_button("Sort By", ListCommandKB::SortBy);
        let mut row_2 = keyboard_new_row(1);
        row_2[0] = inline_button("Cancel", ListCommandKB::ListConfig);
        InlineKeyboardMarkup { inline_keyboard: vec![row_1, row_2] }
    }
}*/

#[cfg(test)]
mod tests
{
    
}


