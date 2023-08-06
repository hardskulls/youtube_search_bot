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
    let message_handler =
        Update::filter_message()
            .branch(dptree::entry().filter_command::<Command>().endpoint(handle_commands))
            .branch(dptree::filter(is_other_command::<Command>).endpoint(handle_unknown_command))
            .branch(dptree::case![DialogueData { state, last_callback, message_with_kb }].endpoint(handle_text));
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

pub async fn build_storage() -> Arc<ErasedStorage<DialogueData>>
{
    /*let redis_youtube_access_token_storage = env!("REDIS_BOT_DATA_STORAGE");
    if let Ok(redis_storage) = RedisStorage::open(redis_youtube_access_token_storage, serializer::Json).await
    {
        log::info!("[ LOG ] 💾 <| Using `RedisStorage` to store dialogue state. |> ");
        TraceStorage::new(redis_storage).erase()
    }
    else
    {
        log::info!("[ LOG ] 💾(❌) <| Failed to get `RedisStorage` storage and `SqliteStorage` storage. |> ");
        log::info!("[ LOG ] 💾(✅) <| Using `InMemStorage` to store dialogue state. |> ");
        TraceStorage::new(InMemStorage::<DialogueData>::new()).erase()
    }*/
    log::info!("[ LOG ] 💾(✅) <| Using `InMemStorage` to store dialogue state. |> ");
    TraceStorage::new(InMemStorage::<DialogueData>::new()).erase()
}


