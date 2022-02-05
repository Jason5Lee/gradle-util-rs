use crate::{utils, Logged};

#[cfg(windows)]
const GRADLEW: &str = "./gradlew.bat";
#[cfg(not(windows))]
const GRADLEW: &str = "./gradlew";

pub fn chver(ver: String, yolo: bool) -> Result<(), Logged> {
    if yolo {
        utils::write_wrapper_properties(utils::WRAPPER_PROPERTIES_PATH, &ver)?;
    }
    run_gradlew_wrapper(&ver)
}

fn run_gradlew_wrapper(ver: &str) -> Result<(), Logged> {
    std::process::Command::new(GRADLEW)
        .args(&["wrapper", "--gradle-version", ver])
        .spawn()
        .map_err(|err| log_error!("Failed to run `gradlew`. {}.", err))
        .and_then(|mut child| {
            child
                .wait()
                .map_err(|err| log_error!("Failed to run `gradlew`. {}.", err))
                .and_then(|status| {
                    if !status.success() {
                        Err(handle_gradlew_status(status))
                    } else {
                        Ok(())
                    }
                })
        })
}

fn handle_gradlew_status(status: std::process::ExitStatus) -> Logged {
    match status.code() {
        Some(code) => log_error!("`gradlew` exited with status code: {}.", code),
        None => log_error!("`gradlew` terminated by signal."),
    }
}
