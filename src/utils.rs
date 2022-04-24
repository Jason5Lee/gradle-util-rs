use crate::{log_error_with_timestamp, Logged};
use std::path::Path;

pub const WRAPPER_PROPERTIES_FILENAME: &str = "gradle-wrapper.properties";
pub const WRAPPER_PROPERTIES_DIR: &str = "gradle/wrapper";
pub const WRAPPER_PROPERTIES_PATH: &str = "gradle/wrapper/gradle-wrapper.properties";

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

pub fn write_wrapper_properties<P: AsRef<Path>>(path: P, version: &str) -> Result<(), Logged> {
    std::fs::write(path, wrapper_properties_content(version)).map_err(|err| {
        log_error_with_timestamp(format_args!("error while writing to gradle wrapper properties file, {}", err))
    })
}
