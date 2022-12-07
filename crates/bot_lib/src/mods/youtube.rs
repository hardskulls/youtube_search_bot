use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use google_youtube3::
{
    oauth2::ApplicationSecret,
    oauth2,
    YouTube,
    oauth2::authenticator::Authenticator,
    oauth2::authenticator_delegate::InstalledFlowDelegate
};
use google_youtube3::api::SubscriptionListResponse;
use hyper::{Client};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use redis::Commands;
use teloxide::{Bot, requests::Requester, types::ChatId};
use crate::mods::youtube::helpers::{make_auth_url, query_pairs};
use crate::mods::youtube::types::{MapErrToString, REDIRECT_URI, TelegramBotInstalledFlow};

pub(crate) mod types;
pub(crate) mod helpers;

impl InstalledFlowDelegate for TelegramBotInstalledFlow
{
    fn redirect_uri(&self) -> Option<&str>
    {
        Some(REDIRECT_URI)
    }

    fn present_user_url<'a>(&'a self, url: &'a str, need_code: bool)
        -> Pin<Box<dyn Future<Output=Result<String, String>> + Send + 'a>>
    {
        // !! TODO: Is it safe to create pins manually? !!
        Box::pin(present_user_url(url, need_code))
    }
}

async fn present_user_url(url: &str, need_code: bool) -> Result<String, String>
{
    Ok("[:: crates/bot_lib/src/mods/youtube.rs : 'impl InstalledFlowDelegate for TelegramBotInstalledFlow' ::]".to_owned())
}

/*async fn present_user_url() -> std::result::Result<String, String>
{
    let client = redis::Client::open("redis://127.0.0.1/").map_err_to_str()?;
    let mut con = client.get_connection().map_err_to_str()?;
    let chat_id: i64 = con.get("my_counter").map_err_to_str()?;

    let token = std::env::var("TELOXIDE_TOKEN").map_err_to_str()?;
    let bot = Bot::new(&token);

    let secret = oauth2::read_application_secret("").await.map_err_to_str()?;
    let (RESPONSE_TYPE, scope) = ("code".to_owned(), ["https://www.googleapis.com/auth/youtube".to_owned()]);
    let url = make_auth_url(secret.CLIENT_ID, &secret.redirect_uris, RESPONSE_TYPE, &scope, &[]).map_err_to_str()?;

    let text = format!("Follow this link to log in {}", url);
    bot.send_message(ChatId(chat_id), text).await.map_err_to_str()?;

    // let access_token: String;
    // let clos = || {  };

    // let app: axum::Router = axum::Router::new().route("/google_callback", axum::routing::get(|| async { "Hello, World!" }));
    // axum::Server::bind(&"0.0.0.0:8433".parse().unwrap())
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();

    todo!()
}*/

async fn request(request: axum::http::Request<axum::body::Body>)
{
    let (parts, body) = request.into_parts();
    let params: Vec<_> = query_pairs(&parts.uri).collect();
    let Some(&(_, exchange_code)) = params.iter().find(|&&(key, _val)| key == "code") else { return };
    let Some(&(_, state)) = params.iter().find(|&&(key, _val)| key == "state") else { return };
}

pub(crate) async fn authentificator<T: AsRef<Path>>(path: T) -> std::io::Result<ApplicationSecret>
{
    oauth2::read_application_secret(path).await
}

pub(crate) async fn youtube_service<P: AsRef<Path>>(path: P) -> eyre::Result<YouTube<HttpsConnector<HttpConnector>>>
{
    let secret: ApplicationSecret = oauth2::read_application_secret(path).await?;
    let authenticator: Authenticator<_> =
        oauth2::InstalledFlowAuthenticator::builder(secret, oauth2::InstalledFlowReturnMethod::HTTPRedirect)
            .flow_delegate(Box::new(TelegramBotInstalledFlow))
            .build()
            .await?;
    let connector =
        hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
    let client: Client<_> = hyper::Client::builder().build(connector);
    Ok(YouTube::new(client, authenticator))
}

pub(crate) async fn search_subs(youtube_hub: &YouTube<HttpsConnector<HttpConnector>>, max_res: u32)
    -> google_youtube3::Result<(hyper::Response<hyper::body::Body>, SubscriptionListResponse)>
{
    youtube_hub.subscriptions()
        .list(&vec!["snippet".into()])
        .max_results(max_res)
        .mine(true)
        .doit()
        .await
}

#[cfg(test)]
mod tests
{
    use std::any::Any;
    use super::*;

    #[tokio::test]
    async fn test_make_url()
    {
        let secret: ApplicationSecret = oauth2::read_application_secret("client_secret_web_app.json").await.unwrap();
        let (response_type, scope) = ("code".to_owned(), ["https://www.googleapis.com/auth/youtube".to_owned()]);
        let url = make_auth_url(secret.client_id, secret.redirect_uris[0].clone(), response_type, &scope, &[]);
        assert!(matches!(url, Ok(_))) ;;
        dbg!(&url) ;;
        println!("{}", url.as_ref().unwrap()) ;;
    }

    #[tokio::test]
    async fn auth_test()
    {
        let secret: ApplicationSecret = oauth2::read_application_secret("client_secret_web_app.json").await.expect("client_secret_web_app.json");
        let auth: oauth2::authenticator::Authenticator<_> =
            oauth2::InstalledFlowAuthenticator::builder(secret, oauth2::InstalledFlowReturnMethod::HTTPRedirect)
                .build()
                .await
                .unwrap();
        let connector =
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build();
        let client = hyper::Client::builder().build(connector);
        println!("client finished");
        let hub = YouTube::new(client, auth);
        println!("hub finished");
        // panic!("just in case");
        let result = hub.search().list(&vec!["snippet".to_owned()]);
        let result = result.max_results(10).q("dad of war").add_type("video").order("relevance").doit().await;
        println!("result finished");
        dbg!(result.type_id());

        match result
        {
            Err(e) => println!(" [ DEBUG ] ... Error: {} ... ", e),
            Ok(res) => println!(" [ INFO ] ... Success: {:?} ... ", res),
        }

    }
}


