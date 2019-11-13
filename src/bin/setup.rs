// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(clippy::pedantic)]

use elefren::scopes::{Scopes, Write};
use elefren::Registration;
use failure::{Fallible, ResultExt, SyncFailure};
use std::env;
use std::io::{self, Write as _};

fn main() -> Fallible<()> {
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

    println!("NEWBOT_BASE={}", mastodon.data.base);
    println!("NEWBOT_TOKEN={}", mastodon.data.token);
    Ok(())
}
