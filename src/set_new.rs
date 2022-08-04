use anyhow::Context;
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;
use std::{path::Path, sync::mpsc};

use crate::{log_error_with_timestamp, utils};

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
) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::watcher(tx, watch_duration).context("unable to create watcher")?;
    for watch_dir in watch_dirs.into_iter() {
        watcher
            .watch(watch_dir, RecursiveMode::Recursive)
            .context("unable to watch")?;
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

                    path.push(utils::WRAPPER_PROPERTIES_PATH);

                    utils::create_dir_write_wrapper_properties(&path, &version).unwrap_or_else(
                        |err| {
                            log_error_with_timestamp(
                                anyhow::Error::from(err).context(format!(
                                    "failed to write file at `{}`",
                                    path.display()
                                )),
                            )
                        },
                    )
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
