use serde::Serialize;

use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup};

use crate::mods::inline_keyboards::types::{ListCommandKB, SearchMode, SearchCommandKB, SearchTarget, SortMode, ListFilter, ListTarget};


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

pub fn inline_button<S: Into<String>, D: Serialize>(text: S, data: D) -> InlineKeyboardButton
{
    let callback_string_data = data;
    InlineKeyboardButton { text: text.into(), kind: callback_data(callback_string_data) }
}


pub trait SaveState
{
    // fn save_state(&self) -> Op
}


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
                {
                    let (sub, plist) =
                        (SearchCommandKB::TargetContent(SearchTarget::Subscription), SearchCommandKB::TargetContent(SearchTarget::PlayList));
                    InlineKeyboardMarkup::default()
                        .append_to_row(0, inline_button(sub.button_text(), sub))
                        .append_to_row(0, inline_button(plist.button_text(), plist))
                        .append_to_row(1, inline_button("Cancel ❌", SearchCommandKB::SearchConfig))
                        .into()
                },
            SearchCommandKB::SearchBy =>
                {
                    let (title, descr) =
                        (SearchCommandKB::SearchByContent(SearchMode::Title), SearchCommandKB::SearchByContent(SearchMode::Description));
                    InlineKeyboardMarkup::default()
                        .append_to_row(0, inline_button(title.button_text(), title))
                        .append_to_row(0, inline_button(descr.button_text(), descr))
                        .append_to_row(1, inline_button("Cancel ❌", SearchCommandKB::SearchConfig))
                        .into()
                },
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, inline_button(SearchCommandKB::ResultLimit.button_text(), SearchCommandKB::ResultLimit))
                    .append_to_row(0, inline_button(SearchCommandKB::Target.button_text(), SearchCommandKB::Target))
                    .append_to_row(0, inline_button(SearchCommandKB::SearchBy.button_text(), SearchCommandKB::SearchBy))
                    .append_to_row(1, inline_button("Cancel ❌", SearchCommandKB::SearchConfig))
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
                {
                    let (sub, plist) =
                        (ListCommandKB::TargetContent(ListTarget::Subscription), ListCommandKB::TargetContent(ListTarget::PlayList));
                    InlineKeyboardMarkup::default()
                        .append_to_row(0, inline_button(sub.button_text(), sub))
                        .append_to_row(0, inline_button(plist.button_text(), plist))
                        .append_to_row(1, inline_button("Cancel ❌", ListCommandKB::ListConfig))
                        .into()
                },
            ListCommandKB::Filter =>
                {
                    let (hey, ho) =
                        (ListCommandKB::FilterContent(ListFilter::Hey), ListCommandKB::FilterContent(ListFilter::Ho));
                    InlineKeyboardMarkup::default()
                        .append_to_row(0, inline_button(hey.button_text(), hey))
                        .append_to_row(0, inline_button(ho.button_text(), ho))
                        .append_to_row(1, inline_button("Cancel ❌", ListCommandKB::ListConfig))
                        .into()
                },
            ListCommandKB::SortBy =>
                {
                    let (sort_by_date, sort_by_alphabet) =
                        (ListCommandKB::SortContent(SortMode::Date), ListCommandKB::SortContent(SortMode::Alphabet));
                    InlineKeyboardMarkup::default()
                        .append_to_row(0, inline_button(sort_by_date.button_text(), sort_by_date))
                        .append_to_row(0, inline_button(sort_by_alphabet.button_text(), sort_by_alphabet))
                        .append_to_row(1, inline_button("Cancel ❌", ListCommandKB::ListConfig))
                        .into()
                },
            _ =>
                InlineKeyboardMarkup::default()
                    .append_to_row(0, inline_button(ListCommandKB::ResultLimit.button_text(), ListCommandKB::ResultLimit))
                    .append_to_row(0, inline_button(ListCommandKB::Target.button_text(), ListCommandKB::Target))
                    .append_to_row(0, inline_button(ListCommandKB::SortBy.button_text(), ListCommandKB::SortBy))
                    .append_to_row(0, inline_button(ListCommandKB::Filter.button_text(), ListCommandKB::Filter))
                    .append_to_row(1, inline_button("Cancel ❌", ListCommandKB::ListConfig))
                    .into()
        }
    }
}


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


