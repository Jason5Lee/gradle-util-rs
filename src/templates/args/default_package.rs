use crate::{log_error, Logged};

const JAVA_KEYWORDS: phf::Set<&'static str> = phf::phf_set! {"abstract", "continue", "for", "new", "switch", "assert", "default", "goto", "package", "synchronized", "boolean", "do", "if", "private", "this", "break", "double", "implements", "protected", "throw", "byte", "else", "import", "public", "throws", "case", "enum", "instanceof", "return", "transient", "catch", "extends", "int", "short", "try", "char", "final", "interface", "static", "void", "class", "finally", "long", "strictfp", "volatile", "const", "float", "native", "super", "while"};
const KOTLIN_HARD_KEYWORDS: phf::Set<&'static str> = phf::phf_set! {"as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if", "in", "interface", "is", "null", "object", "package", "return", "super", "this", "throw", "true", "try", "typealias", "typeof", "val", "var", "when", "while"};

const PACKAGE_ERR_PREFIX: &str = "default package name invalid and cannot be fixed";
const PACKAGE_ERR_SUFFIX: &str = "please assign the package";

#[derive(Default)]
pub(super) struct PackageInfo {
    pub(super) package: String,
    pub(super) package_path: String,
}

pub(super) fn analysis_package(package: &str) -> Result<PackageInfo, Logged> {
    let mut package_info = PackageInfo::default();
    let mut first = true;

    for item in package.split('.') {
        let item = fix_package_item(item)?;
        if !first {
            package_info.package.push('.');
            package_info.package_path.push('/');
        }
        first = false;

        package_info.package.push_str(&item);
        package_info.package_path.push_str(&item);
    }
    Ok(package_info)
}

pub(super) fn get_package(group: &str, artifact: &str) -> Result<PackageInfo, Logged> {
    let mut package_info = PackageInfo::default();

    for group_path in group.split('.') {
        let item = fix_package_item(group_path)?;

        package_info.package.push_str(&item);
        package_info.package.push('.');

        package_info.package_path.push_str(&item);
        package_info.package_path.push('/');
    }
    let item = fix_package_item(artifact)?;
    package_info.package.push_str(&item);
    package_info.package_path.push_str(&item);
    Ok(package_info)
}

// Fix the package item https://docs.oracle.com/javase/tutorial/java/package/namingpkgs.html
// It assumes the `item` is either extracted from a valid group or a valid name.
// Which means it should not be empty.
fn fix_package_item(item: &str) -> Result<String, Logged> {
    // Note: we can actually use kotlin keyword in package,
    // but in the code it need to be wrapped in ``.
    // This makes the logic very complicated so we just change it.
    if JAVA_KEYWORDS.contains(item) || KOTLIN_HARD_KEYWORDS.contains(item) {
        return Ok(format!("{}_", item));
    }

    let mut item_chars = item.chars();
    let first = item_chars.next();
    let mut result: String;
    if let Some(first) = first {
        if first.is_numeric() {
            result = String::with_capacity(item.len() + 1);
            result.push('_');
            result.push(first);
        } else {
            result = String::with_capacity(item.len());
            result.push(fix_non_numeric_char(first)?);
        }
    } else {
        return Err(log_error(format_args!(
            "{}, empty item found in package, {}",
            PACKAGE_ERR_PREFIX, PACKAGE_ERR_SUFFIX
        )));
    };

    for ch in item_chars {
        if ch.is_numeric() {
            result.push(ch)
        } else {
            result.push(fix_non_numeric_char(ch)?)
        }
    }

    Ok(result)
}

fn fix_non_numeric_char(ch: char) -> Result<char, Logged> {
    if ch.is_alphabetic() {
        Ok(ch.to_ascii_lowercase())
    } else if ch == '-' {
        Ok('_')
    } else {
        Err(log_error(format_args!(
            "{}, invalid char `{}` found in package, {}",
            PACKAGE_ERR_PREFIX, ch, PACKAGE_ERR_SUFFIX
        )))
    }
}
