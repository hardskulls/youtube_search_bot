
pub mod formatting
{
    use std::io::Write;
    use env_logger::fmt::Formatter;
    use log::Record;

    pub fn format_logs(buf : &mut Formatter, record : &Record) -> std::io::Result<()>
    {

        let (file, line) = (record.file().unwrap_or("unknown file"), record.line().unwrap_or(0));
        let time = chrono::Local::now().format("Date %Y.%m.%d | Time %H:%M:%S");
        let (level, args) = (record.level(), record.args());

        writeln!(buf, " [:: LOG ::]:[ {} ] : [Location '{}:{}'] : [{}] :: ( {} ) ", level, file, line, time, args)
    }
}

pub mod net
{
    use std::convert::Infallible;
    use std::future::Future;
    use std::net::SocketAddr;
    use axum::routing::any;
    use teloxide::requests::Requester;
    use teloxide::stop::StopToken;
    use teloxide::update_listeners::{UpdateListener, webhooks};
    
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
}


