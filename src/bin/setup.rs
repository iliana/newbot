// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(clippy::pedantic)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write as _};

const REDIRECT_URI: &str = "urn:ietf:wg:oauth:2.0:oob";
const SCOPE: &str = "write:statuses";

#[derive(Debug, Serialize)]
struct App {
    client_name: &'static str,
    redirect_uris: &'static str,
    scopes: &'static str,
    website: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientPair {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Serialize)]
struct Authorization<'a> {
    client_id: &'a str,
    redirect_uri: &'static str,
    scope: &'static str,
    response_type: &'static str,
}

#[derive(Debug, Serialize)]
struct TokenRequest<'a> {
    #[serde(flatten)]
    client: &'a ClientPair,
    code: &'a str,
    grant_type: &'static str,
    redirect_uri: &'static str,
}

#[derive(Debug, Deserialize)]
struct Token {
    access_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let base = env::var("NEWBOT_BASE")?;
    let client = reqwest::Client::new();
    let pair: ClientPair = client
        .post(&format!("{}/api/v1/apps", base))
        .json(&App {
            client_name: "newbot",
            redirect_uris: REDIRECT_URI,
            scopes: SCOPE,
            website: "https://github.com/iliana/newbot",
        })
        .send()
        .await?
        .json()
        .await?;
    let url = format!(
        "{}/oauth/authorize?{}",
        base,
        serde_urlencoded::to_string(&Authorization {
            client_id: &pair.client_id,
            redirect_uri: REDIRECT_URI,
            scope: SCOPE,
            response_type: "code",
        })?
    );
    eprintln!("Get an auth token from:");
    eprintln!("  {}", url);
    eprint!("Auth token: ");
    io::stdout().flush()?;
    let mut code = String::new();
    io::stdin().read_line(&mut code)?;
    let url = format!(
        "{}/oauth/token?{}",
        base,
        serde_urlencoded::to_string(&TokenRequest {
            client: &pair,
            code: code.trim(),
            grant_type: "authorization_code",
            redirect_uri: REDIRECT_URI,
        })?
    );
    let token: Token = client.post(&url).send().await?.json().await?;

    println!("NEWBOT_BASE={}", base);
    println!("NEWBOT_TOKEN={}", token.access_token);
    Ok(())
}
