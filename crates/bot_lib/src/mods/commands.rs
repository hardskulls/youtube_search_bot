use teloxide::
{
    Bot,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{InlineKeyboardMarkup, Me, Message},
    utils::command::BotCommands
};

use crate::mods::dialogue::types::{DialogueData, ListConfigData, MessageWithKB, SearchConfigData, State, TheDialogue};
use crate::mods::inline_keyboards::funcs::{CreateKB, KeyboardText};
use crate::mods::inline_keyboards::types::SearchCommandKB::SearchConfig;

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
}

pub async fn handle_commands(bot: Bot, msg: Message, dialogue: TheDialogue, cmd: Command) -> eyre::Result<()>
{
    let (message_text, opt_keyboard, opt_dialogue_data): (_, Option<InlineKeyboardMarkup>, Option<DialogueData>) =
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

#[inline]
pub fn is_other_command<B: BotCommands>(msg: Message, me: Me) -> bool
{
    let bot_name = me.user.username.expect("Bots must have a username");
    if let Some(text) = msg.text()
    { matches!(text.chars().next(), Some('/')) && B::parse(text, bot_name.as_str()).is_err() }
    else
    { false }
}

#[inline]
pub async fn handle_unknown_command(bot: Bot, msg: Message) -> eyre::Result<()>
{
    bot.send_message(msg.chat.id, "Unknown command 🤷‍♀️").await?;
    Ok(())
}


