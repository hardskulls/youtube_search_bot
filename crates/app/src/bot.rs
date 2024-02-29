use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

use teloxide::dispatching::dialogue::{
    serializer, ErasedStorage, InMemStorage, RedisStorage, Storage, TraceStorage,
};
use teloxide::dispatching::{DpHandlerDescription, HandlerExt, UpdateFilterExt};
use teloxide::dptree;
use teloxide::prelude::{DependencyMap, Handler, Update};

use internal::commands::Command;
use internal::dialogue::DialogueData;
use internal::handlers::{
    handle_callback, handle_commands, handle_text, handle_unknown_command, is_other_command,
};

use error_traits::MergeOkErr;

pub async fn schema_and_storage<S>(
    build_storage: impl Future<Output = Arc<S>>,
) -> (
    Handler<'static, DependencyMap, Result<(), ()>, DpHandlerDescription>,
    Arc<S>,
)
where
    S: Storage<DialogueData> + Send + Sync + ?Sized + 'static,
    <S as Storage<DialogueData>>::Error: Debug + Send,
{
    let storage = build_storage.await;

    let registered_commands = dptree::entry()
        .filter_command::<Command>()
        .endpoint(handle_commands);
    let unknown_commands =
        dptree::filter(is_other_command::<Command>).endpoint(handle_unknown_command);
    let text = dptree::case![DialogueData {
        state,
        last_callback,
        message_with_kb
    }]
    .endpoint(handle_text);

    let message_handler = Update::filter_message()
        .branch(registered_commands)
        .branch(unknown_commands)
        .branch(text);
    let callback_handler = Update::filter_callback_query().endpoint(handle_callback);
    let main_handler = dptree::entry()
        .enter_dialogue::<Update, S, DialogueData>()
        .branch(message_handler)
        .branch(callback_handler);
    (main_handler, storage)
}

pub type WrappedStorage = Arc<ErasedStorage<DialogueData>>;

pub enum BotInternalDataStorage<'a> {
    Redis(&'a str),
}

impl<'a> BotInternalDataStorage<'a> {
    pub async fn build(&self) -> eyre::Result<WrappedStorage> {
        match *self {
            BotInternalDataStorage::Redis(url) => RedisStorage::open(url, serializer::Json)
                .await
                .map(|redis_storage| {
                    log::info!("[ LOG ] 💾 <| Using `RedisStorage` to store dialogue state. |> ");
                    TraceStorage::new(redis_storage).erase()
                })
                .map_err(<_>::into),
        }
    }
}

pub async fn build_storage(primary_storage: Option<BotInternalDataStorage<'_>>) -> WrappedStorage {
    build_stor_internal(primary_storage)
        .await
        .ok_or_else(|| {
            log::info!("[ LOG ] 💾(✅) <| Using `InMemStorage` to store dialogue state. |> ");
            TraceStorage::new(InMemStorage::<DialogueData>::new()).erase()
        })
        .merge_ok_err()
}

async fn build_stor_internal(
    prim_stor: Option<BotInternalDataStorage<'_>>,
) -> Option<WrappedStorage> {
    prim_stor
        .map(|s| async move { s.build().await })?
        .await
        .ok()
        .map(|redis_storage| {
            log::info!("[ LOG ] 💾 <| Using `RedisStorage` to store dialogue state. |> ");
            redis_storage
        })
}
