
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use std::io::Write;

use axum::Router;
use axum::routing::any;

use internal::auth_server::{handle_auth_code, serve_all};


#[tokio::main]
async fn main() -> eyre::Result<()>
{
    env_logger::Builder::new()
        .format
        (
            |buf, record|
                writeln!
                (
                    buf,
                    "{}:{} {} [{}] - {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    record.args()
                )
        )
        .init();
    //simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info))?;
    
    log::info!(" [:: LOG ::]    ( ⚙ <| Building 'auth_server'... |> ⚙ )");
    let router : Router =
        Router::new()
            .route("/google_callback_auth_code", any(handle_auth_code))
            .route("/", any(serve_all));
    
    let port = std::env::var("PORT")?.parse::<u16>()?;
    axum::Server::try_bind(&([0, 0, 0, 0], port).into())?
        .serve(router.into_make_service())
        .await?;
    
    log::info!(" [:: LOG ::]    ( ❌ <| 'auth_server' finished |> ❌ )");
    Ok(())
}


