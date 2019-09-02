// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(clippy::pedantic)]

mod emoji;

use elefren::scopes::{Scopes, Write};
use elefren::status_builder::{StatusBuilder, Visibility};
use elefren::{Data, Mastodon, MastodonClient, Registration};
use failure::{Compat, Error, Fallible, ResultExt, SyncFailure};
use lambda_runtime::{lambda, Context};
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::borrow::Cow;
use std::env;
use std::io::{self, Write as _};

static ZWS: char = '\u{200b}';

fn setup() -> Fallible<()> {
    let base = env::var("NEWBOT_BASE").context("env var NEWBOT_BASE not set")?;
    let registration = Registration::new(base)
        .client_name("newbot")
        .scopes(Scopes::write(Write::Statuses))
        .build()
        .map_err(SyncFailure::new)?;
    let url = registration.authorize_url().map_err(SyncFailure::new)?;

    eprintln!("Get an auth token from:");
    eprintln!("  {}", url);
    eprint!("Auth token: ");
    io::stdout().flush().context("failed to flush stdout")?;
    let mut auth_token = String::new();
    io::stdin()
        .read_line(&mut auth_token)
        .context("failed to read auth token")?;
    let mastodon = registration
        .complete(auth_token.trim())
        .map_err(SyncFailure::new)?;

    for (k, v) in &[
        ("BASE", mastodon.data.base),
        ("CLIENT_ID", mastodon.data.client_id),
        ("CLIENT_SECRET", mastodon.data.client_secret),
        ("REDIRECT", mastodon.data.redirect),
        ("TOKEN", mastodon.data.token),
    ] {
        println!("NEWBOT_{}={}", k, v);
    }
    Ok(())
}

fn send_toot() -> Fallible<()> {
    let mastodon = Mastodon::from(envy::prefixed("NEWBOT_").from_env::<Data>()?);
    let emojos = mastodon
        .get_emojis()
        .map_err(SyncFailure::new)?
        .items_iter()
        .filter(|emoji| emoji.shortcode != "newl" && emoji.shortcode != "newr")
        .collect::<Vec<_>>();

    let n = thread_rng().gen_range(0, emoji::EMOJI_SETS.len() + emojos.len());
    let emoji = if n < emoji::EMOJI_SETS.len() {
        let set = emoji::EMOJI_SETS[n];
        Cow::from(if set.len() == 1 {
            set[0]
        } else {
            set[thread_rng().gen_range(0, set.len())]
        })
    } else {
        let n = n - emoji::EMOJI_SETS.len();
        Cow::from(format!(":{}:", emojos[n].shortcode))
    };

    let status = StatusBuilder::new()
        .status(format!(":newl:{}{}{}:newr:", ZWS, emoji, ZWS))
        .visibility(if env::var_os("NEWBOT_LIVE_MODE").is_some() {
            Visibility::Unlisted
        } else {
            Visibility::Direct
        })
        .build()
        .map_err(SyncFailure::new)?;
    mastodon.new_status(status).map_err(SyncFailure::new)?;
    Ok(())
}

#[derive(Deserialize)]
struct EmptyEvent {}

fn handler(_: EmptyEvent, _: Context) -> Result<(), Compat<Error>> {
    send_toot().map_err(Error::compat)
}

fn main() -> Fallible<()> {
    if env::var_os("AWS_LAMBDA_RUNTIME_API").is_some() {
        lambda!(handler);
        Ok(())
    } else {
        dotenv::dotenv().ok();
        if env::var_os("NEWBOT_SETUP_MODE").is_some() {
            setup()
        } else {
            send_toot()
        }
    }
}
