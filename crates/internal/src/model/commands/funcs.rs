use error_traits::{LogErr, MapErrBy, WrapInRes};

use crate::model::db::{delete_access_token, get_access_token};
use crate::model::dialogue::funcs::get_dialogue_data;
use crate::model::dialogue::types::{
    ListCommandSettings, MessageTriplet, SearchCommandSettings,
    SearchVideosInPlaylistsCommandSettings, State, TheDialogue,
};
use crate::model::net::funcs::build_post_request;
use crate::model::net::types::REVOKE_ACCESS_TOKEN_URL;
use crate::model::utils::{maybe_print, HTMLise};
use crate::model::youtube::types::YouTubeAccessToken;
use crate::StdResult;

fn build_log_out_req(token: YouTubeAccessToken) -> eyre::Result<reqwest::RequestBuilder> {
    log::info!(" [:: LOG ::]     @[fn]:[model::commands::build_log_out_req] :: [Started]");

    let token = token
        .refresh_token
        .as_deref()
        .unwrap_or(token.access_token.as_str());
    let params = &[("token", token)];
    build_post_request(REVOKE_ACCESS_TOKEN_URL, params)
}

/// Revoke `refresh token` and delete token from db.
pub(crate) async fn log_out(user_id: &str, db_url: &str) -> eyre::Result<MessageTriplet> {
    let log_prefix = "@[fn]:[model::commands::log_out] ";
    log::info!("{log_prefix}:: [Started]");

    match get_access_token(user_id, db_url) {
        Ok(token) => {
            let resp = build_log_out_req(token)?.send().await?;
            let revoked_token_successfully = resp.status().is_success();

            log::info!("{log_prefix}revoked_token_successfully is: {revoked_token_successfully:?}");

            log::debug!("{log_prefix} ( resp is: '{:#?}' )", resp);
            log::debug!("{log_prefix} ( body is: '{:#?}' )", resp.text().await);

            delete_access_token(user_id, db_url)?;

            ("Logged out successfully ✅".to_owned(), None, None).in_ok()
        }
        Err(e) => e.in_err(),
    }
}

/// Pretty print config.
fn print_search_config(search_settings: &SearchCommandSettings) -> String {
    let SearchCommandSettings {
        target, search_in, ..
    } = search_settings;
    let SearchCommandSettings {
        result_limit,
        text_to_search,
        ..
    } = search_settings;
    let t = format!(
        "{}{}{}{}",
        maybe_print(format!("\n🎯 {}  =  ", "Target".to_bold()), target, ""),
        maybe_print(
            format!("\n💳 {}  =  ", "Search in".to_bold()),
            search_in,
            ""
        ),
        maybe_print(
            format!("\n🧮 {}  =  ", "Result limit".to_bold()),
            result_limit,
            ""
        ),
        maybe_print(
            format!("\n💬 {}  =  ", "Text to search".to_bold()),
            text_to_search,
            ""
        )
    );
    if t.is_empty() {
        "You've activated 'search command' 🔎".to_owned()
    } else {
        format!("Your search parameters are{t}")
    }
}

/// Pretty print config.
fn print_list_config(list_settings: &ListCommandSettings) -> String {
    let ListCommandSettings {
        target,
        result_limit,
        sorting,
    } = list_settings;
    let t = format!(
        "{}{}{}",
        maybe_print(format!("\n🎯 {}  =  ", "Target".to_bold()), target, ""),
        maybe_print(format!("\n🗃 {}  =  ", "Sorting".to_bold()), sorting, ""),
        maybe_print(
            format!("\n🧮 {}  =  ", "Result limit".to_bold()),
            result_limit,
            ""
        )
    );
    if t.is_empty() {
        "You've activated 'list command' 📃".to_owned()
    } else {
        format!("Your list parameters are{t}")
    }
}

/// Pretty print config.
fn print_search_videos_in_pls_config(
    list_settings: &SearchVideosInPlaylistsCommandSettings,
) -> String {
    let SearchVideosInPlaylistsCommandSettings {
        search_in,
        text_to_search,
        result_limit,
    } = list_settings;
    let t = format!(
        "{}{}{}",
        maybe_print(
            format!("\n💳 {}  =  ", "Search in".to_bold()),
            search_in,
            ""
        ),
        maybe_print(
            format!("\n🧮 {}  =  ", "Result limit".to_bold()),
            result_limit,
            ""
        ),
        maybe_print(
            format!("\n💬 {}  =  ", "Text to search".to_bold()),
            text_to_search,
            ""
        )
    );
    if t.is_empty() {
        "You've activated 'search videos in you playlist' 📃".to_owned()
    } else {
        format!("Your search parameters are{t}")
    }
}

pub(crate) async fn info(dialogue: &TheDialogue) -> StdResult<MessageTriplet, MessageTriplet> {
    log::info!(" [:: LOG ::]     @[fn]:[model::commands::info] :: [Started]");

    let log_prefix = " [:: LOG ::]  :  @fn:[commands::common::info]  ->  error: ";
    let user_error: fn() -> MessageTriplet = || ("Info command failed ❌".to_owned(), None, None);

    let create_msg = |m: &str| (m.to_owned(), None, None);

    let d_data = get_dialogue_data(dialogue)
        .await
        .log_err(log_prefix)
        .map_err_by(user_error)?;
    match d_data.state {
        State::Starting => create_msg("Bot just started 🚀").in_ok(),
        State::SearchCommandActive(search_config) => {
            create_msg(&print_search_config(&search_config)).in_ok()
        }
        State::ListCommandActive(list_config) => {
            create_msg(&print_list_config(&list_config)).in_ok()
        }
        State::SearchVideosInPlaylistsCommandActive(search_vids_in_pls_config) => create_msg(
            &print_search_videos_in_pls_config(&search_vids_in_pls_config),
        )
        .in_ok(),
    }
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests {
    use crate::model::commands::funcs::{
        build_log_out_req, print_list_config, print_search_config,
    };
    use crate::model::dialogue::types::{ListCommandSettings, SearchCommandSettings};
    use crate::model::keyboards::types::Requestable;
    use crate::model::net::funcs::build_post_request;
    use crate::model::net::types::{SubscriptionRequester, REVOKE_ACCESS_TOKEN_URL};
    use crate::model::youtube::types::YouTubeAccessToken;
    use google_youtube3::hyper;
    use std::str::{from_utf8, FromStr};

    #[test]
    fn printable_test() {
        let c = SearchCommandSettings::default();
        assert_eq!(
            print_search_config(&c),
            "You've activated 'search command' 🔎"
        );

        let mut c = ListCommandSettings::default();
        assert_eq!(print_list_config(&c), "You've activated 'list command' 📃");

        c.target = Requestable::Subscription(SubscriptionRequester).into();
        assert_eq!(
            print_list_config(&c),
            "Your list parameters are\n🎯 <b>Target</b>  =  Subscription"
        );
    }

    #[test]
    fn request_build_test() {
        let (access_token, refresh_token) = (
            "acc_tok_653654265432".into(),
            "ref_tok_76876576345".to_owned().into(),
        );
        let (expires_in, scope, token_type) =
            (time::OffsetDateTime::now_utc(), vec![], "Bearer".to_owned());
        let token = YouTubeAccessToken {
            access_token,
            expires_in,
            refresh_token,
            scope,
            token_type,
        };

        let req = build_log_out_req(token.clone()).unwrap().build().unwrap();

        assert_eq!(
            req.headers()
                .get(reqwest::header::HOST)
                .unwrap()
                .to_str()
                .unwrap(),
            "oauth2.googleapis.com"
        );
        assert_eq!(
            req.headers()
                .get(reqwest::header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap(),
            "application/x-www-form-urlencoded"
        );
        assert_eq!(req.url().as_str(), "https://oauth2.googleapis.com/revoke");

        let expected_body =
            reqwest::Body::from(format!("token={t}", t = token.refresh_token.unwrap()));

        assert_eq!(
            req.body().unwrap().as_bytes().unwrap(),
            expected_body.as_bytes().unwrap()
        );
    }

    trait ShortUnwrap<T> {
        fn unwr(self) -> T;
    }

    impl<T> ShortUnwrap<T> for Option<T> {
        fn unwr(self) -> T {
            self.unwrap()
        }
    }

    impl<T, E> ShortUnwrap<T> for Result<T, E>
    where
        E: std::fmt::Debug,
    {
        fn unwr(self) -> T {
            self.unwrap()
        }
    }

    #[test]
    fn test_request_builder() {
        let params: &[(&str, &str)] = &[("token", "HeyHo"), ("answer", "YoHoHo")];
        let expected_query = "token=HeyHo&answer=YoHoHo";
        let body_kv_pairs = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        assert_eq!(body_kv_pairs, expected_query);

        let url = reqwest::Url::parse_with_params(REVOKE_ACCESS_TOKEN_URL, params).unwr();

        assert_eq!(url.domain().unwr(), "oauth2.googleapis.com");
        assert_eq!(url.host_str().unwr(), "oauth2.googleapis.com");

        let req_builder = build_post_request(REVOKE_ACCESS_TOKEN_URL, params).unwr();
        let req = req_builder.build().unwr();
        let body_as_utf8 = from_utf8(req.body().unwr().as_bytes().unwr()).unwr();

        assert_eq!(body_as_utf8, expected_query);

        let url = REVOKE_ACCESS_TOKEN_URL;
        let u = hyper::http::uri::Uri::from_str(url).unwr();

        assert_eq!(u.authority().unwr().as_str(), "oauth2.googleapis.com");
        assert_eq!(u.scheme_str().unwr(), "https");
        assert_eq!(u.to_string(), url);
    }
}
