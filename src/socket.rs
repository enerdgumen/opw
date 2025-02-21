use anyhow::bail;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

fn socket_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    Path::new(&home).join(".config/op/opw-daemon.sock")
}

pub fn send_request(request: String) -> Result<String> {
    let socket_path = socket_path();
    let mut stream = UnixStream::connect(&socket_path).context("Is the daemon running?")?;

    stream.write_all(request.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    Ok(response)
}

pub fn handle_requests(handler: impl Fn(String) -> Result<String>) -> Result<()> {
    let socket_path = socket_path();

    if UnixStream::connect(&socket_path).is_ok() {
        bail!("Daemon is already running");
    }

    if let Some(socket_folder) = socket_path.parent() {
        fs::create_dir_all(socket_folder).ok();
    }

    std::fs::remove_file(&socket_path).ok();
    let listener = UnixListener::bind(&socket_path)?;
    eprintln!("Daemon listening on {socket_path:?}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_request(stream, &handler) {
                    eprintln!("Failed to handle request: {error:?}");
                }
            }
            Err(error) => {
                eprintln!("Failed to accept connection: {error:?}");
            }
        }
    }

    Ok(())
}

fn handle_request(
    mut stream: UnixStream,
    handler: impl Fn(String) -> Result<String>,
) -> Result<()> {
    let mut request = String::new();
    stream.read_to_string(&mut request)?;

    let response = handler(request)?;
    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
