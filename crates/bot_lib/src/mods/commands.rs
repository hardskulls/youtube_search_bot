use teloxide::
{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{InlineKeyboardMarkup, Message},
    utils::command::BotCommands
};
use crate::mods::commands::funcs::log_out;

use crate::mods::dialogue::types::{DialogueData, ListConfigData, MessageWithKB, SearchConfigData, State, TheDialogue};
use crate::mods::errors::MergeOkErr;
use crate::mods::inline_keyboards::traits::{CreateKB, KeyboardText};
use crate::mods::inline_keyboards::types::SearchCommandKB::SearchConfig;

pub(crate) mod funcs;

/// List of commands available in the bot.
#[derive(Clone, BotCommands)]
#[command(rename_rule = "snake_case", description = "These commands are available")]
pub enum Command
{
    #[command(description = "Start Bot")]
    Start,
    #[command(description = "Show Bot Status")]
    Info,
    #[command(description = "Search Something")]
    Search,
    #[command(description = "List Something")]
    List,
    #[command(description = "Delete User Data")]
    LogOut,
}

/// Main command handler.
pub async fn handle_commands(bot: Bot, msg: Message, dialogue: TheDialogue, cmd: Command) -> eyre::Result<()>
{
    let (message_text, opt_keyboard, opt_dialogue_data): (String, Option<InlineKeyboardMarkup>, Option<DialogueData>) =
        match cmd
        {
            Command::Start => ("Bot started, send something ⌨ \n Use /search or /list commands 🚀".to_owned(), None, None),
            Command::Info => ("This Bot lets you search stuff on your YouTube channel 🔎 \n Use /search or /list commands 🚀".to_owned(), None, None),
            Command::Search =>
                {
                    let state = State::SearchCommandActive(SearchConfigData { ..Default::default() });
                    (SearchConfig.keyboard_text(), SearchConfig.create_kb(), DialogueData { state, ..Default::default() }.into())
                }
            Command::List =>
                {
                    let state = State::ListCommandActive(ListConfigData { ..Default::default() });
                    (SearchConfig.keyboard_text(), SearchConfig.create_kb(), DialogueData { state, ..Default::default() }.into())
                }
            Command::LogOut =>
                {
                    let user_id = msg.from().ok_or(eyre::eyre!("No User Id"))?.id.to_string();
                    let redis_url = std::env::var("REDIS_URL")?;
                    log_out(&user_id, &redis_url).await.merge_ok_err()
                }
        };
    let message_to_send = bot.send_message(msg.chat.id, &message_text);
    if let (Some(d_data), Some(kb)) = (opt_dialogue_data, opt_keyboard)
    {
        let sent_msg = message_to_send.reply_markup(kb).await?;
        let new_dialogue_data = DialogueData { message_with_kb: MessageWithKB { opt_message: sent_msg.into() }, ..d_data };
        dialogue.update(new_dialogue_data).await.map_err(|e| eyre::anyhow!(e))?;
    }
    else
    { message_to_send.await?; }
    Ok(())
}


