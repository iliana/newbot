// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(clippy::pedantic)]

mod emoji;

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::env;

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

macro_rules! env_func {
    ($f:ident, $var:ident) => {
        fn $f() -> Result<&'static str> {
            lazy_static! {
                static ref VAR: std::result::Result<String, env::VarError> =
                    env::var(stringify!($var));
            }

            VAR.as_ref()
                .map(String::as_str)
                .map_err(Clone::clone)
                .context(format!("failed to get {}", stringify!($var)))
        }
    };
}

env_func!(base, NEWBOT_BASE);
env_func!(token, NEWBOT_TOKEN);

#[derive(Debug, Serialize)]
struct NewStatus {
    status: String,
    visibility: Visibility,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum Visibility {
    Direct,
    Unlisted,
}

async fn draft_toot() -> Result<NewStatus> {
    #[derive(Debug, Deserialize)]
    struct Emoji {
        shortcode: String,
    }

    let emojos: Vec<Emoji> = CLIENT
        .get(&format!("{}/api/v1/custom_emojis", base()?))
        .send()
        .await
        .context("failed to send api/v1/custom_emojis")?
        .json()
        .await
        .context("failed to deserialize api/v1/custom_emojis")?;

    let n = thread_rng().gen_range(0, emoji::EMOJI_SETS.len() + emojos.len());
    let emoji = if n < emoji::EMOJI_SETS.len() {
        let set = emoji::EMOJI_SETS[n];
        Cow::from(set[thread_rng().gen_range(0, set.len())])
    } else {
        let n = n - emoji::EMOJI_SETS.len();
        Cow::from(format!(":{}:", emojos[n].shortcode))
    };

    Ok(NewStatus {
        status: format!(":newl:\u{200b}{}\u{200b}:newr:", emoji),
        visibility: if env::var_os("NEWBOT_LIVE_MODE").is_some() {
            Visibility::Unlisted
        } else {
            Visibility::Direct
        },
    })
}

async fn send_toot() -> Result<()> {
    CLIENT
        .post(&format!("{}/api/v1/statuses", base()?))
        .bearer_auth(token()?)
        .json(&draft_toot().await?)
        .send()
        .await
        .context("failed to send api/v1/statuses")?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Empty {}

#[lambda::lambda]
#[tokio::main]
async fn main(_: Empty) -> Result<()> {
    send_toot().await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_draft_toot() {
        std::env::set_var("NEWBOT_BASE", "https://cybre.space");
        let status = super::draft_toot().await.unwrap();
        assert!(status.status.len() > 18);
    }
}
