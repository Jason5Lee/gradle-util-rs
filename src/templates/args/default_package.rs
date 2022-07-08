const JAVA_KEYWORDS: phf::Set<&'static str> = phf::phf_set! {"abstract", "continue", "for", "new", "switch", "assert", "default", "goto", "package", "synchronized", "boolean", "do", "if", "private", "this", "break", "double", "implements", "protected", "throw", "byte", "else", "import", "public", "throws", "case", "enum", "instanceof", "return", "transient", "catch", "extends", "int", "short", "try", "char", "final", "interface", "static", "void", "class", "finally", "long", "strictfp", "volatile", "const", "float", "native", "super", "while"};
const KOTLIN_HARD_KEYWORDS: phf::Set<&'static str> = phf::phf_set! {"as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if", "in", "interface", "is", "null", "object", "package", "return", "super", "this", "throw", "true", "try", "typealias", "typeof", "val", "var", "when", "while"};

const PACKAGE_ERR_PREFIX: &str = "default package name invalid and cannot be fixed";
const PACKAGE_ERR_SUFFIX: &str = "please manually define the package";

pub(super) fn get_package(group: &str, artifact: &str) -> Result<String, String> {
    let mut result = String::new();

    for group_path in group.split('.') {
        let item = fix_package_item(group_path)?;

        result.push_str(&item);
        result.push('.');
    }
    let item = fix_package_item(artifact)?;
    result.push_str(&item);
    Ok(result)
}

// Fix the package item https://docs.oracle.com/javase/tutorial/java/package/namingpkgs.html
// It assumes the `item` is either extracted from a valid group or a valid name.
// Which means it should not be empty.
fn fix_package_item(item: &str) -> Result<String, String> {
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
        return Err(format!(
            "{}, empty item found, {}",
            PACKAGE_ERR_PREFIX, PACKAGE_ERR_SUFFIX
        ));
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

fn fix_non_numeric_char(ch: char) -> Result<char, String> {
    if ch.is_alphabetic() {
        Ok(ch.to_ascii_lowercase())
    } else if ch == '-' {
        Ok('_')
    } else {
        Err(format!(
            "{}, invalid char `{}` found, {}",
            PACKAGE_ERR_PREFIX, ch, PACKAGE_ERR_SUFFIX
        ))
    }
}
