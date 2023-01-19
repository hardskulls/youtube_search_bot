use std::fmt::Display;

use teloxide::types::InlineKeyboardMarkup;

use crate::keyboards::funcs::{button, inline_button};
use crate::keyboards::types::{Buttons, ListCommandButtons, SearchCommandButtons, SearchIn, Sorting, Target};

/// Creates `InlineKeyboardMarkup`.
pub trait CreateKB
{ fn create_kb(&self) -> Option<InlineKeyboardMarkup>; }

impl CreateKB for SearchCommandButtons
{
    fn create_kb(&self) -> Option<InlineKeyboardMarkup>
    {
        match *self
        {
            SearchCommandButtons::ResultLimit => None,
            SearchCommandButtons::TargetOptions =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::Target(Target::Subscription))))
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::Target(Target::PlayList))))
                    .append_to_row(1, inline_button("Cancel ❌", Buttons::SearchButtons(SearchCommandButtons::SearchSettings)))
                    .into(),
            SearchCommandButtons::SearchInOptions =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::SearchIn(SearchIn::Title))))
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::SearchIn(SearchIn::Description))))
                    .append_to_row(1, inline_button("Cancel ❌", Buttons::SearchButtons(SearchCommandButtons::SearchSettings)))
                    .into(),
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::ResultLimit)))
                    .append_to_row(0, button(Buttons::SearchButtons(SearchCommandButtons::TargetOptions)))
                    .append_to_row(1, button(Buttons::SearchButtons(SearchCommandButtons::SearchInOptions)))
                    .append_to_row(1, button(Buttons::SearchButtons(SearchCommandButtons::TextToSearch)))
                    .append_to_row(2, button(Buttons::SearchButtons(SearchCommandButtons::Execute)))
                    .append_to_row(2, inline_button("Cancel ❌", Buttons::SearchButtons(SearchCommandButtons::SearchSettings)))
                    .into(),
        }
    }
}

impl CreateKB for ListCommandButtons
{
    fn create_kb(&self) -> Option<InlineKeyboardMarkup>
    {
        match *self
        {
            ListCommandButtons::ResultLimit => None,
            ListCommandButtons::TargetOptions =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::Target(Target::Subscription))))
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::Target(Target::PlayList))))
                    .append_to_row(1, inline_button("Cancel ❌", Buttons::ListButtons(ListCommandButtons::ListSettings)))
                    .into(),
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::TargetOptions)))
                    .append_to_row(0, button(Buttons::ListButtons(ListCommandButtons::ResultLimit)))
                    .append_to_row(1, button(Buttons::ListButtons(ListCommandButtons::SortingOptions)))
                    .append_to_row(2, button(Buttons::ListButtons(ListCommandButtons::Execute)))
                    .append_to_row(2, inline_button("Cancel ❌", Buttons::ListButtons(ListCommandButtons::ListSettings)))
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


