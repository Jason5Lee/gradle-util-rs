use crate::{run_gradlew_wrapper, utils, Logged};
use std::path::PathBuf;
use std::process::Stdio;

pub fn chver(project_dir: PathBuf, ver: String, yolo: bool) -> Result<(), Logged> {
    if yolo {
        utils::write_wrapper_properties(utils::WRAPPER_PROPERTIES_PATH, &ver)?;
    }
    run_gradlew_wrapper(
        project_dir,
        &["wrapper", "--gradle-version", &ver],
        Stdio::null(),
    )?;
    Ok(())
}
