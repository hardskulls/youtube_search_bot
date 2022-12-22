use axum::Router;
use axum::routing::any;
use bot_lib::net::funcs::{handle_auth_code, serve_all};

#[tokio::main]
async fn main() -> eyre::Result<()>
{
    // !! All `logs::info!` work only after this line + env variable `RUST_LOG` set to `INFO`. !!
    simple_logger::init_with_env().or_else(|_| simple_logger::init_with_level(log::Level::Info))?;
    
    log::info!(" [:: LOG ::] ... : ( ⚙ <| Building 'auth_server'... |> ⚙ )");
    // build our application with a single route
    let router: Router =
        Router::new()
            .route("/google_callback_auth_code", any(handle_auth_code))
            .route("/", any(serve_all));
    
    // run it with hyper on localhost:8443
    let port = std::env::var("PORT")?.parse::<u16>()?;
    axum::Server::try_bind(&([0, 0, 0, 0], port).into())?
        .serve(router.into_make_service())
        .await?;
    
    log::info!(" [:: LOG ::] ... : ( ❌ <| 'auth_server' finished |> ❌ )");
    Ok(())
}


