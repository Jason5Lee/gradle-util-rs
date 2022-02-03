use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::time::Duration;
use std::ffi::OsStr;

use crate::{Logged, LoggedSideEffect, utils};

const GRADDLE_PROPERTIES_FILENAME: &str = "gradle.properties";

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

                    path.push(utils::WRAPPER_PROPERTIES_DIR);

                    (|| -> Result<(), Logged> {
                        std::fs::create_dir_all(&path).map_err(|err| {
                            log_error!("Failed to create directory `{:?}`. {}", path, err)
                        })?;

                        path.push(utils::WRAPPER_PROPERTIES_FILENAME);
                        utils::write_wrapper_properties(&path, &version)
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
