#[test]
fn test_replace_newer() {
    let old = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#;
    assert_eq!(
        r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3.3-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#,
        super::replace_version(
            &[super::version::GradleVersion::parse("7.3.3").unwrap()],
            old
        )
    );
}

#[test]
fn test_not_replace_older() {
    let old = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3.4-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#;
    assert_eq!(
        old,
        super::replace_version(
            &[super::version::GradleVersion::parse("7.3.3").unwrap()],
            old
        )
    );
}
#[test]
fn test_not_replace_different_minor() {
    let old = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3.4-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#;
    assert_eq!(
        old,
        super::replace_version(
            &[super::version::GradleVersion::parse("7.4.4").unwrap()],
            old
        )
    );
}
#[test]
fn test_not_replace_different_major() {
    let old = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3.4-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#;
    assert_eq!(
        old,
        super::replace_version(
            &[super::version::GradleVersion::parse("8.3.4").unwrap()],
            old
        )
    );
}

#[test]
fn test_replace_multiple() {
    let old = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3.1-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#;
    assert_eq!(
        r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3.3-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"#,
        super::replace_version(
            &[
                super::version::GradleVersion::parse("7.3").unwrap(),
                super::version::GradleVersion::parse("7.4.3").unwrap(),
                super::version::GradleVersion::parse("8.3.3").unwrap(),
                super::version::GradleVersion::parse("7.3.3").unwrap()
            ],
            old
        )
    );
}
