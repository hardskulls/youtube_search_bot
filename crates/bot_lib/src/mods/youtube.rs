use std::{path::Path, future::Future, pin::Pin};
use google_youtube3::
{
    oauth2,
    oauth2::ApplicationSecret,
    oauth2::authenticator::Authenticator,
    oauth2::authenticator_delegate::InstalledFlowDelegate,
    YouTube,
    api::SubscriptionListResponse
};
use hyper::Client;

use crate::mods::youtube::types::{REDIRECT_URI, TelegramBotInstalledFlow, YouTubeService};

pub(crate) mod types;
pub(crate) mod funcs;

impl InstalledFlowDelegate for TelegramBotInstalledFlow
{
    fn redirect_uri(&self) -> Option<&str>
    {
        Some(REDIRECT_URI)
    }

    #[allow(unused_variables)]
    fn present_user_url<'a>(&'a self, url: &'a str, need_code: bool)
        -> Pin<Box<dyn Future<Output=Result<String, String>> + Send + 'a>>
    {
        // !! TODO: Is it safe to create pins manually? !!
        Box::pin(present_user_url())
    }
}

async fn present_user_url() -> Result<String, String>
{
    Ok("[:: crates/bot_lib/src/mods/youtube.rs : 'impl InstalledFlowDelegate for TelegramBotInstalledFlow' ::]".to_owned())
}

pub(crate) async fn youtube_service<P: AsRef<Path>>(path: P) -> eyre::Result<YouTubeService>
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

pub(crate) async fn search_subs(youtube_hub: &YouTubeService, max_res: u32, access_token: &str)
    -> google_youtube3::Result<(hyper::Response<hyper::Body>, SubscriptionListResponse)>
{
    youtube_hub.subscriptions()
        .list(&vec!["snippet".into()])
        .max_results(max_res)
        .param("access_token", access_token)
        .mine(true)
        .doit()
        .await
}

#[cfg(test)]
mod tests
{
    use std::any::Any;

    use crate::mods::youtube::funcs::make_auth_url;

    use super::*;

    #[tokio::test]
    async fn test_make_url()
    {
        let secret: ApplicationSecret = oauth2::read_application_secret("client_secret_web_app.json").await.unwrap();
        let (response_type, scope) = ("code".to_owned(), ["https://www.googleapis.com/auth/youtube".to_owned()]);
        let url = make_auth_url(secret.client_id, secret.redirect_uris[0].clone(), response_type, &scope, &[]);
        assert!(matches!(url, Ok(_))) ;
        dbg!(&url) ;
        println!("{}", url.as_ref().unwrap()) ;
    }

    #[tokio::test]
    async fn auth_test()
    {
        let secret: ApplicationSecret = oauth2::read_application_secret("client_secret_web_app.json").await.expect("client_secret_web_app.json");
        let auth: Authenticator<_> =
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


