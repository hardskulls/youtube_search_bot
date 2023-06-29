
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;

use axum::routing::any;
use teloxide::requests::Requester;
use teloxide::stop::StopToken;
use teloxide::update_listeners::{UpdateListener, webhooks};

use internal::auth_server::handle_auth_code;

async fn spawn_background_server
(
    addr : SocketAddr,
    router : axum::Router,
    stop_flag : impl Future<Output = ()> + Send + 'static,
    stop_token : StopToken
)
{
    let serve =
        async move
            {
                axum::Server::bind(&addr)
                    .serve(router.into_make_service())
                    .with_graceful_shutdown(stop_flag)
                    .await
                    .map_err(|err| { stop_token.stop(); err })
                    .expect("Axum server error");
            };
    tokio::spawn(serve);
}

pub async fn webhook_with_custom_server<R>(bot : R, options : webhooks::Options)
    -> Result<impl UpdateListener<Err = Infallible>, R::Err>
    where
        R : Requester + Send + 'static,
        R::DeleteWebhook : Send,
{
    let webhooks::Options { address, .. } = options;
    let url = options.url.clone();

    let (mut update_listener, stop_flag, app) = webhooks::axum_to_router(bot, options).await?;
    let stop_token = update_listener.stop_token();

    let app = app.route(url.path(), any(serve_all));
    let app = app.route("/google_callback_auth_code", any(handle_auth_code));

    spawn_background_server(address, app, stop_flag, stop_token).await;

    Ok(update_listener)
}

async fn serve_all(req : axum::http::Request<axum::body::Body>) -> &'static str
{
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] started [ OK ] )");

    let (parts, body) = req.into_parts();
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] 'parts' is [| '{:#?}' |] )", &parts);
    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] 'body' is [| '{:#?}' |] )", &body);

    log::info!(" [:: LOG ::]    ( @:[fn::serve_all] finished [ OK ] )");
    "up 🤖✔"
}


