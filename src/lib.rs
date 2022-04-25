use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use termcolor::WriteColor;

#[derive(Clone, Copy)]
pub struct Logged;

pub trait LoggedSideEffect {
    fn ignore_logged_error(self);
}
impl LoggedSideEffect for Result<(), Logged> {
    fn ignore_logged_error(self) {}
}

pub fn log_error(err: std::fmt::Arguments) -> Logged {
    let stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    let mut stderr = stderr.lock();
    let _ = stderr.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Red)));
    let _ = stderr.write(b"error");
    let _ = stderr.reset();
    let _ = writeln!(stderr, ": {}", err);
    Logged
}

pub fn log_error_with_timestamp(err: std::fmt::Arguments) -> Logged {
    let stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
    let mut stderr = stderr.lock();
    let _ = write!(stderr, "[{}]", chrono::Local::now());
    let _ = stderr.set_color(termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Red)));
    let _ = stderr.write(b"error");
    let _ = stderr.reset();
    let _ = writeln!(stderr, ": {}", err);
    Logged
}

pub fn log_warn(warn: std::fmt::Arguments) {
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
) -> Result<Vec<u8>, Logged> {
    let mut wrapper = project_dir;
    wrapper.push(GRADLEW);
    std::process::Command::new(wrapper)
        .args(args)
        .stdout(stdout)
        .output()
        .map_err(|err| log_error(format_args!("failed to run `gradlew`, {}", err)))
        .and_then(|output| {
            if !output.status.success() {
                Err(handle_gradlew_status(output.status))
            } else {
                Ok(output.stdout)
            }
        })
}

fn handle_gradlew_status(status: std::process::ExitStatus) -> Logged {
    match status.code() {
        Some(code) => log_error(format_args!("`gradlew` exited with status code: {}", code)),
        None => log_error(format_args!("`gradlew` terminated by signal.")),
    }
}

pub mod chver;
pub mod set_new;
pub mod templates;
pub(crate) mod utils;
