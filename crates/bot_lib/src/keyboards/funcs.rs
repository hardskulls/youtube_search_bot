use serde::Serialize;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind};
use error_traits::WrapInOk;
use crate::keyboards::traits::ButtonText;
use crate::keyboards::types::Buttons;
use crate::StdResult;

/// Constructs `inline keyboard` button inner data.
fn callback_data<D: Serialize>(callback_data: D) -> StdResult<InlineKeyboardButtonKind, serde_json::error::Error>
{
    let unique_string_identifier: String = serde_json::to_string(&callback_data)?;
    InlineKeyboardButtonKind::CallbackData(unique_string_identifier).in_ok()
}

/// Constructs `inline keyboard` button from anything.
pub(crate) fn inline_button<S: Into<String>>(text: S, data: Buttons) -> StdResult<InlineKeyboardButton, serde_json::error::Error>
{
    InlineKeyboardButton::new(text.into(), callback_data(data)?).in_ok()
}

/// Constructs `inline keyboard` button from `KeyBoard`.
pub(crate) fn button(kb: Buttons) -> StdResult<InlineKeyboardButton, serde_json::error::Error>
{
    InlineKeyboardButton::new(kb.button_text(), callback_data(kb)?).in_ok()
}


