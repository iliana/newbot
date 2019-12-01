// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(clippy::pedantic)]

use failure::Fallible;
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
struct Client {
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
    client: &'a Client,
    code: &'a str,
    grant_type: &'static str,
    redirect_uri: &'static str,
}

#[derive(Debug, Deserialize)]
struct Token {
    access_token: String,
}

fn main() -> Fallible<()> {
    let base = env::var("NEWBOT_BASE")?;
    let client: Client = minreq::post(format!("{}/api/v1/apps", base))
        .with_json(&App {
            client_name: "newbot",
            redirect_uris: REDIRECT_URI,
            scopes: SCOPE,
            website: "https://github.com/iliana/newbot",
        })?
        .send()?
        .json()?;
    let url = format!(
        "{}/oauth/authorize?{}",
        base,
        serde_urlencoded::to_string(&Authorization {
            client_id: &client.client_id,
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
            client: &client,
            code: code.trim(),
            grant_type: "authorization_code",
            redirect_uri: REDIRECT_URI,
        })?
    );
    let token: Token = minreq::post(url).send()?.json()?;

    println!("NEWBOT_BASE={}", base);
    println!("NEWBOT_TOKEN={}", token.access_token);
    Ok(())
}
