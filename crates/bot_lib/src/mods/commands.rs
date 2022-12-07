use teloxide::
{
    Bot,
    requests::Requester,
    types::{Me, Message},
    types::InlineKeyboardMarkup,
    utils::command::BotCommands
};

use SearchCommandKB::SearchConfig;

use crate::mods::dialogue::types::{State, TheDialogue};
use crate::mods::dialogue::helpers::update_state_and_send_message;
use crate::mods::dialogue::types::{DialogueData, ListConfigData, SearchConfigData};
use crate::mods::inline_keyboards::funcs::{CreateKB, KeyboardText};
use crate::mods::inline_keyboards::types::SearchCommandKB;

#[derive(Clone, BotCommands)]
#[command(rename_rule = "snake_case", description = "These commands are available")]
pub enum Command
{
    #[command(description = "Start Bot")]
    Start,
    #[command(description = "Show Game Status")]
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
            Command::Start => ("Bot started, send something ⌨".to_owned(), None, None),
            Command::Info => ("This Bot lets you search stuff on your YouTube channel 🔎".to_owned(), None, None),
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
    update_state_and_send_message(Some(dialogue), opt_dialogue_data, opt_keyboard, bot, msg.chat.id, message_text).await?;
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


