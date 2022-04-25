use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;
use std::{path::Path, sync::mpsc};

use crate::{log_error, log_error_with_timestamp, utils, Logged, LoggedSideEffect};

fn is_gradle_project_file(path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        if file_name == OsStr::new("build.gradle") || file_name == OsStr::new("build.gradle.kts") {
            return true;
        }
    }
    false
}
pub fn set_new(
    watch_dirs: Vec<PathBuf>,
    version: String,
    watch_duration: Duration,
) -> Result<(), Logged> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::watcher(tx, watch_duration)
        .map_err(|err| log_error(format_args!("unable to create watcher, {}", err)))?;
    for watch_dir in watch_dirs.into_iter() {
        watcher
            .watch(watch_dir, RecursiveMode::Recursive)
            .map_err(|err| log_error(format_args!("unable to watch, {}", err)))?
    }

    loop {
        match rx.recv() {
            Ok(DebouncedEvent::Create(mut path)) => {
                if is_gradle_project_file(&path) {
                    path.pop();
                    eprintln!(
                        "[{}] new gradle project detected at `{}`",
                        chrono::Local::now(),
                        path.display()
                    );

                    path.push(utils::WRAPPER_PROPERTIES_DIR);

                    (|| -> Result<(), Logged> {
                        std::fs::create_dir_all(&path).map_err(|err| {
                            log_error_with_timestamp(format_args!(
                                "failed to create directory `{}`, {}",
                                path.display(),
                                err
                            ))
                        })?;

                        path.push(utils::WRAPPER_PROPERTIES_FILENAME);
                        utils::write_wrapper_properties(&path, &version)
                    })()
                    .ignore_logged_error();
                }
            }
            Ok(DebouncedEvent::Error(err, _)) => {
                log_error_with_timestamp(format_args!("error while watching directories, {}", err));
            }

            Err(err) => {
                log_error_with_timestamp(format_args!(
                    "error while receiving watch event, {}",
                    err
                ));
            }
            _ => {}
        }
    }
}
