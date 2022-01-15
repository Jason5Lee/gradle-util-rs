pub mod version;

#[cfg(test)]
mod tests;

use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::fs;

use crate::Logged;

static VERSION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?m)^distributionUrl=(?P<prefix>.*)gradle-(?P<version>[0-9.]+)-(?P<type>bin|all).zip$"#,
    )
    .unwrap()
});

const WRAPPER_PROPERTIES_PATH: &str = "gradle/wrapper/gradle-wrapper.properties";
fn replace_version<'a>(
    versions: &[version::GradleVersion],
    properties_content: &'a str,
) -> Cow<'a, str> {
    let version_regex = &VERSION_REGEX;
    version_regex.replace(properties_content, |caps: &Captures| {
        if let Ok(current_version) = version::GradleVersion::parse(&caps["version"]) {
            if let Some(new_version) = versions
                .iter()
                .find(|&ver| ver.can_replace(&current_version))
            {
                return format!(
                    "distributionUrl={}gradle-{}-{}.zip",
                    &caps["prefix"], new_version, &caps["type"]
                );
            }
        }
        caps[0].to_string()
    })
}

pub fn update(versions: Vec<String>) -> Result<(), Logged> {
    let properties = fs::read_to_string(WRAPPER_PROPERTIES_PATH).map_err(|err| {
        log_error!(
            "Error while opening gradle wrapper properties file. {}",
            err
        )
    })?;
    let new_properties = replace_version(
        &versions
            .into_iter()
            .map(|v| {
                version::GradleVersion::parse(&v)
                    .map_err(|err| log_error!("Invalid version argument {}. {}", v, err))
            })
            .collect::<Result<Vec<version::GradleVersion>, Logged>>()?,
        &properties,
    );
    fs::write(WRAPPER_PROPERTIES_PATH, &*new_properties)
        .map_err(|err| log_error!("Failed to write to wrapper properties file. {}", err))
}
