use anyhow::Context;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use termcolor::WriteColor;

pub fn log_error<D: std::fmt::Display>(err: D) {
    let stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    let mut stderr = stderr.lock();
    let _ = stderr.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Red)));
    let _ = stderr.write(b"error");
    let _ = stderr.reset();
    let _ = writeln!(stderr, ": {}", err);
}

pub fn log_error_with_timestamp<D: std::fmt::Display>(err: D) {
    let stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    let mut stderr = stderr.lock();
    let _ = write!(stderr, "[{}]", chrono::Local::now());
    let _ = stderr.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Red)));
    let _ = stderr.write(b"error");
    let _ = stderr.reset();
    let _ = writeln!(stderr, ": {}", err);
}

pub fn log_warn<D: std::fmt::Display>(warn: D) {
    let stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    let mut stderr = stderr.lock();
    let _ = stderr.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)));
    let _ = stderr.write(b"warning");
    let _ = stderr.reset();
    let _ = writeln!(stderr, ": {}", warn);
}

#[cfg(windows)]
const GRADLEW: &str = "gradlew.bat";
#[cfg(not(windows))]
const GRADLEW: &str = "gradlew";

fn run_gradlew_wrapper(
    project_dir: PathBuf,
    args: &[&str],
    stdout: Stdio,
) -> anyhow::Result<Vec<u8>> {
    let mut wrapper = project_dir;
    wrapper.push(GRADLEW);
    std::process::Command::new(wrapper)
        .args(args)
        .stdout(stdout)
        .output()
        .context("failed to run `gradlew`")
        .and_then(|output| {
            if !output.status.success() {
                Err(handle_gradlew_status(output.status))
            } else {
                Ok(output.stdout)
            }
        })
}

fn handle_gradlew_status(status: std::process::ExitStatus) -> anyhow::Error {
    match status.code() {
        Some(code) => anyhow::anyhow!("`gradlew` exited with status code: {}", code),
        None => anyhow::anyhow!("`gradlew` terminated by signal."),
    }
}

pub mod chver;
pub mod set_new;
pub mod utils;
