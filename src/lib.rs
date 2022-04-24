use std::io::Write;
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

pub mod chver;
pub mod set_new;
pub(crate) mod utils;
