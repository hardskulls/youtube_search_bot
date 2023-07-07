
use serde::Serialize;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind};
use crate::model::keyboards::traits::ButtonText;
use crate::model::keyboards::types::Buttons;


/// Constructs `inline keyboard` button inner data.
fn callback_data<D: Serialize>(callback_data: D) -> InlineKeyboardButtonKind
{
    let unique_string_identifier: String = serde_json::to_string(&callback_data).unwrap_or_else(|_| "Broken button 🚧".to_owned());
    InlineKeyboardButtonKind::CallbackData(unique_string_identifier)
}

/// Constructs `inline keyboard` button from anything.
pub(crate) fn inline_button<S: Into<String>>(text: S, data: Buttons) -> InlineKeyboardButton
{
    InlineKeyboardButton::new(text.into(), callback_data(data))
}

/// Constructs `inline keyboard` button from `KeyBoard`.
pub(crate) fn button(kb: Buttons) -> InlineKeyboardButton
{
    InlineKeyboardButton::new(kb.button_text(), callback_data(kb))
}


