use std::path::Path;

pub const WRAPPER_PROPERTIES_PATH: &str = "gradle/wrapper/gradle-wrapper.properties";

pub fn create_dir_write_file<P: AsRef<Path>, R: ?Sized>(
    overwrite: bool,
    path: P,
    content: &mut R,
) -> std::io::Result<bool>
where
    R: std::io::Read,
{
    let path = path.as_ref();
    if !overwrite && path.exists() {
        return Ok(false);
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file = std::fs::File::create(path)?;
    std::io::copy(content, &mut file)?;
    Ok(true)
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

pub fn create_dir_write_wrapper_properties<P: AsRef<Path>>(
    path: P,
    version: &str,
) -> std::io::Result<()> {
    create_dir_write_file(
        true,
        path,
        &mut std::io::Cursor::new(wrapper_properties_content(version)),
    )
    .map(|_| ())
}
