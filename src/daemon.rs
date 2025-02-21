use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use crate::socket;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub stdout: String,
    pub stderr: String,
}

pub fn execute() -> Result<()> {
    socket::handle_requests(|request| {
        let mut child = Command::new("op")
            .arg("inject")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(request.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let response = Response { stdout, stderr };
        let response = serde_json::to_string(&response)?;
        Ok(response)
    })
}
