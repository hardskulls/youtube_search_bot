use serde::Serialize;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup};

use crate::mods::inline_keyboards::types::{ListCommandKB, SearchMode, SearchCommandKB, SearchTarget, SortMode, ListFilter, ListTarget, KeyBoard};


// fn keyboard_new_row(n: usize) -> Vec<InlineKeyboardButton>
// {
//     let text = "text was not provided";
//     let data = InlineKeyboardButtonKind::CallbackData("data was not provided".into());
//     vec![InlineKeyboardButton::new(text, data); n]
// }

fn callback_data<D: Serialize>(callback_data: D) -> InlineKeyboardButtonKind
{
    let unique_string_identifier: String = serde_json::to_string(&callback_data).expect("Failed to serialize");
    InlineKeyboardButtonKind::CallbackData(unique_string_identifier)
}

pub(crate) fn inline_button<S: Into<String>>(text: S, data: KeyBoard) -> InlineKeyboardButton
{
    InlineKeyboardButton::new(text.into(), callback_data(data))
}

pub(crate) fn button(kb: KeyBoard) -> InlineKeyboardButton
{
    InlineKeyboardButton::new(kb.button_text(), callback_data(kb))
}

/// Creates `InlineKeyboardMarkup`.
pub trait CreateKB
{ fn create_kb(&self) -> Option<InlineKeyboardMarkup>; }

impl CreateKB for SearchCommandKB
{
    fn create_kb(&self) -> Option<InlineKeyboardMarkup>
    {
        match *self
        {
            SearchCommandKB::ResultLimit => None,
            SearchCommandKB::Target =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(KeyBoard::SearchCommand(SearchCommandKB::TargetContent(SearchTarget::Subscription))))
                    .append_to_row(0, button(KeyBoard::SearchCommand(SearchCommandKB::TargetContent(SearchTarget::PlayList))))
                    .append_to_row(1, inline_button("Cancel ❌", KeyBoard::SearchCommand(SearchCommandKB::SearchConfig)))
                    .into(),
            SearchCommandKB::SearchBy =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(KeyBoard::SearchCommand(SearchCommandKB::SearchByContent(SearchMode::Title))))
                    .append_to_row(0, button(KeyBoard::SearchCommand(SearchCommandKB::SearchByContent(SearchMode::Description))))
                    .append_to_row(1, inline_button("Cancel ❌", KeyBoard::SearchCommand(SearchCommandKB::SearchConfig)))
                    .into(),
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(KeyBoard::SearchCommand(SearchCommandKB::ResultLimit)))
                    .append_to_row(0, button(KeyBoard::SearchCommand(SearchCommandKB::Target)))
                    .append_to_row(0, button(KeyBoard::SearchCommand(SearchCommandKB::SearchBy)))
                    .append_to_row(1, inline_button("Cancel ❌", KeyBoard::SearchCommand(SearchCommandKB::SearchConfig)))
                    .into(),
        }
    }
}

impl CreateKB for ListCommandKB
{
    fn create_kb(&self) -> Option<InlineKeyboardMarkup>
    {
        match *self
        {
            ListCommandKB::ResultLimit => None,
            ListCommandKB::Target =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::TargetContent(ListTarget::Subscription))))
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::TargetContent(ListTarget::PlayList))))
                    .append_to_row(1, inline_button("Cancel ❌", KeyBoard::ListCommand(ListCommandKB::ListConfig)))
                    .into(),
            ListCommandKB::Filter =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::FilterContent(ListFilter::Hey))))
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::FilterContent(ListFilter::Ho))))
                    .append_to_row(1, inline_button("Cancel ❌", KeyBoard::ListCommand(ListCommandKB::ListConfig)))
                    .into(),
            ListCommandKB::SortBy =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::SortContent(SortMode::Date))))
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::SortContent(SortMode::Alphabet))))
                    .append_to_row(1, inline_button("Cancel ❌", KeyBoard::ListCommand(ListCommandKB::ListConfig)))
                    .into(),
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::ResultLimit)))
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::Target)))
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::SortBy)))
                    .append_to_row(0, button(KeyBoard::ListCommand(ListCommandKB::Filter)))
                    .append_to_row(1, inline_button("Cancel ❌", KeyBoard::ListCommand(ListCommandKB::ListConfig)))
                    .into()
        }
    }
}


/// Text to show in message with inline keyboard.
pub trait KeyboardText
{ fn keyboard_text(&self) -> String; }

impl KeyboardText for SearchCommandKB
{
    fn keyboard_text(&self) -> String
    {
        match *self
        {
            SearchCommandKB::ResultLimit => "Choose result limit 📇".to_string(),
            SearchCommandKB::Target => "Choose what you want to search 🔎".to_string(),
            SearchCommandKB::SearchBy => "Choose how you want to search 📋".to_string(),
            _ => "Set up your search query ⚙".to_string(),
        }
    }
}

impl KeyboardText for ListCommandKB
{
    fn keyboard_text(&self) -> String
    {
        match *self
        {
            ListCommandKB::ResultLimit => "Choose result limit 📇".to_string(),
            ListCommandKB::Target => "Choose what you want to search 🔎".to_string(),
            ListCommandKB::SortBy => "Choose result sorting 📋".to_string(),
            ListCommandKB::Filter => "Choose result filtering 📊".to_string(),
            _ => "Set up your list query ⚙".to_string(),
        }
    }
}


pub trait ButtonText
{ fn button_text(&self) -> String; }


impl ButtonText for KeyBoard
{
    fn button_text(&self) -> String
    { self.to_string() }
}


impl ButtonText for SearchCommandKB
{
    fn button_text(&self) -> String
    { self.to_string() }
}

impl ButtonText for SearchMode
{
    fn button_text(&self) -> String
    { self.to_string() }
}

impl ButtonText for SearchTarget
{
    fn button_text(&self) -> String
    { self.to_string() }
}


impl ButtonText for ListCommandKB
{
    fn button_text(&self) -> String
    { self.to_string() }
}

impl ButtonText for SortMode
{
    fn button_text(&self) -> String
    { self.to_string() }
}

impl ButtonText for ListFilter
{
    fn button_text(&self) -> String
    { self.to_string() }
}

impl ButtonText for ListTarget
{
    fn button_text(&self) -> String
    { self.to_string() }
}


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


