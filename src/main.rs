// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(clippy::pedantic)]

mod emoji;

use failure::{err_msg, Fallible, ResultExt};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::env;

static ZWS: char = '\u{200b}';

#[derive(Debug, Deserialize)]
struct Emoji {
    shortcode: String,
}

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LambdaInvocationError {
    error_message: String,
    error_type: &'static str,
}

fn draft_toot(base: &str) -> Fallible<NewStatus> {
    let emojos: Vec<Emoji> = minreq::get(format!("{}/api/v1/custom_emojis", base))
        .send()
        .context("failed to request /api/v1/custom_emojis")?
        .json()
        .context("failed to parse /api/v1/custom_emojis")?;

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

    Ok(NewStatus {
        status: format!(":newl:{}{}{}:newr:", ZWS, emoji, ZWS),
        visibility: if env::var_os("NEWBOT_LIVE_MODE").is_some() {
            Visibility::Unlisted
        } else {
            Visibility::Direct
        },
    })
}

fn send_toot(base: &str, token: &str) -> Fallible<()> {
    let status = draft_toot(base)?;
    minreq::post(format!("{}/api/v1/statuses", base))
        .with_header("Authorization", format!("Bearer {}", token))
        .with_json(&status)
        .with_context(|_| format!("failed to serialize {:?}", status))?
        .send()
        .with_context(|_| format!("failed to post {:?} to /api/v1/statuses", status))?;
    Ok(())
}

fn lambda(base: &str, token: &str) -> Fallible<()> {
    let runtime_api =
        env::var("AWS_LAMBDA_RUNTIME_API").context("failed to read AWS_LAMBDA_RUNTIME_API")?;
    let response = minreq::get(format!(
        "http://{}/2018-06-01/runtime/invocation/next",
        runtime_api
    ))
    .send()
    .context("failed to get next invocation")?;
    let request_id = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Lambda-Runtime-Aws-Request-Id"))
        .ok_or_else(|| err_msg("header Lambda-Runtime-Aws-Request-Id missing"))?
        .1;

    match send_toot(&base, &token) {
        Ok(()) => {
            minreq::post(format!(
                "http://{}/2018-06-01/runtime/invocation/{}/response",
                runtime_api, request_id
            ))
            .with_json(&())
            .with_context(|_| format!("failed to serialize {:?}", ()))?
            .send()
            .context("failed to post invocation response")?;
        }
        Err(err) => {
            eprintln!("{:?}", err);
            let body = LambdaInvocationError {
                error_message: err.to_string(),
                error_type: "Failure",
            };
            minreq::post(format!(
                "http://{}/2018-06-01/runtime/invocation/{}/error",
                runtime_api, request_id
            ))
            .with_json(&body)
            .with_context(|_| format!("failed to serialize {:?}", body))?
            .send()
            .context("failed to post invocation error")?;
        }
    };
    Ok(())
}

fn load_env() -> Fallible<(String, String)> {
    let base = env::var("NEWBOT_BASE").context("failed to read NEWBOT_BASE")?;
    let token = env::var("NEWBOT_TOKEN").context("failed to read NEWBOT_TOKEN")?;
    Ok((base, token))
}

fn main() -> Fallible<()> {
    if env::var_os("AWS_LAMBDA_RUNTIME_API").is_some() {
        loop {
            let (base, token) = load_env()?;
            lambda(&base, &token)
                .map_err(|err| eprintln!("{:?}", err))
                .ok();
        }
    } else {
        dotenv::dotenv().ok();
        let (base, token) = load_env()?;
        send_toot(&base, &token)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_draft_toot() {
        let status = super::draft_toot("https://cybre.space").unwrap();
        assert!(status.status.len() > 18);
    }
}
