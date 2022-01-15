use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use crate::{Logged, LoggedSideEffect};

const GRADDLE_PROPERTIES_FILENAME: &str = "gradle.properties";
const WRAPPER_PROPERTIES_FILENAME: &str = "gradle-wrapper.properties";

pub fn set_new(
    watch_dirs: Vec<String>,
    version: String,
    watch_duration: Duration,
) -> Result<(), Logged> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::watcher(tx, watch_duration)
        .map_err(|err| log_error!("Unable to create watcher. {}", err))?;
    for watch_dir in watch_dirs.into_iter() {
        watcher
            .watch(watch_dir, RecursiveMode::Recursive)
            .map_err(|err| log_error!("Unable to watch. {}", err))?
    }

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(mut path)) => {
                if path.file_name() == Some(OsStr::new(GRADDLE_PROPERTIES_FILENAME)) {
                    path.pop();
                    log::info!("New gradle project detected at: {:?}.", path);

                    path.push("gradle/wrapper");

                    (|| -> Result<(), Logged> {
                        std::fs::create_dir_all(&path).map_err(|err| {
                            log_error!("Failed to create directory `{:?}`. {}", path, err)
                        })?;

                        path.push(WRAPPER_PROPERTIES_FILENAME);
                        write_wrapper_properties(&path, &version)
                    })()
                    .ignore_logged_error();
                }
            }
            Ok(DebouncedEvent::Error(err, _)) => {
                log::error!("Error while watching directories. {}", err)
            }

            Err(err) => log::error!("Error while receiving watch event. {}", err),
            _ => {}
        }
    }
}

fn wrapper_properties_content(version: &str) -> String {
    format!(
        r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-{}-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#,
        version
    )
}

fn write_wrapper_properties(path: &PathBuf, version: &str) -> Result<(), Logged> {
    std::fs::write(path, wrapper_properties_content(version)).map_err(|err| {
        log_error!(
            "Error while writing to gradle wrapper properties file. {}.",
            err
        )
    })
}
