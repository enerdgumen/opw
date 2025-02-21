use anyhow::bail;
use anyhow::Result;
use clap::Args;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;
use std::process::Stdio;

use crate::daemon::Response;
use crate::socket;

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^op://[^/]+/[^/]+(?:/[^/]+)?/[^/]+$").unwrap());

#[derive(Args)]
pub struct RunOptions {
    args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct EnvVars(HashMap<String, String>);

impl EnvVars {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self(dotenvy::vars().collect())
    }

    pub fn injectable_1password_references(self) -> Self {
        EnvVars(
            self.0
                .into_iter()
                .filter(|(_, value)| RE.is_match(value))
                .map(|(key, value)| (key, format!("{{{{ {value} }}}}")))
                .collect(),
        )
    }
}

pub fn run_command(options: RunOptions) -> Result<()> {
    let vars = EnvVars::from_env().injectable_1password_references();
    let message = serde_json::to_string(&vars)?;
    let response = socket::send_request(message)?;
    let response: Response = serde_json::from_str(&response)?;

    if !response.stderr.is_empty() {
        bail!("{}", response.stderr);
    }

    if !response.stdout.is_empty() {
        let envs: HashMap<String, String> = serde_json::from_str(&response.stdout)?;

        let mut args = options.args.iter();
        if let Some(program) = args.next() {
            Command::new(program)
                .args(args)
                .envs(&envs)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
                .wait()?;
        }
    }

    Ok(())
}
