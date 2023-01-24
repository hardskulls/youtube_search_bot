use teloxide::Bot;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::Requester;
use teloxide::types::{Me, Message, ParseMode};
use teloxide::utils::command::BotCommands;

use error_traits::MergeOkErr;

use crate::commands::funcs::{info, log_out};
use crate::dialogue::types::{DialogueData, ListCommandSettings, MessageTriplet, MessageWithKB, SearchCommandSettings, State, TheDialogue};
use crate::keyboards::traits::{CreateKB, KeyboardText};
use crate::keyboards::types::ListCommandButtons::ListSettings;
use crate::keyboards::types::SearchCommandButtons::SearchSettings;

pub mod funcs;

/// List of commands available in the bot.
#[derive(Clone, BotCommands)]
#[command(rename_rule = "snake_case", description = "These commands are available")]
pub enum Command
{
    #[command(description = "Start Bot")]
    Start,
    #[command(description = "Info")]
    Info,
    #[command(description = "Search Something")]
    Search,
    #[command(description = "List Something")]
    List,
    #[command(description = "Log Out")]
    LogOut,
}

/// Main command handler.
pub async fn handle_commands(bot: Bot, msg: Message, dialogue: TheDialogue, cmd: Command) -> eyre::Result<()>
{
    let (message_text, opt_keyboard, opt_dialogue_data): MessageTriplet =
        match cmd
        {
            Command::Start => ("Bot started, send something ⌨ \n Use /search or /list commands 🚀".to_owned(), None, None),
            Command::Info => info(&dialogue).await.merge_ok_err(),
            Command::Search =>
                {
                    let state = State::SearchCommandActive(SearchCommandSettings::default());
                    (SearchSettings.kb_text(), SearchSettings.create_kb(), DialogueData { state, ..Default::default() }.into())
                }
            Command::List =>
                {
                    let state = State::ListCommandActive(ListCommandSettings::default());
                    (ListSettings.kb_text(), ListSettings.create_kb(), DialogueData { state, ..Default::default() }.into())
                }
            Command::LogOut =>
                {
                    let user_id = msg.from().ok_or(eyre::eyre!("No User Id"))?.id.to_string();
                    let redis_url = env!("REDIS_URL");
                    log_out(&user_id, redis_url).await.merge_ok_err()
                }
        };
    let message_to_send = bot.send_message(msg.chat.id, &message_text).parse_mode(ParseMode::Html);
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

#[inline]
pub fn is_other_command<B: BotCommands>(msg: Message, me: Me) -> bool
{
    let bot_name = me.username();
    if let Some(text) = msg.text()
    { matches!(text.chars().next(), Some('/')) && B::parse(text, bot_name).is_err() }
    else
    { false }
}

/// Tell user that an unknown command was received.
#[inline]
pub async fn handle_unknown_command(bot: Bot, msg: Message) -> eyre::Result<()>
{
    bot.send_message(msg.chat.id, "Unknown command 🤷‍♀️").await?;
    Ok(())
}


