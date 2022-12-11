use serde::Serialize;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind};

use crate::mods::inline_keyboards::traits::ButtonText;
use crate::mods::inline_keyboards::types::KeyBoard;

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


