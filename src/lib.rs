#[derive(Clone, Copy)]
pub struct Logged;

macro_rules! log_error {
    (target: $target:expr, $($arg:tt)+) => ({
        log::log!(target: $target, log::Level::Error, $($arg)+);
        Logged
    });
    ($($arg:tt)+) => ({
        log::log!(log::Level::Error, $($arg)+);
        Logged
    });
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        log::log!(target: $target, $lvl, $($arg)+);
        Logged
    });
    ($lvl:expr, $($arg:tt)+) => ({
        log::log!($lvl, $($arg)+);
        Logged
    })
}

pub trait LoggedSideEffect {
    fn ignore_logged_error(self);
}
impl LoggedSideEffect for Result<(), Logged> {
    fn ignore_logged_error(self) {}
}

pub mod set_new;
pub mod chver;
pub(crate) mod utils;