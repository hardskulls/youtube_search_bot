
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]


use axum::Router;
use axum::routing::any;

use internal::auth_server::{handle_auth_code, serve_all};
use app::formatting::format_logs;


#[tokio::main]
async fn main() -> eyre::Result<()>
{
    env_logger::Builder::from_default_env().format(format_logs).try_init()?;

    log::info!(" [:: LOG ::]    ( ⚙ <| Building 'auth_server'... |> ⚙ )");
    let router : Router =
        Router::new()
            .route("/google_callback_auth_code", any(handle_auth_code))
            .route("/", any(serve_all));
    
    let port = env!("PORT").parse::<u16>()?;
    axum::Server::try_bind(&([0, 0, 0, 0], port).into())?
        .serve(router.into_make_service())
        .await?;
    
    log::info!(" [:: LOG ::]    ( ❌ <| 'auth_server' finished |> ❌ )");

    Ok(())
}


