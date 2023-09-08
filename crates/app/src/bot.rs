
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

use teloxide::dispatching::{DpHandlerDescription, HandlerExt, UpdateFilterExt};
use teloxide::dispatching::dialogue::{ErasedStorage, InMemStorage, RedisStorage, serializer, Storage, TraceStorage};
use teloxide::dptree;
use teloxide::prelude::{DependencyMap, Handler, Update};

use internal::commands::Command;
use internal::dialogue::DialogueData;
use internal::handlers::{handle_callback, handle_commands, handle_text, handle_unknown_command, is_other_command};


pub async fn schema_and_storage<S>(build_storage: impl Future<Output = Arc<S>>)
    -> (Handler<'static, DependencyMap, Result<(), ()>, DpHandlerDescription>, Arc<S>)
    where
        S: Storage<DialogueData> + Send + Sync + ?Sized + 'static,
        <S as Storage<DialogueData>>::Error: Debug + Send
{
    let storage = build_storage.await;
    
    let registered_commands = dptree::entry().filter_command::<Command>().endpoint(handle_commands);
    let unknown_commands = dptree::filter(is_other_command::<Command>).endpoint(handle_unknown_command);
    let text = dptree::case![DialogueData { state, last_callback, message_with_kb }].endpoint(handle_text);
    
    let message_handler =
        Update::filter_message()
            .branch(registered_commands)
            .branch(unknown_commands)
            .branch(text);
    let callback_handler =
        Update::filter_callback_query()
            .endpoint(handle_callback);
    let main_handler =
        dptree::entry()
            .enter_dialogue::<Update, S, DialogueData>()
            .branch(message_handler)
            .branch(callback_handler);
    (main_handler, storage)
}

pub type WrappedStorage = Arc<ErasedStorage<DialogueData>>;

pub enum BotInternalDataStorage<'a>
{
    Redis(&'a str)
}

impl<'a> BotInternalDataStorage<'a>
{
    pub async fn build(&self) -> eyre::Result<WrappedStorage>
    {
        let mk_redis =
            |redis_storage|
                {
                    log::info!("[ LOG ] 💾 <| Using `RedisStorage` to store dialogue state. |> ");
                    TraceStorage::new(redis_storage).erase()
                };
        match *self
        {
            BotInternalDataStorage::Redis(url) =>
                RedisStorage::open(url, serializer::Json)
                    .await
                    .map(mk_redis)
                    .map_err(<_>::into)
                
        }
    }
}

pub async fn build_storage(primary_storage: Option<BotInternalDataStorage<'_>>) -> WrappedStorage
{
    let default_s =
        ||
            {
                log::info!("[ LOG ] 💾(✅) <| Using `InMemStorage` to store dialogue state. |> ");
                TraceStorage::new(InMemStorage::<DialogueData>::new()).erase()
            };
    if let Some(s) = primary_storage
    {
        if let Ok(redis_storage) = s.build().await
        {
            log::info!("[ LOG ] 💾 <| Using `RedisStorage` to store dialogue state. |> ");
            redis_storage
        }
        else
        {
            log::info!("[ LOG ] 💾(❌) <| Failed to get `RedisStorage` storage and `SqliteStorage` storage. |> ");
            default_s()
        }
    }
    else
    { default_s() }
}


